use bitris::Shape;
use derive_more::Constructor;
use itertools::{Itertools, repeat_n};

use crate::{ForEachVisitor, ShapeCounter, ShapeOrder, ShapeSequence};
use crate::bit_shapes::BitShapes;

/// Calculate the number of permutations.
fn calculate_permutation_size(len: usize, pop: usize) -> usize {
    assert!(pop <= len);
    assert!(0 < pop);
    ((len - pop + 1)..=len).fold(1, |sum, it| sum * it)
}

/// A collection of elements to define the order/sequence of the shapes.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum PatternElement {
    /// A fixed shape (like `T`)
    One(Shape),

    /// A sequence fixed shapes (like `TIO`)
    Fixed(BitShapes),

    /// One from all shapes (like. `*`)
    Wildcard,

    /// Permutations by taking `usize` shapes from `ShapeCounter`. Duplicates are not removed.
    /// (like `[TIO]p2`)
    Permutation(ShapeCounter, usize),

    /// Permutations by taking all shapes from `ShapeCounter`. Duplicates are not removed.
    /// (like `[TIOLJSZ]p7`, `*!`)
    Factorial(ShapeCounter),
}

impl PatternElement {
    /// Returns all `Vec<Shape>`s represented by the pattern.
    pub fn to_shapes_vec(&self) -> Vec<Vec<Shape>> {
        match *self {
            PatternElement::One(shape) => vec![vec![shape]],
            PatternElement::Fixed(shapes) => vec![shapes.to_vec()],
            PatternElement::Wildcard => Shape::all_into_iter().map(|it| vec![it]).collect(),
            PatternElement::Permutation(counter, pop) => {
                assert!(0 < pop && pop <= counter.len());
                counter.to_pairs().into_iter()
                    .flat_map(|(shape, count)| { repeat_n(shape, count as usize).into_iter() })
                    .permutations(pop)
                    .collect_vec()
            }
            PatternElement::Factorial(counter) => {
                counter.to_pairs().into_iter()
                    .flat_map(|(shape, count)| { repeat_n(shape, count as usize).into_iter() })
                    .permutations(counter.len())
                    .collect_vec()
            }
        }
    }

    /// The count of shapes the pattern has.
    pub fn len_shapes_vec(&self) -> usize {
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
            PatternElement::Factorial(counter) => counter.len(),
        }
    }
}

/// Define the order/sequence of the shapes.
/// ```
/// use bitris_commands::prelude::*;
/// use PatternElement::*;
///
/// let patterns = Pattern::new(vec![One(Shape::T), Wildcard, Wildcard]);
/// assert_eq!(patterns.len_shapes_vec(), 49);
/// assert_eq!(patterns.dim_shapes(), 3);
///
/// let patterns = Pattern::new(vec![Fixed(BitShapes::try_from(vec![Shape::T, Shape::I]).unwrap())]);
/// assert_eq!(patterns.len_shapes_vec(), 1);
/// assert_eq!(patterns.dim_shapes(), 2);
///
/// let patterns = Pattern::new(vec![Permutation(ShapeCounter::one_of_each(), 3)]);
/// assert_eq!(patterns.len_shapes_vec(), 210);
/// assert_eq!(patterns.dim_shapes(), 3);
/// ```
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Constructor)]
pub struct Pattern {
    elements: Vec<PatternElement>,
}

impl Pattern {
    #[allow(dead_code)]
    fn walk_shapes(&self, visitor: &mut impl ForEachVisitor<Vec<Shape>>) {
        let all_shapes_vec: Vec<Vec<Vec<Shape>>> = self.elements.clone()
            .into_iter()
            .map(|it| it.to_shapes_vec())
            .collect();

        let mut buffer: Vec<Shape> = Vec::with_capacity(self.dim_shapes());

        fn build(
            all_shapes_vec: &Vec<Vec<Vec<Shape>>>,
            index: usize,
            buffer: &mut Vec<Shape>,
            visitor: &mut impl ForEachVisitor<Vec<Shape>>,
        ) {
            if index < all_shapes_vec.len() - 1 {
                for shapes in &all_shapes_vec[index] {
                    let size = buffer.len();
                    buffer.extend(shapes.iter());
                    build(all_shapes_vec, index + 1, buffer, visitor);
                    buffer.resize(size, Shape::T);
                }
            } else {
                for shapes in &all_shapes_vec[index] {
                    let size = buffer.len();
                    buffer.extend(shapes.iter());
                    visitor.visit(buffer);
                    buffer.resize(size, Shape::T);
                }
            }
        }

        build(&all_shapes_vec, 0, &mut buffer, visitor);
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
            .map(|it| it.len_shapes_vec())
            .fold(1, |sum, it| sum * it)
    }

    /// The number of elements in one shapes.
    pub fn dim_shapes(&self) -> usize {
        if self.elements.is_empty() {
            return 0;
        }
        self.elements.iter()
            .map(|it| it.dim_shapes())
            .fold(0, |sum, it| sum + it)
    }
}


#[cfg(test)]
mod tests {
    use bitris::Shape;

    use crate::{Pattern, PatternElement, ShapeCounter};
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
        assert_eq!(pattern.len_shapes_vec(), 1);

        let counter = ShapeCounter::from(vec![Shape::I, Shape::O, Shape::T]);
        let pattern = PatternElement::Permutation(counter, 1);
        assert_eq!(pattern.dim_shapes(), 1);
        assert_eq!(pattern.len_shapes_vec(), 3);

        let counter = ShapeCounter::from(vec![Shape::I, Shape::O, Shape::T]);
        let pattern = PatternElement::Permutation(counter, 2);
        assert_eq!(pattern.dim_shapes(), 2);
        assert_eq!(pattern.len_shapes_vec(), 6);

        let counter = ShapeCounter::one_of_each();
        let pattern = PatternElement::Permutation(counter, 3);
        assert_eq!(pattern.dim_shapes(), 3);
        assert_eq!(pattern.len_shapes_vec(), 210);

        let counter = ShapeCounter::one_of_each();
        let pattern = PatternElement::Permutation(counter, 5);
        assert_eq!(pattern.dim_shapes(), 5);
        assert_eq!(pattern.len_shapes_vec(), 2520);
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
        let patterns = Pattern::new(vec![]);
        assert_eq!(patterns.len_shapes_vec(), 0);
        assert_eq!(patterns.dim_shapes(), 0);
        assert_eq!(patterns.to_sequences().len(), 0);
    }

    #[test]
    fn test() {
        let patterns = Pattern::new(vec![
            PatternElement::Permutation(ShapeCounter::one_of_each(), 6),
            PatternElement::Permutation(ShapeCounter::one_of_each(), 3),
        ]);
        assert_eq!(patterns.len_shapes_vec(), 5040 * 210);
        assert_eq!(patterns.dim_shapes(), 9);
        assert_eq!(patterns.to_sequences().len(), 5040 * 210);
    }
}
