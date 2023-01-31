use bitris::prelude::*;

use crate::BitShapes;

/// Represents an order of shapes.
/// "Order" means affected by the hold operation.
/// Thus, it allows branches to be produced, indicating that they are not necessarily consumed from the head.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default, Debug)]
pub struct ShapeOrder {
    shapes: Vec<Shape>,
}

impl ShapeOrder {
    #[inline]
    pub fn new(shapes: Vec<Shape>) -> Self {
        Self { shapes }
    }

    #[inline]
    pub fn new_cursor(&self) -> OrderCursor<Shape> {
        (&self.shapes).into()
    }

    #[inline]
    pub fn shapes(&self) -> &[Shape] {
        self.shapes.as_slice()
    }
}

impl From<&BitShapes> for ShapeOrder {
    fn from(bit_shapes: &BitShapes) -> Self {
        Self::new(bit_shapes.to_vec())
    }
}

impl From<Vec<Shape>> for ShapeOrder {
    fn from(shapes: Vec<Shape>) -> Self {
        Self::new(shapes)
    }
}

impl From<&[Shape]> for ShapeOrder {
    fn from(shapes: &[Shape]) -> Self {
        Self::new(shapes.iter().map(|&shape| shape).collect())
    }
}

impl FromIterator<Shape> for ShapeOrder {
    fn from_iter<T: IntoIterator<Item=Shape>>(iter: T) -> Self {
        Self::new(iter.into_iter().collect())
    }
}
