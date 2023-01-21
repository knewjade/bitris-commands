use std::ops::Index;
use std::slice::Iter;

use thiserror::Error;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default, Debug)]
pub(crate) struct Array4<T> {
    items: [T; 4],
    len: u8,
}

impl<T: Copy> Array4<T> {
    #[inline]
    #[cfg(test)]
    fn one(e1: T) -> Self {
        Self { items: [e1; 4], len: 1 }
    }

    #[inline]
    #[cfg(test)]
    fn two(e1: T, e2: T) -> Self {
        Self { items: [e1, e2, e2, e2], len: 2 }
    }

    #[inline]
    #[cfg(test)]
    fn three(e1: T, e2: T, e3: T) -> Self {
        Self { items: [e1, e2, e3, e3], len: 3 }
    }
}

impl<T> Array4<T> {
    #[inline]
    #[cfg(test)]
    fn four(e1: T, e2: T, e3: T, e4: T) -> Self {
        Self { items: [e1, e2, e3, e4], len: 4 }
    }

    #[inline]
    fn as_slice(&self) -> &[T] {
        &self.items[0..self.len()]
    }

    #[inline]
    fn len(&self) -> usize {
        self.len as usize
    }

    #[inline]
    pub(crate) fn iter(&self) -> Iter<'_, T> {
        self.as_slice().iter()
    }
}


// TODO A collection of errors that occur when making clipped board.
#[derive(Error, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub(crate) enum Array4CreationError {
    #[error("TODO")]
    CapacityOver,
}

impl<T: Copy + Default> TryFrom<Vec<T>> for Array4<T> {
    type Error = Array4CreationError;

    fn try_from(value: Vec<T>) -> Result<Self, Self::Error> {
        use Array4CreationError::*;

        let len = value.len();
        if 4 < len {
            return Err(CapacityOver);
        }

        let items: [T; 4] = [
            *value.get(0).unwrap_or(&T::default()),
            *value.get(1).unwrap_or(&T::default()),
            *value.get(2).unwrap_or(&T::default()),
            *value.get(3).unwrap_or(&T::default()),
        ];
        Ok(Self { items, len: len as u8 })
    }
}

impl<T> Index<usize> for Array4<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        assert!(index < self.len(), "the len is {} but the index is {}", self.len(), index);
        &self.items[index]
    }
}


#[cfg(test)]
mod tests {
    use crate::internals::{Array4, Array4CreationError};

    #[test]
    fn try_from() {
        use Array4CreationError::*;
        assert_eq!(Array4::try_from(vec![1]).unwrap().as_slice(), &[1]);
        assert_eq!(Array4::try_from(vec![1, 2]).unwrap().as_slice(), &[1, 2]);
        assert_eq!(Array4::try_from(vec![1, 2, 3]).unwrap().as_slice(), &[1, 2, 3]);
        assert_eq!(Array4::try_from(vec![1, 2, 3, 4]).unwrap().as_slice(), &[1, 2, 3, 4]);

        assert_eq!(Array4::try_from(vec![1, 2, 3, 4, 5]).unwrap_err(), CapacityOver);
    }

    #[test]
    fn one() {
        let array = Array4::one(1);
        assert_eq!(array.len(), 1);
        assert_eq!(array.as_slice(), &[1]);
        assert_eq!(array[0], 1);
    }

    #[test]
    fn two() {
        let array = Array4::two(1, 2);
        assert_eq!(array.len(), 2);
        assert_eq!(array.as_slice(), &[1, 2]);
        assert_eq!(array[0], 1);
        assert_eq!(array[1], 2);
    }

    #[test]
    fn three() {
        let array = Array4::three(1, 2, 3);
        assert_eq!(array.len(), 3);
        assert_eq!(array.as_slice(), &[1, 2, 3]);
        assert_eq!(array[0], 1);
        assert_eq!(array[1], 2);
        assert_eq!(array[2], 3);
    }

    #[test]
    fn four() {
        let array = Array4::four(1, 2, 3, 4);
        assert_eq!(array.len(), 4);
        assert_eq!(array.as_slice(), &[1, 2, 3, 4]);
        assert_eq!(array[0], 1);
        assert_eq!(array[1], 2);
        assert_eq!(array[2], 3);
        assert_eq!(array[3], 4);
    }

    #[test]
    fn iter() {
        let array = Array4::three(1, 2, 3);
        let mut iter = array.iter();
        assert_eq!(iter.next(), Some(&1));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), None);
    }
}
