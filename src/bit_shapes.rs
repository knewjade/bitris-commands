use bitris::prelude::*;
use derive_more::Constructor;
use thiserror::Error;

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
        self.into_iter().collect()
    }

    /// ```
    /// use bitris_commands::prelude::*;
    ///
    /// let shapes = BitShapes::try_from(vec![Shape::T, Shape::I]).unwrap();
    /// assert_eq!(shapes.get(0), Some(Shape::T));
    /// assert_eq!(shapes.get(1), Some(Shape::I));
    /// assert_eq!(shapes.get(2), None);
    /// ```
    pub fn get(self, index: usize) -> Option<Shape> {
        self.into_iter().skip(index).next()
    }
}

/// Iterator implementation for `BitShapes`.
pub struct BitShapesIterator {
    value: u64,
    len: usize,
}

impl Iterator for BitShapesIterator {
    type Item = Shape;

    fn next(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            return None;
        }

        let shape_value = self.value % 7;
        self.value /= 7;
        self.len -= 1;
        Some(Shape::try_from(shape_value as usize).unwrap())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl IntoIterator for BitShapes {
    type Item = Shape;
    type IntoIter = BitShapesIterator;

    fn into_iter(self) -> Self::IntoIter {
        BitShapesIterator { value: self.value, len: self.len as usize }
    }
}

/// A collection of errors that occur when making `BitShapes`.
#[derive(Error, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum BitShapesCreationError {
    #[error("The shapes are too long. It supports up to 22 shapes.")]
    TooManyShapes(usize),
}

impl TryFrom<&[Shape]> for BitShapes {
    type Error = BitShapesCreationError;

    fn try_from(shapes: &[Shape]) -> Result<Self, Self::Error> {
        if 23 <= shapes.len() {
            return Err(BitShapesCreationError::TooManyShapes(shapes.len()));
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
    type Error = BitShapesCreationError;

    /// ```
    /// use bitris_commands::prelude::*;
    ///
    /// let shapes = BitShapes::try_from(vec![Shape::I, Shape::O, Shape::T]).unwrap();
    /// assert_eq!(shapes.len(), 3);
    /// assert_eq!(shapes.to_vec(), vec![Shape::I, Shape::O, Shape::T]);
    ///
    /// let result = BitShapes::try_from(vec![Shape::T].repeat(23));
    /// assert_eq!(result, Err(BitShapesCreationError::TooManyShapes(23)));
    /// ```
    fn try_from(shapes: &Vec<Shape>) -> Result<Self, Self::Error> {
        BitShapes::try_from(shapes.as_slice())
    }
}

forward_impl_try_from!(BitShapes, BitShapesCreationError, from Vec<Shape>);

impl TryFrom<&ShapeSequence> for BitShapes {
    type Error = BitShapesCreationError;

    fn try_from(order: &ShapeSequence) -> Result<Self, Self::Error> {
        BitShapes::try_from(order.shapes())
    }
}

forward_impl_try_from!(BitShapes, BitShapesCreationError, from ShapeSequence);


#[cfg(test)]
mod tests {
    use bitris::prelude::Shape;
    use itertools::Itertools;

    use crate::BitShapes;

    #[test]
    fn empty() {
        let shapes = BitShapes::empty();
        assert_eq!(shapes.len(), 0);
    }

    #[test]
    fn len7() {
        let shapes = BitShapes::try_from(Shape::all_iter().collect_vec()).unwrap();
        assert_eq!(shapes.len(), 7);
    }

    #[test]
    fn len22() {
        let shapes = BitShapes::try_from(vec![Shape::T, Shape::I].repeat(11)).unwrap();
        assert_eq!(shapes.len(), 22);
        assert_eq!(shapes.to_vec(), vec![Shape::T, Shape::I].repeat(11));
    }
}
