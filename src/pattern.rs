use std::collections::BTreeSet;
use std::iter::Map;
use std::usize;
use std::vec::IntoIter;

use bitris::{OrderCursor, PopOp};
use bitris::prelude::Shape;
use itertools::{Itertools, repeat_n};
use thiserror::Error;
use tinyvec::{Array, ArrayVec};

use crate::{ShapeCounter, ShapeOrder, ShapeSequence};
use crate::bit_shapes::BitShapes;

/// Calculate the count of permutations.
fn calculate_permutation_size(len: usize, pop: usize) -> usize {
    debug_assert!(0 < pop && pop <= len);
    ((len - pop + 1)..len).fold(len, |sum, it| sum * it)
}

/// A collection of elements to define the order/sequence of the shapes.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum PatternElement {
    /// A fixed shape (like `T`)
    One(Shape),

    /// A sequence fixed shapes (like `TIO`)
    /// If you want to specify a length longer than BitShapes supports, split it into `One` or shorter `Fixed`.
    Fixed(BitShapes),

    /// One from all shapes (like. `*`)
    Wildcard,

    /// Permutations by taking `usize` shapes from `ShapeCounter`. Duplicates are not removed.
    /// (like `[TIO]p2`, `[JJSZ]p3`)
    ///
    /// Panic if usize(the pop size) is larger than the count of shapes(ShapeCounter).
    Permutation(ShapeCounter, usize),

    /// Permutations by taking all shapes from `ShapeCounter`. Duplicates are not removed.
    /// (like `[TIOLJSZ]p7`, `*!`)
    ///
    /// Panic if the shape counter is empty.
    Factorial(ShapeCounter),
}

impl PatternElement {
    /// Returns all `Vec<Shape>`s represented by the element.
    pub fn to_shapes_vec(&self) -> Vec<Vec<Shape>> {
        match self {
            PatternElement::One(shape) => vec![vec![*shape]],
            PatternElement::Fixed(shapes) => vec![shapes.to_vec()],
            PatternElement::Wildcard => Shape::all_iter().map(|it| vec![it]).collect(),
            PatternElement::Permutation(counter, pop) => {
                let pop = *pop;
                assert!(0 < pop && pop <= counter.len());
                counter.to_pairs().into_iter()
                    .flat_map(|(shape, count)| { repeat_n(shape, count as usize).into_iter() })
                    .permutations(pop)
                    .collect_vec()
            }
            PatternElement::Factorial(counter) => {
                assert!(!counter.is_empty());
                counter.to_pairs().into_iter()
                    .flat_map(|(shape, count)| { repeat_n(shape, count as usize).into_iter() })
                    .permutations(counter.len())
                    .collect_vec()
            }
        }
    }

    /// The count of shapes the pattern has.
    pub fn count_shapes(&self) -> usize {
        match self {
            PatternElement::One(..) => 1,
            PatternElement::Fixed(..) => 1,
            PatternElement::Wildcard => 7,
            PatternElement::Permutation(counter, pop) => {
                let pop = *pop;
                assert!(0 < pop && pop <= counter.len());
                calculate_permutation_size(counter.len(), pop)
            }
            PatternElement::Factorial(counter) => calculate_permutation_size(counter.len(), counter.len()),
        }
    }

    /// The number of elements in one shapes.
    pub fn dim_shapes(&self) -> usize {
        match *self {
            PatternElement::One(..) => 1,
            PatternElement::Fixed(shapes) => shapes.len(),
            PatternElement::Wildcard => 1,
            PatternElement::Permutation(counter, pop) => {
                assert!(0 < pop && pop <= counter.len());
                pop
            }
            PatternElement::Factorial(counter) => {
                assert!(!counter.is_empty());
                counter.len()
            }
        }
    }

    /// The count of shape counters the element has.
    pub fn to_shape_counter_vec(&self) -> Vec<ShapeCounter> {
        match *self {
            PatternElement::One(shape) => vec![ShapeCounter::from(shape)],
            PatternElement::Fixed(shapes) => vec![ShapeCounter::from(shapes.to_vec())],
            PatternElement::Wildcard => Shape::all_iter().map(|shape| ShapeCounter::from(shape)).collect(),
            PatternElement::Permutation(counter, pop) => counter.subset(pop),
            PatternElement::Factorial(counter) => vec![counter],
        }
    }
}

/// Define the order/sequence of the shapes.
/// ```
/// use bitris_commands::prelude::*;
/// use PatternElement::*;
///
/// // `T**` (e.g. TTT, TTI, TTO, ..., TZZ: 49 sequences)
/// let pattern = Pattern::try_from(vec![One(Shape::T), Wildcard, Wildcard]).unwrap();
/// assert_eq!(pattern.len_shapes_vec(), 49);
/// assert_eq!(pattern.dim_shapes(), 3);
///
/// // `TI` (1 sequence)
/// let pattern = Pattern::try_from(vec![Fixed(BitShapes::try_from(vec![Shape::T, Shape::I]).unwrap())]).unwrap();
/// assert_eq!(pattern.len_shapes_vec(), 1);
/// assert_eq!(pattern.dim_shapes(), 2);
///
/// // `[TIOLJSZ]p3` (e.g. TIO, TIL, ..., TOI, ..., TZS: 210 sequences)
/// let pattern = Pattern::try_from(vec![Permutation(ShapeCounter::one_of_each(), 3)]).unwrap();
/// assert_eq!(pattern.len_shapes_vec(), 210);
/// assert_eq!(pattern.dim_shapes(), 3);
/// ```
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Pattern {
    elements: Vec<PatternElement>,
}

/// A collection of errors that occur when making the pattern.
#[derive(Error, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum PatternCreationError {
    #[error("This does not have shape sequences.")]
    NoShapeSequences,
    #[error("The elements contains invalid permutation.")]
    ContainsInvalidPermutation,
}

impl TryFrom<Vec<PatternElement>> for Pattern {
    type Error = PatternCreationError;

    fn try_from(elements: Vec<PatternElement>) -> Result<Self, Self::Error> {
        Pattern::try_new(elements)
    }
}

impl Pattern {
    pub fn try_new(elements: Vec<PatternElement>) -> Result<Self, PatternCreationError> {
        use PatternElement::*;
        use PatternCreationError::*;

        if elements.is_empty() {
            return Err(NoShapeSequences);
        }

        for element in &elements {
            match element {
                Permutation(counter, pop) => {
                    if counter.len() <= 0 || *pop <= 0 || counter.len() < *pop {
                        return Err(ContainsInvalidPermutation);
                    }
                }
                _ => {}
            }
        }

        Ok(Self { elements })
    }

    #[allow(dead_code)]
    fn walk_shapes_walk(&self, visitor: &mut impl FnMut(&[Shape])) {
        struct Buffer<'a> {
            all_vec: &'a Vec<Vec<Vec<Shape>>>,
            buffer: Vec<Shape>,
        }

        impl Buffer<'_> {
            fn build(&mut self, index: usize, visitor: &mut impl FnMut(&[Shape])) {
                if index < self.all_vec.len() - 1 {
                    for shapes in &self.all_vec[index] {
                        let size = self.buffer.len();
                        self.buffer.extend(shapes.iter());
                        self.build(index + 1, visitor);
                        self.buffer.resize(size, Shape::T);
                    }
                } else {
                    for shapes in &self.all_vec[index] {
                        let size = self.buffer.len();
                        self.buffer.extend(shapes.iter());
                        visitor(&self.buffer.as_slice());
                        self.buffer.resize(size, Shape::T);
                    }
                }
            }
        }

        let all_vec: Vec<Vec<Vec<Shape>>> = self.elements.clone()
            .into_iter()
            .map(|it| it.to_shapes_vec())
            .collect();

        let mut buffer = Buffer {
            all_vec: &all_vec,
            buffer: Vec::with_capacity(self.dim_shapes()),
        };

        buffer.build(0, visitor);
    }

    #[allow(dead_code)]
    fn to_shapes_vec(&self) -> Vec<Vec<Shape>> {
        if self.elements.is_empty() {
            return Vec::new();
        }

        let mut vec = Vec::with_capacity(self.len_shapes_vec());
        self.walk_shapes_walk(&mut |shapes| {
            vec.push(shapes.iter().map(|&shape| shape).collect());
        });
        vec
    }

    /// Returns all sequences represented by the pattern.
    pub fn to_sequences(&self) -> Vec<ShapeSequence> {
        self.to_shapes_vec().into_iter()
            .map(|it| ShapeSequence::new(it))
            .collect()
    }

    /// Returns all orders represented by the pattern.
    pub fn to_orders(&self) -> Vec<ShapeOrder> {
        self.to_shapes_vec().into_iter()
            .map(|it| ShapeOrder::new(it))
            .collect()
    }

    /// The count of shapes the pattern has.
    pub fn len_shapes_vec(&self) -> usize {
        if self.elements.is_empty() {
            return 0;
        }
        self.elements.iter()
            .map(|it| it.count_shapes())
            .fold(1, |sum, it| sum * it)
    }

    /// The number of elements in one shapes.
    pub fn dim_shapes(&self) -> usize {
        debug_assert!(!self.elements.is_empty(), "The pattern do not have shapes.");
        self.elements.iter()
            .map(|it| it.dim_shapes())
            .fold(0, |sum, it| sum + it)
    }

    #[allow(dead_code)]
    fn shape_counters_walk(&self, visitor: &mut impl FnMut(ShapeCounter)) {
        struct Buffer<'a> {
            all_vec: &'a Vec<Vec<ShapeCounter>>,
        }

        impl Buffer<'_> {
            fn build(&mut self, index: usize, buffer: ShapeCounter, visitor: &mut impl FnMut(ShapeCounter)) {
                if index < self.all_vec.len() - 1 {
                    for shapes in &self.all_vec[index] {
                        self.build(index + 1, buffer + shapes, visitor);
                    }
                } else {
                    for shapes in &self.all_vec[index] {
                        visitor(buffer + shapes);
                    }
                }
            }
        }

        let all_vec: Vec<Vec<ShapeCounter>> = self.elements.clone()
            .into_iter()
            .map(|it| it.to_shape_counter_vec())
            .collect();

        let mut buffer = Buffer {
            all_vec: &all_vec,
        };

        buffer.build(0, ShapeCounter::empty(), visitor);
    }

    /// Return all shape counters that the pattern may have.
    pub fn to_shape_counter_vec(&self) -> Vec<ShapeCounter> {
        if self.elements.is_empty() {
            return Vec::new();
        }

        let mut vec = Vec::<ShapeCounter>::new();
        self.shape_counters_walk(&mut |shape_counter| {
            vec.push(shape_counter);
        });
        vec
    }

    /// Returns true if a shape sequence is represented by pattern.
    ///
    /// Note that even if the shape sequence exceeds the length of the pattern, it returns true if the subsequence from the beginning satisfies the condition.
    /// ```
    /// use bitris_commands::prelude::*;
    /// use Shape::*;
    /// let pattern = Pattern::try_from(vec![
    ///     PatternElement::One(T),
    ///     PatternElement::Wildcard,
    ///     PatternElement::Permutation(vec![L, L, J].into(), 2),
    /// ]).unwrap();
    ///
    /// // Success
    /// assert!(pattern.contains(&vec![T, T, L, J].into()));
    /// assert!(pattern.contains(&vec![T, I, L, J].into()));
    /// assert!(pattern.contains(&vec![T, O, L, J].into()));
    /// // ....
    /// assert!(pattern.contains(&vec![T, Z, L, J].into()));
    ///
    /// assert!(pattern.contains(&vec![T, T, L, L].into()));
    /// assert!(pattern.contains(&vec![T, T, L, J].into()));
    /// assert!(pattern.contains(&vec![T, T, J, L].into()));
    ///
    /// assert!(pattern.contains(&vec![T, T, J, L, O].into()));
    ///
    /// // Failure
    /// assert!(!pattern.contains(&vec![T].into()));
    /// assert!(!pattern.contains(&vec![I, T, L, J].into()));
    /// assert!(!pattern.contains(&vec![T, T, L, O].into()));
    /// ````
    pub fn contains(&self, shape_sequence: &ShapeSequence) -> bool {
        let dim = self.dim_shapes();
        if shape_sequence.len() < dim {
            return false;
        }

        let all_shapes = shape_sequence.shapes();
        let mut index = 0;
        for element in self.elements.iter() {
            match element {
                PatternElement::One(shape) => {
                    if all_shapes[index] != *shape {
                        return false;
                    }
                    index += 1;
                }
                PatternElement::Fixed(shapes) => {
                    let shapes = shapes.to_vec();
                    let pop = shapes.len();
                    if shapes.as_slice() != &all_shapes[index..index + pop] {
                        return false;
                    }
                    index += pop;
                }
                PatternElement::Wildcard => {
                    index += 1;
                }
                PatternElement::Permutation(counter, pop) => {
                    assert!(0 < *pop && *pop <= counter.len());
                    if !counter.contains_all(&ShapeCounter::from(&all_shapes[index..index + pop])) {
                        return false;
                    }
                    index += pop;
                }
                PatternElement::Factorial(counter) => {
                    assert!(!counter.is_empty());
                    let pop = counter.len();
                    if *counter != ShapeCounter::from(&all_shapes[index..index + pop]) {
                        return false;
                    }
                    index += pop;
                }
            }
        }
        true
    }

    /// Returns a matcher that determines whether a sequence of the shapes is contained in the pattern.
    pub fn new_matcher(&self) -> ShapeMatcher {
        self.into()
    }
}

/// Determines if a sequence of shapes matches a pattern element.
#[derive(Copy, Clone, PartialEq, PartialOrd, Hash, Debug)]
struct ShapeMatcherBuffer {
    used: ShapeCounter,
    counter: usize,
}

impl ShapeMatcherBuffer {
    #[inline]
    fn empty() -> Self {
        Self { used: ShapeCounter::empty(), counter: 0 }
    }

    fn match_shape(self, element: &PatternElement, target: Shape) -> (bool, Option<Self>) {
        #[inline]
        fn new_next_matcher(
            buffer: ShapeMatcherBuffer,
            element_dim_shapes: usize,
            target: Shape,
        ) -> Option<ShapeMatcherBuffer> {
            if buffer.counter + 1 < element_dim_shapes {
                Some(ShapeMatcherBuffer {
                    used: buffer.used + target,
                    counter: buffer.counter + 1,
                })
            } else {
                None
            }
        }

        match element {
            PatternElement::One(shape) => (target == *shape, None),
            PatternElement::Fixed(shapes) => {
                match shapes.get(self.counter) {
                    None => (false, None),
                    Some(shape) => {
                        if target == shape {
                            (true, new_next_matcher(self, shapes.len(), target))
                        } else {
                            (false, None)
                        }
                    }
                }
            }
            PatternElement::Wildcard => (true, None),
            PatternElement::Permutation(shape_counter, pop) => {
                let remaining = shape_counter - self.used;
                if remaining.contains(target) {
                    (true, new_next_matcher(self, *pop, target))
                } else {
                    (false, None)
                }
            }
            PatternElement::Factorial(shape_counter) => {
                let remaining = shape_counter - self.used;
                if remaining.contains(target) {
                    (true, new_next_matcher(self, shape_counter.len(), target))
                } else {
                    (false, None)
                }
            }
        }
    }
}

/// Determines if a sequence of shapes matches a pattern.
#[derive(Copy, Clone, PartialEq, PartialOrd, Hash, Debug)]
pub struct ShapeMatcher<'a> {
    pattern: &'a Pattern,
    current: Option<(usize, ShapeMatcherBuffer)>,
    next: Option<usize>,
    failed: bool,
}

impl<'a> ShapeMatcher<'a> {
    #[inline]
    fn new(pattern: &'a Pattern) -> Self {
        assert!(!pattern.elements.is_empty());
        match pattern.elements.len() {
            1 => Self { pattern, current: Some((0, ShapeMatcherBuffer::empty())), next: None, failed: false },
            _ => Self { pattern, current: Some((0, ShapeMatcherBuffer::empty())), next: Some(1), failed: false },
        }
    }

    /// Returns `true` if a remaining shape exists next.
    #[inline]
    pub fn has_next(&self) -> bool {
        self.current.is_some()
    }

    /// Returns the result of the match and the next matcher.
    /// If the next shape is contained in the pattern, returns `true` and a matcher to match the next shape.
    /// If not contained, returns `false` and an empty matcher with no next shape.
    ///
    /// If the pattern has successfully matched until the end, the returned matcher will always return `true` regardless of the length of the sequence.
    /// For example, if the pattern represents "TSO", the sequence "TSOZ..." will also be considered `true`.
    /// Note that to take the length of the sequence into account, use `has_next()` as well.
    #[inline]
    pub fn match_shape(self, target: Shape) -> (bool, ShapeMatcher<'a>) {
        if self.failed {
            return (false, self);
        }

        return match self.current {
            None => (true, self),
            Some((current_index, current_buffer)) => {
                let (matched, next_buffer) = current_buffer.match_shape(&self.pattern.elements[current_index], target);
                if !matched {
                    return (false, Self {
                        pattern: self.pattern,
                        current: None,
                        next: None,
                        failed: true,
                    });
                }

                (true, match next_buffer {
                    Some(buffer) => Self {
                        pattern: self.pattern,
                        current: Some((current_index, buffer)),
                        next: self.next,
                        failed: false,
                    },
                    None => match self.next {
                        None => Self {
                            pattern: self.pattern,
                            current: None,
                            next: None,
                            failed: false,
                        },
                        Some(next_index) => Self {
                            pattern: self.pattern,
                            current: Some((next_index, ShapeMatcherBuffer::empty())),
                            next: if next_index + 1 < self.pattern.elements.len() {
                                Some(next_index + 1)
                            } else {
                                None
                            },
                            failed: false,
                        },
                    },
                })
            }
        };
    }
}

impl<'a> From<&'a Pattern> for ShapeMatcher<'a> {
    fn from(pattern: &'a Pattern) -> Self {
        Self::new(pattern)
    }
}


#[derive(Clone, PartialEq, PartialOrd, Hash, Debug)]
pub struct PatternHoldExpand<'a> {
    pattern: &'a Pattern,
    vec: Vec<Vec<RelatedPatternElement>>,
}

impl PatternHoldExpand<'_> {
    /// Returns a matcher that determines whether a sequence of the shapes is contained in the pattern.
    pub fn new_matcher(&self) -> ShapeMatcher2 {
        self.into()
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
enum RelatedPatternElement {
    Fixed(Shape),
    Shared(usize),
}

impl<'a> From<&'a Pattern> for PatternHoldExpand<'a> {
    fn from(pattern: &'a Pattern) -> Self {
        fn simplify((element_index, element): (usize, &PatternElement)) -> Vec<RelatedPatternElement> {
            use RelatedPatternElement::*;
            match element {
                PatternElement::One(shape) => vec![Fixed(*shape)],
                PatternElement::Fixed(shapes) => shapes.to_vec().into_iter().map(|shape| Fixed(shape)).collect(),
                PatternElement::Wildcard => vec![Shared(element_index)],
                PatternElement::Permutation(_, pop) => {
                    vec![
                        vec![Shared(element_index)].repeat(*pop),
                    ].concat()
                }
                PatternElement::Factorial(counter) => {
                    vec![
                        vec![Shared(element_index)].repeat(counter.len()),
                    ].concat()
                }
            }
        }

        let simply_elements: Vec<RelatedPatternElement> = pattern.elements.iter()
            .enumerate()
            .flat_map(simplify)
            .collect();

        assert!(!simply_elements.is_empty());

        struct Builder<'a> {
            results: BTreeSet<Vec<RelatedPatternElement>>,
            buffer: Vec<&'a RelatedPatternElement>,
        }

        impl<'a> Builder<'a> {
            fn build(&mut self, cursor: OrderCursor<'a, RelatedPatternElement>) {
                if !cursor.has_next() {
                    self.results.insert(self.buffer.iter().map(|&it| *it).collect());
                    return;
                }

                {
                    let (popped, next_cursor) = cursor.pop(PopOp::First);
                    if let Some(pair) = popped {
                        self.buffer.push(pair);
                        self.build(next_cursor);
                        self.buffer.pop();
                    }
                }
                {
                    let (popped, next_cursor) = cursor.pop(PopOp::Second);
                    if let Some(pair) = popped {
                        self.buffer.push(pair);
                        self.build(next_cursor);
                        self.buffer.pop();
                    }
                }
            }
        }

        let mut builder = Builder {
            results: BTreeSet::default(),
            buffer: Vec::new(),
        };

        let cursor = OrderCursor::from(&simply_elements);
        builder.build(cursor);

        Self {
            pattern,
            vec: builder.results.into_iter().collect(),
        }
    }
}

/// Determines if a sequence of shapes matches a pattern.
#[derive(Copy, Clone, PartialEq, PartialOrd, Hash, Debug)]
pub struct ShapeMatcher3 {
    p_index: usize,
    stored: ArrayVec<[(usize, ShapeCounter); 2]>,
    // (shared_index, used)
    current: Option<usize>, // (expanded_pattern_element_index)
}

/// Determines if a sequence of shapes matches a pattern.
#[derive(Clone, PartialEq, PartialOrd, Hash, Debug)]
pub struct ShapeMatcher2<'a> {
    pattern: &'a PatternHoldExpand<'a>,
    candidates: Vec<ShapeMatcher3>,
    succeed_always: bool,
}

impl<'a> ShapeMatcher2<'a> {
    #[inline]
    fn new(pattern: &'a PatternHoldExpand) -> Self {
        assert!(!pattern.vec.is_empty());
        assert!(pattern.vec.iter().all(|it| !it.is_empty()));

        let candidates = pattern.vec.iter()
            .enumerate()
            .filter(|(_, elements)| !elements.is_empty())
            .map(|(p_index, _)| ShapeMatcher3 {
                p_index,
                stored: ArrayVec::new(),
                current: Some(0),
            })
            .collect();

        Self {
            pattern,
            candidates,
            succeed_always: false,
        }
    }

    /// Returns `true` if a remaining shape exists next.
    #[inline]
    pub fn has_next(&self) -> bool {
        !self.candidates.is_empty() && self.candidates.iter().any(|candidate| candidate.current.is_some())
    }

    /// Returns the result of the match and the next matcher.
    /// If the next shape is contained in the pattern, returns `true` and a matcher to match the next shape.
    /// If not contained, returns `false` and an empty matcher with no next shape.
    ///
    /// If the pattern has successfully matched until the end, the returned matcher will always return `true` regardless of the length of the sequence.
    /// For example, if the pattern represents "TSO", the sequence "TSOZ..." will also be considered `true`.
    /// Note that to take the length of the sequence into account, use `has_next()` as well.
    #[inline]
    pub fn match_shape(&self, target: Shape) -> (bool, ShapeMatcher2<'a>) {
        if self.succeed_always {
            return (true, self.clone());
        }

        if self.candidates.is_empty() {
            return (false, self.clone());
        }

        let mut next_candidates = Vec::<ShapeMatcher3>::new();
        'outer: for candidate in &self.candidates {
            let index = if let Some(index) = candidate.current {
                index
            } else {
                continue 'outer;
            };
            let next_stored = match &self.pattern.vec[candidate.p_index][index] {
                RelatedPatternElement::Fixed(shape) => {
                    if target != *shape {
                        continue 'outer;
                    }
                    candidate.stored
                }
                RelatedPatternElement::Shared(pattern_element_index) => {
                    let gen_next_stored = || -> Option<ArrayVec<[(usize, ShapeCounter); 2]>> {
                        let pattern_element = self.pattern.pattern.elements[*pattern_element_index];
                        let mut next_stored = candidate.stored.clone();

                        let mut nsi = None;
                        for nsi2 in 0..=1 {
                            if candidate.stored.get(nsi2).filter(|it| it.0 == *pattern_element_index).is_some() {
                                nsi = Some(nsi2);
                                break;
                            }
                        }

                        let nsi = if let Some(nsi) = nsi {
                            nsi
                        } else {
                            let i = next_stored.len();
                            next_stored.push((*pattern_element_index, ShapeCounter::empty()));
                            i
                        };

                        next_stored[nsi].1 += target;

                        let matches = match pattern_element {
                            PatternElement::Wildcard => true,
                            PatternElement::Permutation(counter, _) => counter.contains_all(&next_stored[nsi].1),
                            PatternElement::Factorial(counter) => counter.contains_all(&next_stored[nsi].1),
                            _ => panic!("Unreachable"),
                        };

                        if !matches {
                            return None;
                        }

                        let len_used = next_stored[nsi].1.len();
                        let last = len_used == pattern_element.dim_shapes();

                        if last {
                            next_stored.remove(nsi);
                        }

                        Some(next_stored)
                    };

                    if let Some(next_stored) = gen_next_stored() {
                        next_stored
                    } else {
                        continue 'outer;
                    }
                }
            };

            next_candidates.push(ShapeMatcher3 {
                p_index: candidate.p_index,
                stored: next_stored,
                current: if index + 1 < self.pattern.vec[candidate.p_index].len() {
                    Some(index + 1)
                } else {
                    None
                },
            });
        }

        let succeed = !next_candidates.is_empty();
        let succeed_always = succeed && next_candidates.iter().any(|candidate| candidate.current.is_none());
        (succeed, Self {
            pattern: self.pattern,
            candidates: if succeed_always {
                Vec::new()
            } else {
                next_candidates
            },
            succeed_always,
        })
    }
}

impl<'a> From<&'a PatternHoldExpand<'a>> for ShapeMatcher2<'a> {
    fn from(pattern: &'a PatternHoldExpand) -> Self {
        Self::new(pattern)
    }
}


#[cfg(test)]
mod tests {
    use bitris::prelude::Shape;

    use crate::{Pattern, PatternCreationError, PatternElement, PatternHoldExpand, ShapeCounter};
    use crate::bit_shapes::BitShapes;

    #[test]
    fn one() {
        let pattern = PatternElement::One(Shape::I);
        assert_eq!(pattern.to_shapes_vec(), vec![vec![Shape::I]]);
    }

    #[test]
    fn fixed() {
        let shapes = BitShapes::try_from(vec![Shape::T, Shape::O, Shape::L]).unwrap();
        let pattern = PatternElement::Fixed(shapes);
        assert_eq!(pattern.to_shapes_vec(), vec![vec![Shape::T, Shape::O, Shape::L]]);
    }

    #[test]
    fn pattern_permutation() {
        let counter = ShapeCounter::from(vec![Shape::I]);
        let pattern = PatternElement::Permutation(counter, 1);
        assert_eq!(pattern.dim_shapes(), 1);
        assert_eq!(pattern.count_shapes(), 1);

        let counter = ShapeCounter::from(vec![Shape::I, Shape::O, Shape::T]);
        let pattern = PatternElement::Permutation(counter, 1);
        assert_eq!(pattern.dim_shapes(), 1);
        assert_eq!(pattern.count_shapes(), 3);

        let counter = ShapeCounter::from(vec![Shape::I, Shape::O, Shape::T]);
        let pattern = PatternElement::Permutation(counter, 2);
        assert_eq!(pattern.dim_shapes(), 2);
        assert_eq!(pattern.count_shapes(), 6);

        let counter = ShapeCounter::one_of_each();
        let pattern = PatternElement::Permutation(counter, 3);
        assert_eq!(pattern.dim_shapes(), 3);
        assert_eq!(pattern.count_shapes(), 210);

        let counter = ShapeCounter::one_of_each();
        let pattern = PatternElement::Permutation(counter, 5);
        assert_eq!(pattern.dim_shapes(), 5);
        assert_eq!(pattern.count_shapes(), 2520);
    }

    #[test]
    #[should_panic]
    fn invalid_pattern_permutation() {
        let counter = ShapeCounter::from(vec![Shape::I]);
        let invalid_pattern = PatternElement::Permutation(counter, 2);
        invalid_pattern.dim_shapes();
    }

    #[test]
    fn empty() {
        assert_eq!(Pattern::try_from(vec![]).unwrap_err(), PatternCreationError::NoShapeSequences);
    }

    #[test]
    fn contains_invalid_permutation() {
        use PatternElement::*;
        assert_eq!(
            Pattern::try_from(vec![Permutation(ShapeCounter::one_of_each(), 8)]).unwrap_err(),
            PatternCreationError::ContainsInvalidPermutation,
        );
        assert_eq!(
            Pattern::try_from(vec![Permutation(ShapeCounter::one_of_each(), 0)]).unwrap_err(),
            PatternCreationError::ContainsInvalidPermutation,
        );
        assert_eq!(
            Pattern::try_from(vec![Permutation(ShapeCounter::empty(), 0)]).unwrap_err(),
            PatternCreationError::ContainsInvalidPermutation,
        );
        assert_eq!(
            Pattern::try_from(vec![Permutation(ShapeCounter::empty(), 1)]).unwrap_err(),
            PatternCreationError::ContainsInvalidPermutation,
        );
    }

    #[test]
    fn large() {
        let pattern = Pattern::try_from(vec![
            PatternElement::Permutation(ShapeCounter::one_of_each(), 6),
            PatternElement::Permutation(ShapeCounter::one_of_each(), 3),
        ]).unwrap();
        assert_eq!(pattern.len_shapes_vec(), 5040 * 210);
        assert_eq!(pattern.dim_shapes(), 9);
        assert_eq!(pattern.to_sequences().len(), 5040 * 210);
    }

    #[test]
    fn pattern_element_to_shape_counter_vec() {
        use PatternElement::*;
        assert_eq!(
            One(Shape::T).to_shape_counter_vec(),
            vec![ShapeCounter::one(Shape::T)],
        );
        assert_eq!(
            Fixed(BitShapes::try_from(vec![Shape::T, Shape::O]).unwrap()).to_shape_counter_vec(),
            vec![ShapeCounter::one(Shape::T) + Shape::O],
        );
        assert_eq!(
            Wildcard.to_shape_counter_vec(),
            vec![
                ShapeCounter::one(Shape::T),
                ShapeCounter::one(Shape::I),
                ShapeCounter::one(Shape::O),
                ShapeCounter::one(Shape::L),
                ShapeCounter::one(Shape::J),
                ShapeCounter::one(Shape::S),
                ShapeCounter::one(Shape::Z),
            ],
        );
        assert_eq!(
            Permutation(ShapeCounter::one_of_each(), 6).to_shape_counter_vec(),
            vec![
                ShapeCounter::one_of_each() - Shape::T,
                ShapeCounter::one_of_each() - Shape::I,
                ShapeCounter::one_of_each() - Shape::O,
                ShapeCounter::one_of_each() - Shape::L,
                ShapeCounter::one_of_each() - Shape::J,
                ShapeCounter::one_of_each() - Shape::S,
                ShapeCounter::one_of_each() - Shape::Z,
            ],
        );
        assert_eq!(
            Factorial(ShapeCounter::one_of_each()).to_shape_counter_vec(),
            vec![ShapeCounter::one_of_each()],
        );
    }

    #[test]
    fn pattern_to_shape_counters_vec() {
        use PatternElement::*;
        use Shape::*;
        assert_eq!(
            Pattern::try_from(vec![
                One(T),
                Fixed(BitShapes::try_from(vec![T, O]).unwrap()),
                Wildcard,
            ]).unwrap().to_shape_counter_vec(),
            vec![
                ShapeCounter::from(vec![T, T, O, T]),
                ShapeCounter::from(vec![T, T, O, I]),
                ShapeCounter::from(vec![T, T, O, O]),
                ShapeCounter::from(vec![T, T, O, L]),
                ShapeCounter::from(vec![T, T, O, J]),
                ShapeCounter::from(vec![T, T, O, S]),
                ShapeCounter::from(vec![T, T, O, Z]),
            ],
        );
        assert_eq!(
            Pattern::try_from(vec![
                Permutation(ShapeCounter::from(vec![T, T, O, I]), 2),
                Factorial(ShapeCounter::from(vec![S, Z])),
            ]).unwrap().to_shape_counter_vec(),
            vec![
                ShapeCounter::from(vec![O, I, S, Z]),
                ShapeCounter::from(vec![T, O, S, Z]),
                ShapeCounter::from(vec![T, I, S, Z]),
                ShapeCounter::from(vec![T, T, S, Z]),
            ],
        );
    }

    #[test]
    fn can_accept_case1() {
        use Shape::*;
        let pattern = Pattern::try_from(vec![
            PatternElement::One(T),
            PatternElement::Fixed(vec![S, Z].try_into().unwrap()),
            PatternElement::Wildcard,
        ]).unwrap();

        // Success
        assert!(pattern.contains(&vec![T, S, Z, T].into()));
        assert!(pattern.contains(&vec![T, S, Z, I].into()));
        assert!(pattern.contains(&vec![T, S, Z, O].into()));
        assert!(pattern.contains(&vec![T, S, Z, L].into()));
        assert!(pattern.contains(&vec![T, S, Z, J].into()));
        assert!(pattern.contains(&vec![T, S, Z, S].into()));
        assert!(pattern.contains(&vec![T, S, Z, Z].into()));
        assert!(pattern.contains(&vec![T, S, Z, T, O].into()));

        // Failure at One
        assert!(!pattern.contains(&vec![I, S, Z, T].into()));

        // Failure at Fixed
        assert!(!pattern.contains(&vec![T, O, Z, T].into()));
        assert!(!pattern.contains(&vec![T, S, L, T].into()));
    }

    #[test]
    fn can_accept_case2() {
        use Shape::*;
        let pattern = Pattern::try_from(vec![
            PatternElement::One(T),
            PatternElement::Factorial(vec![T, I, O].into()),
        ]).unwrap();

        // Success
        assert!(pattern.contains(&vec![T, T, I, O].into()));
        assert!(pattern.contains(&vec![T, T, O, I].into()));
        assert!(pattern.contains(&vec![T, I, T, O].into()));
        assert!(pattern.contains(&vec![T, I, O, T].into()));
        assert!(pattern.contains(&vec![T, O, T, I].into()));
        assert!(pattern.contains(&vec![T, O, I, T].into()));

        // Failure at One
        assert!(!pattern.contains(&vec![S, T, I, O].into()));

        // Failure at Factorial
        assert!(!pattern.contains(&vec![T, S, I, O].into()));
        assert!(!pattern.contains(&vec![T, T, T, O].into()));
    }

    #[test]
    fn can_accept_case3() {
        use Shape::*;
        let pattern = Pattern::try_from(vec![
            PatternElement::One(T),
            PatternElement::Permutation(vec![L, L, J, S, Z].into(), 2),
        ]).unwrap();

        // Success
        assert!(pattern.contains(&vec![T, L, L].into()));
        assert!(pattern.contains(&vec![T, L, J].into()));
        assert!(pattern.contains(&vec![T, L, S].into()));
        assert!(pattern.contains(&vec![T, L, Z].into()));
        assert!(pattern.contains(&vec![T, J, L].into()));
        assert!(pattern.contains(&vec![T, J, S].into()));
        assert!(pattern.contains(&vec![T, J, Z].into()));
        assert!(pattern.contains(&vec![T, S, L].into()));
        assert!(pattern.contains(&vec![T, S, J].into()));
        assert!(pattern.contains(&vec![T, S, Z].into()));
        assert!(pattern.contains(&vec![T, Z, L].into()));
        assert!(pattern.contains(&vec![T, Z, J].into()));
        assert!(pattern.contains(&vec![T, Z, S].into()));

        // Failure at One
        assert!(!pattern.contains(&vec![O, L, L].into()));

        // Failure at Permutation
        assert!(!pattern.contains(&vec![T, O, J].into()));
        assert!(!pattern.contains(&vec![T, J, J].into()));
    }

    #[test]
    fn cursor_one_accept() {
        use Shape::*;
        let pattern = Pattern::try_from(vec![
            PatternElement::One(T),
            PatternElement::One(I),
            PatternElement::One(O),
        ]).unwrap();

        {
            let mut cursor = pattern.new_matcher();

            for shape in [T, I, O] {
                assert!(cursor.has_next());

                let (accepted, next) = cursor.match_shape(shape);
                assert!(accepted);

                cursor = next;
            }

            assert!(!cursor.has_next());

            let (accepted, cursor) = cursor.match_shape(Z);
            assert!(accepted);
            assert!(!cursor.has_next());
        }

        {
            let cursor = pattern.new_matcher();
            assert!(cursor.has_next());

            let (accepted, cursor) = cursor.match_shape(O);
            assert!(!accepted);
            assert!(!cursor.has_next());

            let (accepted, cursor) = cursor.match_shape(T);
            assert!(!accepted);
            assert!(!cursor.has_next());
        }
    }

    #[test]
    fn cursor_fixed_accept() {
        use Shape::*;
        let pattern = Pattern::try_from(vec![
            PatternElement::Fixed(BitShapes::try_from(vec![S]).unwrap()),
            PatternElement::Fixed(BitShapes::try_from(vec![T, I, O]).unwrap()),
        ]).unwrap();

        {
            let mut cursor = pattern.new_matcher();

            for shape in [S, T, I, O] {
                assert!(cursor.has_next());

                let (accepted, next) = cursor.match_shape(shape);
                assert!(accepted);

                cursor = next;
            }

            assert!(!cursor.has_next());

            let (accepted, cursor) = cursor.match_shape(Z);
            assert!(accepted);
            assert!(!cursor.has_next());
        }

        {
            let cursor = pattern.new_matcher();
            assert!(cursor.has_next());

            let (accepted, cursor) = cursor.match_shape(O);
            assert!(!accepted);
            assert!(!cursor.has_next());

            let (accepted, cursor) = cursor.match_shape(S);
            assert!(!accepted);
            assert!(!cursor.has_next());
        }
    }

    #[test]
    fn cursor_wildcard_accept() {
        use Shape::*;
        let pattern = Pattern::try_from(vec![
            PatternElement::Wildcard,
            PatternElement::Wildcard,
        ]).unwrap();

        {
            for shapes in [[S, T], [T, T]] {
                let mut cursor = pattern.new_matcher();

                for shape in shapes {
                    assert!(cursor.has_next());

                    let (accepted, next) = cursor.match_shape(shape);
                    assert!(accepted);

                    cursor = next;
                }

                assert!(!cursor.has_next());

                let (accepted, cursor) = cursor.match_shape(Z);
                assert!(accepted);
                assert!(!cursor.has_next());
            }
        }
    }

    #[test]
    fn cursor_permutation_accept() {
        use Shape::*;
        let pattern = Pattern::try_from(vec![
            PatternElement::Permutation(vec![T, T, I, O].try_into().unwrap(), 2),
            PatternElement::Permutation(vec![L, J].try_into().unwrap(), 2),
        ]).unwrap();

        {
            for shapes in [[T, T, L, J], [I, O, J, L]] {
                let mut cursor = pattern.new_matcher();

                for shape in shapes {
                    assert!(cursor.has_next());

                    let (accepted, next) = cursor.match_shape(shape);
                    assert!(accepted);

                    cursor = next;
                }

                assert!(!cursor.has_next());

                let (accepted, cursor) = cursor.match_shape(Z);
                assert!(accepted);
                assert!(!cursor.has_next());
            }
        }

        {
            let cursor = pattern.new_matcher();
            assert!(cursor.has_next());

            let (accepted, cursor) = cursor.match_shape(T);
            assert!(accepted);
            assert!(cursor.has_next());

            let (accepted, cursor) = cursor.match_shape(T);
            assert!(accepted);
            assert!(cursor.has_next());

            let (accepted, cursor) = cursor.match_shape(T);
            assert!(!accepted);
            assert!(!cursor.has_next());

            let (accepted, cursor) = cursor.match_shape(L);
            assert!(!accepted);
            assert!(!cursor.has_next());
        }

        {
            let cursor = pattern.new_matcher();
            assert!(cursor.has_next());

            let (accepted, cursor) = cursor.match_shape(I);
            assert!(accepted);
            assert!(cursor.has_next());

            let (accepted, cursor) = cursor.match_shape(I);
            assert!(!accepted);
            assert!(!cursor.has_next());

            let (accepted, cursor) = cursor.match_shape(T);
            assert!(!accepted);
            assert!(!cursor.has_next());
        }
    }

    #[test]
    fn cursor_factorial_accept() {
        use Shape::*;
        let pattern = Pattern::try_from(vec![
            PatternElement::Factorial(vec![T, T, O].try_into().unwrap()),
            PatternElement::Factorial(vec![L, J].try_into().unwrap()),
        ]).unwrap();

        {
            for shapes in [[T, T, O, J, L], [T, O, T, L, J], [O, T, T, L, J]] {
                let mut cursor = pattern.new_matcher();

                for shape in shapes {
                    assert!(cursor.has_next());

                    let (accepted, next) = cursor.match_shape(shape);
                    assert!(accepted);

                    cursor = next;
                }

                assert!(!cursor.has_next());

                let (accepted, cursor) = cursor.match_shape(Z);
                assert!(accepted);
                assert!(!cursor.has_next());
            }
        }

        {
            let cursor = pattern.new_matcher();
            assert!(cursor.has_next());

            let (accepted, cursor) = cursor.match_shape(T);
            assert!(accepted);
            assert!(cursor.has_next());

            let (accepted, cursor) = cursor.match_shape(T);
            assert!(accepted);
            assert!(cursor.has_next());

            let (accepted, cursor) = cursor.match_shape(T);
            assert!(!accepted);
            assert!(!cursor.has_next());

            let (accepted, cursor) = cursor.match_shape(L);
            assert!(!accepted);
            assert!(!cursor.has_next());
        }

        {
            let cursor = pattern.new_matcher();
            assert!(cursor.has_next());

            let (accepted, cursor) = cursor.match_shape(O);
            assert!(accepted);
            assert!(cursor.has_next());

            let (accepted, cursor) = cursor.match_shape(O);
            assert!(!accepted);
            assert!(!cursor.has_next());

            let (accepted, cursor) = cursor.match_shape(T);
            assert!(!accepted);
            assert!(!cursor.has_next());
        }
    }

    #[test]
    fn extend() {
        use Shape::*;
        let pattern = Pattern::try_from(vec![
            PatternElement::Fixed(vec![T, I].try_into().unwrap()),
            PatternElement::Factorial(vec![L, J].try_into().unwrap()),
            PatternElement::One(O),
        ]).unwrap();

        let expand = PatternHoldExpand::from(&pattern);

        for shapes in [[T, I, J, L, O], [I, L, J, O, T], [I, J, T, O, L]] {
            let mut matcher = expand.new_matcher();
            for shape in shapes {
                assert!(matcher.has_next());

                let (matched, next) = matcher.match_shape(shape);
                assert!(matched);

                matcher = next;
            }

            assert!(!matcher.has_next());

            let (matched, matcher) = matcher.match_shape(S);
            assert!(matched);
            assert!(!matcher.has_next());
        }
    }

    #[test]
    fn extend2() {
        use Shape::*;
        let pattern = vec![
            PatternElement::Factorial(vec![T, Z].try_into().unwrap()),
            PatternElement::Factorial(vec![T, S].try_into().unwrap()),
        ].try_into().unwrap();

        let expand = PatternHoldExpand::from(&pattern);

        for shapes in [[T, T, Z, S]] {
            let mut matcher = expand.new_matcher();
            for shape in shapes {
                assert!(matcher.has_next());

                let (matched, next) = matcher.match_shape(shape);
                assert!(matched);

                matcher = next;
            }

            assert!(!matcher.has_next());

            let (matched, matcher) = matcher.match_shape(O);
            assert!(matched);
            assert!(!matcher.has_next());
        }
    }
}
