use std::vec::IntoIter;

use bitris::prelude::*;

use crate::{BitShapes, ShapeOrder};
use crate::internal_macros::forward_impl_from;
use crate::internals::{FuzzyShape, FuzzyShapeOrder};

/// Represents a sequence of shapes.
/// "Sequence" means that it is not affected by the hold operation.
/// Thus, it indicates that the branch is consumed from the head without being present.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default, Debug)]
pub struct ShapeSequence {
    shapes: Vec<Shape>,
}

impl ShapeSequence {
    #[inline]
    pub fn new(shapes: Vec<Shape>) -> Self {
        Self { shapes }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.shapes.len()
    }

    #[inline]
    pub fn shapes(&self) -> &[Shape] {
        self.shapes.as_slice()
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item=&Shape> {
        self.shapes.iter()
    }

    #[inline]
    pub fn to_shape_order(&self) -> ShapeOrder {
        ShapeOrder::new(self.shapes.clone())
    }

    /// If `self` is the resulting sequence of shapes, infer the order that could be the input.
    /// Let `infer_size` be the length of the order you wish to infer.
    /// If panics, `infer_size < sequence_length`.
    #[allow(dead_code)]
    fn infer_input(&self, infer_size: usize) -> Vec<FuzzyShapeOrder> {
        assert!(self.shapes.len() <= infer_size);

        if self.shapes.is_empty() {
            return vec![FuzzyShapeOrder::new(
                vec![FuzzyShape::Unknown; infer_size],
            )];
        }

        let mut orders = Vec::<FuzzyShapeOrder>::new();
        self.infer_input_walk(infer_size, &mut |fuzzy_shapes| {
            orders.push(FuzzyShapeOrder::new(fuzzy_shapes.to_vec()));
        });
        orders
    }

    /// See `infer_input()` for details.
    pub(crate) fn infer_input_walk(&self, infer_size: usize, visitor: &mut impl FnMut(&[FuzzyShape])) {
        fn rec(shapes: &Vec<Shape>, visitor: &mut impl FnMut(&[FuzzyShape]), buffer: &mut Vec<FuzzyShape>, from: usize, depth: usize, stock_index: usize) {
            use FuzzyShape::*;

            let to = shapes.len();
            let number = if depth < to {
                Some(depth)
            } else {
                None
            };

            if depth < from - 1 {
                // add
                buffer[depth + 1] = number.map_or(Unknown, |it| Known(shapes[it]));
                rec(shapes, visitor, buffer, from, depth + 1, stock_index);
                buffer[depth + 1] = Unknown;
            }

            {
                // stock
                buffer[stock_index] = number.map_or(Unknown, |it| Known(shapes[it]));
                if depth < from - 1 {
                    rec(shapes, visitor, buffer, from, depth + 1, depth + 1);
                } else {
                    visitor(buffer.as_slice());
                }
                buffer[stock_index] = Unknown;
            }
        }

        let mut buffer = vec![FuzzyShape::Unknown; infer_size];
        rec(&self.shapes, visitor, &mut buffer, infer_size, 0, 0);
    }
}

impl From<&BitShapes> for ShapeSequence {
    fn from(bit_shapes: &BitShapes) -> Self {
        Self::new(bit_shapes.to_vec())
    }
}

impl From<Vec<Shape>> for ShapeSequence {
    fn from(shapes: Vec<Shape>) -> Self {
        Self::new(shapes)
    }
}

impl From<&[Shape]> for ShapeSequence {
    fn from(shapes: &[Shape]) -> Self {
        Self::new(shapes.iter().map(|&shape| shape).collect())
    }
}

impl FromIterator<Shape> for ShapeSequence {
    fn from_iter<T: IntoIterator<Item=Shape>>(iter: T) -> Self {
        Self::new(iter.into_iter().collect())
    }
}

impl IntoIterator for ShapeSequence {
    type Item = Shape;
    type IntoIter = IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.shapes.into_iter()
    }
}

forward_impl_from!(ShapeSequence, from BitShapes);


#[cfg(test)]
mod tests {
    use bitris::prelude::*;

    use crate::internals::{FuzzyShape, FuzzyShapeOrder};
    use crate::ShapeSequence;

    #[test]
    fn infer_input() {
        use Shape::*;
        use FuzzyShape::*;

        let shape_sequence = ShapeSequence::new(vec![T, S]);

        let orders = shape_sequence.infer_input(2);
        assert_eq!(orders, vec![
            FuzzyShapeOrder::new(vec![Known(S), Known(T)]),
            FuzzyShapeOrder::new(vec![Known(T), Known(S)]),
        ]);

        let orders = shape_sequence.infer_input(3);
        assert_eq!(orders, vec![
            FuzzyShapeOrder::new(vec![Unknown, Known(T), Known(S)]),
            FuzzyShapeOrder::new(vec![Known(S), Known(T), Unknown]),
            FuzzyShapeOrder::new(vec![Known(T), Unknown, Known(S)]),
            FuzzyShapeOrder::new(vec![Known(T), Known(S), Unknown]),
        ]);

        let orders = shape_sequence.infer_input(4);
        assert_eq!(orders.len(), 8);
    }

    #[test]
    fn infer_input_from_empty() {
        use FuzzyShape::*;

        let shape_sequence = ShapeSequence::new(vec![]);

        let orders = shape_sequence.infer_input(0);
        assert_eq!(orders, vec![
            FuzzyShapeOrder::new(vec![]),
        ]);

        let orders = shape_sequence.infer_input(1);
        assert_eq!(orders, vec![
            FuzzyShapeOrder::new(vec![Unknown]),
        ]);

        let orders = shape_sequence.infer_input(2);
        assert_eq!(orders, vec![
            FuzzyShapeOrder::new(vec![Unknown, Unknown]),
        ]);
    }

    #[test]
    #[should_panic]
    fn infer_input_failed_to_assertion() {
        let shape_sequence = ShapeSequence::new(vec![Shape::T, Shape::S]);
        shape_sequence.infer_input(1);
    }
}
