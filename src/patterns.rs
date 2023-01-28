use bitris::prelude::Shape;
use itertools::{Itertools, repeat_n};
use thiserror::Error;

use crate::{ForEachVisitor, ShapeCounter, ShapeOrder, ShapeSequence};
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
        match *self {
            PatternElement::One(shape) => vec![vec![shape]],
            PatternElement::Fixed(shapes) => vec![shapes.to_vec()],
            PatternElement::Wildcard => Shape::all_iter().map(|it| vec![it]).collect(),
            PatternElement::Permutation(counter, pop) => {
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
    pub fn len_shapes(&self) -> usize {
        match *self {
            PatternElement::One(_) => 1,
            PatternElement::Fixed(_) => 1,
            PatternElement::Wildcard => 7,
            PatternElement::Permutation(counter, pop) => {
                assert!(0 < pop && pop <= counter.len());
                calculate_permutation_size(counter.len(), pop)
            }
            PatternElement::Factorial(counter) => calculate_permutation_size(counter.len(), counter.len()),
        }
    }

    /// The number of elements in one shapes.
    pub fn dim_shapes(&self) -> usize {
        match *self {
            PatternElement::One(_) => 1,
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
    fn walk_shapes(&self, visitor: &mut impl ForEachVisitor<Vec<Shape>>) {
        struct Buffer<'a> {
            all_vec: &'a Vec<Vec<Vec<Shape>>>,
            buffer: Vec<Shape>,
        }

        impl Buffer<'_> {
            fn build(&mut self, index: usize, visitor: &mut impl ForEachVisitor<Vec<Shape>>) {
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
                        visitor.visit(&self.buffer);
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

        struct Aggregator {
            out: Vec<Vec<Shape>>,
        }

        impl ForEachVisitor<Vec<Shape>> for Aggregator {
            fn visit(&mut self, shapes: &Vec<Shape>) {
                self.out.push(shapes.clone());
            }
        }

        let capacity = self.len_shapes_vec();
        let mut visitor = Aggregator { out: Vec::with_capacity(capacity) };

        self.walk_shapes(&mut visitor);

        visitor.out
    }

    /// Returns all sequences represented by the patterns.
    pub fn to_sequences(&self) -> Vec<ShapeSequence> {
        self.to_shapes_vec().into_iter()
            .map(|it| ShapeSequence::new(it))
            .collect()
    }

    /// Returns all orders represented by the patterns.
    pub fn to_orders(&self) -> Vec<ShapeOrder> {
        self.to_shapes_vec().into_iter()
            .map(|it| ShapeOrder::new(it))
            .collect()
    }

    /// The count of shapes the patterns has.
    pub fn len_shapes_vec(&self) -> usize {
        if self.elements.is_empty() {
            return 0;
        }
        self.elements.iter()
            .map(|it| it.len_shapes())
            .fold(1, |sum, it| sum * it)
    }

    /// The number of elements in one shapes.
    pub fn dim_shapes(&self) -> usize {
        assert!(!self.elements.is_empty(), "The pattern do not have shapes.");
        self.elements.iter()
            .map(|it| it.dim_shapes())
            .fold(0, |sum, it| sum + it)
    }

    #[allow(dead_code)]
    fn walk_shape_counters(&self, visitor: &mut impl ForEachVisitor<ShapeCounter>) {
        struct Buffer<'a> {
            all_vec: &'a Vec<Vec<ShapeCounter>>,
        }

        impl Buffer<'_> {
            fn build(&mut self, index: usize, buffer: ShapeCounter, visitor: &mut impl ForEachVisitor<ShapeCounter>) {
                if index < self.all_vec.len() - 1 {
                    for shapes in &self.all_vec[index] {
                        self.build(index + 1, buffer + shapes, visitor);
                    }
                } else {
                    for shapes in &self.all_vec[index] {
                        visitor.visit(&(buffer + shapes));
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

        struct Aggregator {
            out: Vec<ShapeCounter>,
        }

        impl ForEachVisitor<ShapeCounter> for Aggregator {
            fn visit(&mut self, shape_counters: &ShapeCounter) {
                self.out.push(shape_counters.clone());
            }
        }

        let mut visitor = Aggregator { out: Vec::new() };

        self.walk_shape_counters(&mut visitor);

        visitor.out
    }
}


#[cfg(test)]
mod tests {
    use bitris::prelude::Shape;

    use crate::{Pattern, PatternCreationError, PatternElement, ShapeCounter};
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
        assert_eq!(pattern.len_shapes(), 1);

        let counter = ShapeCounter::from(vec![Shape::I, Shape::O, Shape::T]);
        let pattern = PatternElement::Permutation(counter, 1);
        assert_eq!(pattern.dim_shapes(), 1);
        assert_eq!(pattern.len_shapes(), 3);

        let counter = ShapeCounter::from(vec![Shape::I, Shape::O, Shape::T]);
        let pattern = PatternElement::Permutation(counter, 2);
        assert_eq!(pattern.dim_shapes(), 2);
        assert_eq!(pattern.len_shapes(), 6);

        let counter = ShapeCounter::one_of_each();
        let pattern = PatternElement::Permutation(counter, 3);
        assert_eq!(pattern.dim_shapes(), 3);
        assert_eq!(pattern.len_shapes(), 210);

        let counter = ShapeCounter::one_of_each();
        let pattern = PatternElement::Permutation(counter, 5);
        assert_eq!(pattern.dim_shapes(), 5);
        assert_eq!(pattern.len_shapes(), 2520);
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
        let patterns = Pattern::try_from(vec![
            PatternElement::Permutation(ShapeCounter::one_of_each(), 6),
            PatternElement::Permutation(ShapeCounter::one_of_each(), 3),
        ]).unwrap();
        assert_eq!(patterns.len_shapes_vec(), 5040 * 210);
        assert_eq!(patterns.dim_shapes(), 9);
        assert_eq!(patterns.to_sequences().len(), 5040 * 210);
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
}
