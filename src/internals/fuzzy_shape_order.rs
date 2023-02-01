use bitris::prelude::Shape;

use crate::internals::fuzzy_shape::FuzzyShape;
use crate::ShapeOrder;

/// Represents an order of shapes that includes fuzzy.
/// "Order" means affected by the hold operation.
/// Thus, it allows branches to be produced, indicating that they are not necessarily consumed from the head.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default, Debug)]
pub(crate) struct FuzzyShapeOrder {
    shapes: Vec<FuzzyShape>,
}

impl FuzzyShapeOrder {
    #[inline]
    pub fn new(shapes: Vec<FuzzyShape>) -> Self {
        Self { shapes }
    }

    /// Expand unknown shapes to the order assumed as the shape of each.
    #[allow(dead_code)]
    fn expand_as_wildcard(&self) -> Vec<ShapeOrder> {
        let mut out = Vec::<ShapeOrder>::new();
        self.expand_as_wildcard_walk(&mut |shapes| {
            out.push(ShapeOrder::new(shapes.to_vec()));
        });
        out
    }

    /// See `expand_as_wildcard()` for details.
    pub(crate) fn expand_as_wildcard_walk(&self, visitor: &mut impl FnMut(&[Shape])) {
        fn build(shapes: &Vec<FuzzyShape>, buffer: &mut Vec<Shape>, visitor: &mut impl FnMut(&[Shape])) {
            let index = buffer.len();
            if shapes.len() <= index {
                let x: &[Shape] = buffer.as_slice();
                visitor(x);
                return;
            }

            match shapes[index] {
                FuzzyShape::Known(shape) => {
                    buffer.push(shape);
                    build(shapes, buffer, visitor);
                    buffer.pop();
                }
                FuzzyShape::Unknown => {
                    for shape in Shape::all_iter() {
                        buffer.push(shape);
                        build(shapes, buffer, visitor);
                        buffer.pop();
                    }
                }
            }
        }

        assert!(!self.shapes.is_empty());
        let mut buffer = Vec::<Shape>::with_capacity(self.shapes.len());
        build(&self.shapes, &mut buffer, visitor);
    }
}

impl From<Vec<FuzzyShape>> for FuzzyShapeOrder {
    fn from(fuzzy_shapes: Vec<FuzzyShape>) -> Self {
        Self::new(fuzzy_shapes)
    }
}

impl From<&[FuzzyShape]> for FuzzyShapeOrder {
    fn from(fuzzy_shapes: &[FuzzyShape]) -> Self {
        Self::new(fuzzy_shapes.iter().map(|&fuzzy_shape| fuzzy_shape).collect())
    }
}

impl FromIterator<FuzzyShape> for FuzzyShapeOrder {
    fn from_iter<T: IntoIterator<Item=FuzzyShape>>(iter: T) -> Self {
        Self::new(iter.into_iter().collect())
    }
}


#[cfg(test)]
mod tests {
    use bitris::prelude::*;

    use crate::internals::{FuzzyShape, FuzzyShapeOrder};
    use crate::ShapeOrder;

    #[test]
    fn fuzzy() {
        use Shape::*;
        use FuzzyShape::*;
        let fuzzy_shape_order = FuzzyShapeOrder::new(vec![Known(T), Unknown, Known(O)]);
        let orders = fuzzy_shape_order.expand_as_wildcard();
        assert_eq!(orders, vec![
            ShapeOrder::new(vec![T, T, O]),
            ShapeOrder::new(vec![T, I, O]),
            ShapeOrder::new(vec![T, O, O]),
            ShapeOrder::new(vec![T, L, O]),
            ShapeOrder::new(vec![T, J, O]),
            ShapeOrder::new(vec![T, S, O]),
            ShapeOrder::new(vec![T, Z, O]),
        ]);
    }
}
