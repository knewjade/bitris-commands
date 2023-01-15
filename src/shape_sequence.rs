use bitris::Shape;

use crate::{BitShapes, ForEachVisitor, ShapeOrder};
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
    pub fn shapes(&self) -> &[Shape] {
        self.shapes.as_slice()
    }

    #[inline]
    pub fn to_shape_order(&self) -> ShapeOrder {
        ShapeOrder::new(self.shapes.clone())
    }
}

impl ShapeSequence {
    /// If `self` is the resulting sequence of shapes, infer the order that could be the input.
    /// Let `infer_size` be the length of the order you wish to infer.
    /// If panics, `infer_size < sequence_length`.
    #[allow(dead_code)]
    fn infer_input(&self, infer_size: usize) -> Vec<FuzzyShapeOrder> {
        assert!(self.shapes.len() <= infer_size);

        if self.shapes.is_empty() {
            let mut shapes = Vec::<FuzzyShape>::with_capacity(infer_size);
            shapes.resize(infer_size, FuzzyShape::Unknown);
            return vec![FuzzyShapeOrder::new(shapes)];
        }

        pub struct FuzzyOrderVecAggregator {
            orders: Vec<FuzzyShapeOrder>,
        }

        impl ForEachVisitor<[FuzzyShape]> for FuzzyOrderVecAggregator {
            fn visit(&mut self, fuzzy_shapes: &[FuzzyShape]) {
                self.orders.push(FuzzyShapeOrder::new(fuzzy_shapes.to_vec()));
            }
        }

        let mut visitor = FuzzyOrderVecAggregator { orders: Vec::new() };
        self.infer_input_walk(infer_size, &mut visitor);
        visitor.orders
    }

    /// See `infer_input()` for details.
    pub(crate) fn infer_input_walk(&self, infer_size: usize, visitor: &mut impl ForEachVisitor<[FuzzyShape]>) {
        fn rec<'a>(shapes: &Vec<Shape>, visitor: &mut impl ForEachVisitor<[FuzzyShape]>, buffer: &'a mut Vec<FuzzyShape>, from: usize, depth: usize, stock_index: usize) {
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
                    visitor.visit(buffer.as_slice());
                }
                buffer[stock_index] = Unknown;
            }
        }

        let mut buffer = Vec::<FuzzyShape>::with_capacity(infer_size);
        buffer.resize(infer_size, FuzzyShape::Unknown);

        rec(&self.shapes, visitor, &mut buffer, infer_size, 0, 0);
    }
}

impl From<&BitShapes> for ShapeSequence {
    fn from(bit_shapes: &BitShapes) -> Self {
        Self { shapes: bit_shapes.to_vec() }
    }
}

forward_impl_from!(ShapeSequence, from BitShapes);


#[cfg(test)]
mod tests {
    use bitris::*;

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
