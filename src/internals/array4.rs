use std::ops::Index;
use std::slice::Iter;

use thiserror::Error;

pub(crate) type DynArray4<T> = DynArrayN<T, 4>;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub(crate) struct DynArrayN<T, const N: usize> {
    items: [T; N],
    len: u8,
}

impl<T: Copy> DynArrayN<T, 4> {
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

    #[inline]
    #[cfg(test)]
    fn four(e1: T, e2: T, e3: T, e4: T) -> Self {
        Self { items: [e1, e2, e3, e4], len: 4 }
    }
}

impl<T, const N: usize> DynArrayN<T, N> {
    #[inline]
    fn get(&self, index: usize) -> Option<&T> {
        if index < self.len() {
            Some(&self.items[index])
        } else {
            None
        }
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


// A collection of errors that occur when making Array4.
#[derive(Error, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub(crate) enum ArrayNCreationError {
    #[error("The length is greater than N.")]
    CapacityOver,
}

impl<T: Default + Copy, const N: usize> TryFrom<Vec<T>> for DynArrayN<T, N> {
    type Error = ArrayNCreationError;

    fn try_from(value: Vec<T>) -> Result<Self, Self::Error> {
        use ArrayNCreationError::*;

        let len = value.len();
        if N < len {
            return Err(CapacityOver);
        }

        let mut items: [T; N] = [T::default(); N];
        for index in 0..len {
            items[index] = value[index];
        }
        Ok(Self { items, len: len as u8 })
    }
}

impl<T, const N: usize> Index<usize> for DynArrayN<T, N> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        assert!(index < self.len(), "the len is {} but the index is {}", self.len(), index);
        &self.items[index]
    }
}


#[cfg(test)]
mod tests {
    use crate::internals::{DynArray4, ArrayNCreationError};

    #[test]
    fn try_from() {
        use ArrayNCreationError::*;
        assert_eq!(DynArray4::try_from(vec![1]).unwrap().as_slice(), &[1]);
        assert_eq!(DynArray4::try_from(vec![1, 2]).unwrap().as_slice(), &[1, 2]);
        assert_eq!(DynArray4::try_from(vec![1, 2, 3]).unwrap().as_slice(), &[1, 2, 3]);
        assert_eq!(DynArray4::try_from(vec![1, 2, 3, 4]).unwrap().as_slice(), &[1, 2, 3, 4]);

        assert_eq!(DynArray4::try_from(vec![1, 2, 3, 4, 5]).unwrap_err(), CapacityOver);
    }

    #[test]
    fn one() {
        let array = DynArray4::one(1);
        assert_eq!(array.len(), 1);
        assert_eq!(array.as_slice(), &[1]);
        assert_eq!(array[0], 1);
        assert_eq!(array.get(0), Some(&1));
        assert_eq!(array.get(1), None);
    }

    #[test]
    fn two() {
        let array = DynArray4::two(1, 2);
        assert_eq!(array.len(), 2);
        assert_eq!(array.as_slice(), &[1, 2]);
        assert_eq!(array[0], 1);
        assert_eq!(array[1], 2);
        assert_eq!(array.get(1), Some(&2));
        assert_eq!(array.get(2), None);
    }

    #[test]
    fn three() {
        let array = DynArray4::three(1, 2, 3);
        assert_eq!(array.len(), 3);
        assert_eq!(array.as_slice(), &[1, 2, 3]);
        assert_eq!(array[0], 1);
        assert_eq!(array[1], 2);
        assert_eq!(array[2], 3);
        assert_eq!(array.get(2), Some(&3));
        assert_eq!(array.get(3), None);
    }

    #[test]
    fn four() {
        let array = DynArray4::four(1, 2, 3, 4);
        assert_eq!(array.len(), 4);
        assert_eq!(array.as_slice(), &[1, 2, 3, 4]);
        assert_eq!(array[0], 1);
        assert_eq!(array[1], 2);
        assert_eq!(array[2], 3);
        assert_eq!(array[3], 4);
        assert_eq!(array.get(3), Some(&4));
        assert_eq!(array.get(4), None);
    }

    #[test]
    fn iter() {
        let array = DynArray4::three(1, 2, 3);
        let mut iter = array.iter();
        assert_eq!(iter.next(), Some(&1));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), None);
    }
}
