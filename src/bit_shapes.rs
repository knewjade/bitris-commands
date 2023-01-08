use bitris::Shape;
use derive_more::Constructor;

use crate::internal_macros::forward_impl_try_from;
use crate::ShapeSequence;

/// Preserves the order of shapes.
/// Internally, it's represented in bits, making copying and comparing lightweight.
///
/// Instead, the maximum number of shapes is limited to 22.
/// ```
/// use bitris_commands::prelude::*;
///
/// let shapes = BitShapes::try_from(vec![Shape::T, Shape::I, Shape::O]).unwrap();
/// assert_eq!(shapes.len(), 3);
/// assert_eq!(shapes.to_vec(), vec![Shape::T, Shape::I, Shape::O]);
/// ```
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default, Debug, Constructor)]
pub struct BitShapes {
    value: u64,
    len: u8,
}

impl BitShapes {
    /// ```
    /// use bitris_commands::BitShapes;
    /// let shapes = BitShapes::empty();
    /// assert_eq!(shapes.len(), 0);
    /// assert_eq!(shapes.to_vec(), Vec::new());
    /// ```
    #[inline]
    pub fn empty() -> Self {
        BitShapes::new(0, 0)
    }

    #[inline]
    pub fn len(self) -> usize {
        self.len as usize
    }

    /// ```
    /// use bitris_commands::prelude::*;
    ///
    /// let shapes = BitShapes::try_from(vec![Shape::T, Shape::I, Shape::O]).unwrap();
    /// assert_eq!(shapes.to_vec(), vec![Shape::T, Shape::I, Shape::O]);
    ///
    /// ```
    pub fn to_vec(self) -> Vec<Shape> {
        let mut value = self.value;
        let len = self.len();
        let mut vec = Vec::with_capacity(len);
        for _ in 0..len {
            let shape_value = value % 7;
            value /= 7;
            vec.push(Shape::try_from(shape_value as usize).ok().unwrap());
        }
        vec
    }
}

// A collection of errors that occur when converting to the shape.
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum BitShapesTryFromError {
    TooManyShapes(usize),
}

impl TryFrom<&[Shape]> for BitShapes {
    type Error = BitShapesTryFromError;

    fn try_from(shapes: &[Shape]) -> Result<Self, Self::Error> {
        if 23 <= shapes.len() {
            return Err(BitShapesTryFromError::TooManyShapes(shapes.len()));
        }

        let mut value = 0u64;
        let mut scale = 1u64;
        for shape in shapes {
            value += *shape as u64 * scale;
            scale *= 7;
        }
        Ok(BitShapes::new(value, shapes.len() as u8))
    }
}

impl TryFrom<&Vec<Shape>> for BitShapes {
    type Error = BitShapesTryFromError;

    /// ```
    /// use bitris_commands::prelude::*;
    ///
    /// let shapes = BitShapes::try_from(vec![Shape::I, Shape::O, Shape::T]).unwrap();
    /// assert_eq!(shapes.len(), 3);
    /// assert_eq!(shapes.to_vec(), vec![Shape::I, Shape::O, Shape::T]);
    ///
    /// let result = BitShapes::try_from(vec![Shape::T].repeat(23));
    /// assert_eq!(result, Err(BitShapesTryFromError::TooManyShapes(23)));
    /// ```
    fn try_from(shapes: &Vec<Shape>) -> Result<Self, Self::Error> {
        BitShapes::try_from(shapes.as_slice())
    }
}

forward_impl_try_from!(BitShapes, BitShapesTryFromError, from Vec<Shape>);

impl TryFrom<&ShapeSequence> for BitShapes {
    type Error = BitShapesTryFromError;

    fn try_from(order: &ShapeSequence) -> Result<Self, Self::Error> {
        BitShapes::try_from(order.shapes())
    }
}

forward_impl_try_from!(BitShapes, BitShapesTryFromError, from ShapeSequence);


#[cfg(test)]
mod tests {
    use bitris::Shape;
    use itertools::Itertools;

    use crate::BitShapes;

    #[test]
    fn len7() {
        let shapes = BitShapes::try_from(Shape::all_into_iter().collect_vec()).unwrap();
        assert_eq!(shapes.len(), 7);
    }

    #[test]
    fn len22() {
        let shapes = BitShapes::try_from(vec![Shape::T, Shape::I].repeat(11)).unwrap();
        assert_eq!(shapes.len(), 22);
        assert_eq!(shapes.to_vec(), vec![Shape::T, Shape::I].repeat(11));
    }
}
