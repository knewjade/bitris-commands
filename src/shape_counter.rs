use std::{cmp, ops};
use std::iter::zip;

use bitris::prelude::Shape;
use derive_more::Constructor;
use itertools::Itertools;

use crate::internal_macros::forward_ref_op;

/// Holds the count of each shape. Each shape can hold up to 255 items.
/// ```
/// use bitris_commands::prelude::*;
/// let counter = ShapeCounter::from(vec![Shape::T, Shape::T, Shape::I]);
/// assert_eq!(counter[Shape::T], 2);
/// assert_eq!(counter[Shape::I], 1);
/// assert_eq!(counter[Shape::O], 0);
/// ```
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default, Debug, Constructor)]
pub struct ShapeCounter {
    counters: [u8; 7],
}

impl ShapeCounter {
    /// ```
    /// use bitris_commands::prelude::*;
    /// let counter = ShapeCounter::empty();
    /// assert_eq!(counter.len(), 0);
    /// ```
    #[inline]
    pub fn empty() -> Self {
        ShapeCounter::new([0; 7])
    }

    /// ```
    /// use bitris_commands::prelude::*;
    /// use Shape::*;
    /// assert_eq!(ShapeCounter::one(T), ShapeCounter::from(vec![T]));
    /// ```
    #[inline]
    pub fn one(shape: Shape) -> Self {
        Self::single_shape(shape, 1)
    }

    /// ```
    /// use bitris_commands::prelude::*;
    /// use Shape::*;
    /// let counter = ShapeCounter::one_of_each();
    /// assert_eq!(counter.len(), 7);
    /// assert_eq!(counter, ShapeCounter::from(vec![T, I, O, L, J, S, Z]));
    /// ```
    #[inline]
    pub fn one_of_each() -> Self {
        ShapeCounter::new([1; 7])
    }

    /// ```
    /// use bitris_commands::prelude::*;
    /// use Shape::*;
    /// assert_eq!(ShapeCounter::single_shape(T, 3), ShapeCounter::from(vec![T, T, T]));
    /// ```
    #[inline]
    pub fn single_shape(shape: Shape, len: u8) -> Self {
        let mut counters = [0; 7];
        counters[shape as usize] = len;
        ShapeCounter::new(counters)
    }

    /// ```
    /// use bitris_commands::prelude::*;
    /// use Shape::*;
    /// let counter = ShapeCounter::max();
    /// assert_eq!(counter.len(), 255 * 7);
    /// ```
    #[inline]
    pub fn max() -> Self {
        ShapeCounter::new([u8::MAX; 7])
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.counters.into_iter()
            .map(|it| it as usize)
            .fold(0, |sum, it| sum + it)
    }

    /// Return the count of shape types contained.
    /// ```
    /// use bitris_commands::prelude::*;
    /// use Shape::*;
    /// assert_eq!(ShapeCounter::from(vec![]).count_shape_types(), 0);
    /// assert_eq!(ShapeCounter::from(vec![I, I]).count_shape_types(), 1);
    /// assert_eq!(ShapeCounter::from(vec![O, O, S]).count_shape_types(), 2);
    /// ```
    #[inline]
    pub fn count_shape_types(&self) -> usize {
        self.counters.into_iter()
            .filter(|&it| 0 < it)
            .count()
    }

    /// Returns a pair of each shape and its count.
    /// ```
    /// use bitris_commands::prelude::*;
    /// let counter = ShapeCounter::from(vec![Shape::O, Shape::O, Shape::S]);
    /// assert_eq!(counter.len(), 3);
    /// assert_eq!(counter.to_pairs(), vec![(Shape::O, 2), (Shape::S, 1)]);
    /// ```
    pub fn to_pairs(&self) -> Vec<(Shape, u8)> {
        let mut vec = Vec::<(Shape, u8)>::with_capacity(7);
        for shape in Shape::all_into_iter() {
            let counter = self.counters[shape as usize];
            if 0 < counter {
                vec.push((shape, counter));
            }
        }
        vec
    }

    /// Returns a pair of each shape and its count.
    /// ```
    /// use bitris_commands::prelude::*;
    /// assert!(ShapeCounter::empty().is_empty());
    /// assert!(!ShapeCounter::from(vec![Shape::O]).is_empty());
    /// assert!(!ShapeCounter::one_of_each().is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.counters.iter().all(|&it| it == 0)
    }

    /// Return a new shape counter taking the maximum value of each.
    /// ```
    /// use bitris_commands::prelude::*;
    /// use Shape::*;
    /// let c1 = ShapeCounter::from(vec![O, T, T, S]);
    /// let c2 = ShapeCounter::from(vec![O, O, T]);
    /// assert_eq!(c1.merge_by_max(&c2), ShapeCounter::from(vec![O, O, T, T, S]));
    /// ```
    pub fn merge_by_max(&self, rhs: &ShapeCounter) -> ShapeCounter {
        let mut new = [0; 7];
        for index in 0..7 {
            new[index] = cmp::max(self.counters[index], rhs.counters[index]);
        }
        ShapeCounter::new(new)
    }

    /// Returns a subset of shape counter with N shapes.
    /// ```
    /// use bitris_commands::prelude::*;
    /// use Shape::*;
    /// let counter = ShapeCounter::from(vec![T, T, O, S]);
    /// assert_eq!(
    ///     counter.subset(2),
    ///     vec![
    ///         ShapeCounter::from(vec![O, S]),
    ///         ShapeCounter::from(vec![T, S]),
    ///         ShapeCounter::from(vec![T, O]),
    ///         ShapeCounter::from(vec![T, T]),
    ///     ],
    /// );
    /// ```
    pub fn subset(&self, pop: usize) -> Vec<ShapeCounter> {
        if pop == 0 {
            return vec![ShapeCounter::empty()];
        }

        assert!(pop <= self.len());

        struct Buffer {
            pairs: Vec<(Shape, u8)>,
            max_taken_after_next: Vec<u8>,
        }

        fn build(buffer: &Buffer, index: usize, rest: usize, fixed: ShapeCounter, out: &mut Vec<ShapeCounter>) {
            let (shape, count) = buffer.pairs[index];
            debug_assert!(0 < count);

            let min = cmp::max(0i32, rest as i32 - buffer.max_taken_after_next[index] as i32) as u8;
            let max = cmp::min(count, rest as u8);
            for pop in min..=max {
                let next = fixed + ShapeCounter::single_shape(shape, pop);
                let rest = rest - pop as usize;
                if 0 < rest {
                    build(buffer, index + 1, rest, next, out)
                } else {
                    out.push(next);
                }
            }
        }

        let pairs = self.to_pairs();
        let max_taken_after_next = pairs.iter()
            .map(|&pair| pair.1)
            .rev()
            .take(pairs.len() - 1)
            .fold(vec![0], |mut v, count| {
                v.push(v.last().unwrap() + count);
                v
            })
            .into_iter()
            .rev()
            .collect_vec();
        let buffer = Buffer { pairs, max_taken_after_next };

        let mut counters = Vec::<ShapeCounter>::new();
        build(&buffer, 0, pop, ShapeCounter::empty(), &mut counters);

        counters
    }

    /// Returns true when it has all the other's shapes.
    /// ```
    /// use bitris_commands::prelude::*;
    /// use Shape::*;
    /// assert!(ShapeCounter::from(vec![T, T, S]).contains_all(&ShapeCounter::from(vec![])));
    /// assert!(ShapeCounter::from(vec![T, T, S]).contains_all(&ShapeCounter::from(vec![T, S])));
    /// assert!(ShapeCounter::from(vec![T, T, S]).contains_all(&ShapeCounter::from(vec![T, T, S])));
    ///
    /// assert!(!ShapeCounter::from(vec![T, T, S]).contains_all(&ShapeCounter::from(vec![O])));
    /// assert!(!ShapeCounter::from(vec![T, T, S]).contains_all(&ShapeCounter::from(vec![T, T, S, S])));
    /// ```
    pub fn contains_all(&self, other: &ShapeCounter) -> bool {
        zip(self.counters, other.counters).all(|(mine, yours)| {
            mine >= yours
        })
    }
}

impl From<Shape> for ShapeCounter {
    fn from(shape: Shape) -> Self {
        let mut counters: [u8; 7] = [0; 7];
        counters[shape as usize] += 1;
        ShapeCounter::new(counters)
    }
}

impl From<Vec<Shape>> for ShapeCounter {
    fn from(shapes: Vec<Shape>) -> Self {
        ShapeCounter::from_iter(shapes.into_iter())
    }
}

impl FromIterator<Shape> for ShapeCounter {
    fn from_iter<T: IntoIterator<Item=Shape>>(iter: T) -> Self {
        let mut counters: [u8; 7] = [0; 7];
        for shape in iter {
            counters[shape as usize] += 1;
        }
        ShapeCounter::new(counters)
    }
}

impl ops::Index<Shape> for ShapeCounter {
    type Output = u8;

    fn index(&self, shape: Shape) -> &Self::Output {
        &self.counters[shape as usize]
    }
}

impl ops::Add<ShapeCounter> for ShapeCounter {
    type Output = ShapeCounter;

    /// ```
    /// use bitris_commands::prelude::*;
    /// use Shape::*;
    /// assert_eq!(
    ///     ShapeCounter::single_shape(T, 3) + ShapeCounter::single_shape(O, 2),
    ///     ShapeCounter::from(vec![T, T, T, O, O]),
    /// );
    /// ```
    fn add(self, rhs: ShapeCounter) -> Self::Output {
        let mut new = self.counters;
        for index in 0..7 {
            new[index] += rhs.counters[index];
        }
        ShapeCounter::new(new)
    }
}

impl ops::AddAssign<ShapeCounter> for ShapeCounter {
    /// ```
    /// use bitris_commands::prelude::*;
    /// use Shape::*;
    /// let mut counter = ShapeCounter::single_shape(T, 3);
    /// counter += ShapeCounter::single_shape(O, 2);
    /// assert_eq!(counter, ShapeCounter::from(vec![T, T, T, O, O]));
    /// ```
    fn add_assign(&mut self, rhs: ShapeCounter) {
        for index in 0..7 {
            self.counters[index] += rhs.counters[index];
        }
    }
}

impl ops::Sub<ShapeCounter> for ShapeCounter {
    type Output = ShapeCounter;

    /// ```
    /// use bitris_commands::prelude::*;
    /// use Shape::*;
    /// assert_eq!(
    ///     ShapeCounter::single_shape(T, 3) - ShapeCounter::single_shape(T, 2),
    ///     ShapeCounter::from(vec![T]),
    /// );
    /// ```
    fn sub(self, rhs: ShapeCounter) -> Self::Output {
        let mut new = self.counters;
        for index in 0..7 {
            new[index] -= rhs.counters[index];
        }
        ShapeCounter::new(new)
    }
}

impl ops::SubAssign<ShapeCounter> for ShapeCounter {
    /// ```
    /// use bitris_commands::prelude::*;
    /// use Shape::*;
    /// let mut counter = ShapeCounter::single_shape(T, 3);
    /// counter -= ShapeCounter::single_shape(T, 2);
    /// assert_eq!(counter, ShapeCounter::from(vec![T]));
    /// ```
    fn sub_assign(&mut self, rhs: ShapeCounter) {
        for index in 0..7 {
            self.counters[index] -= rhs.counters[index];
        }
    }
}

impl ops::Add<Shape> for ShapeCounter {
    type Output = ShapeCounter;

    /// ```
    /// use bitris_commands::prelude::*;
    /// use Shape::*;
    /// assert_eq!(ShapeCounter::single_shape(T, 3) + O, ShapeCounter::from(vec![T, T, T, O]));
    /// ```
    fn add(self, rhs: Shape) -> Self::Output {
        let mut new = self.counters;
        new[rhs as usize] += 1;
        ShapeCounter::new(new)
    }
}

impl ops::AddAssign<Shape> for ShapeCounter {
    /// ```
    /// use bitris_commands::prelude::*;
    /// use Shape::*;
    /// let mut counter = ShapeCounter::single_shape(T, 3);
    /// counter += O;
    /// assert_eq!(counter, ShapeCounter::from(vec![T, T, T, O]));
    /// ```
    fn add_assign(&mut self, rhs: Shape) {
        self.counters[rhs as usize] += 1;
    }
}

impl ops::Sub<Shape> for ShapeCounter {
    type Output = ShapeCounter;

    /// ```
    /// use bitris_commands::prelude::*;
    /// use Shape::*;
    /// assert_eq!(ShapeCounter::single_shape(T, 3) - T, ShapeCounter::from(vec![T, T]));
    /// ```
    fn sub(self, rhs: Shape) -> Self::Output {
        let mut new = self.counters;
        new[rhs as usize] -= 1;
        ShapeCounter::new(new)
    }
}

impl ops::SubAssign<Shape> for ShapeCounter {
    /// ```
    /// use bitris_commands::prelude::*;
    /// use Shape::*;
    /// let mut counter = ShapeCounter::single_shape(T, 3);
    /// counter -= T;
    /// assert_eq!(counter, ShapeCounter::from(vec![T, T]));
    /// ```
    fn sub_assign(&mut self, rhs: Shape) {
        self.counters[rhs as usize] -= 1;
    }
}

impl<const N: usize> ops::Add<&[Shape; N]> for ShapeCounter {
    type Output = ShapeCounter;

    /// ```
    /// use bitris_commands::prelude::*;
    /// use Shape::*;
    /// assert_eq!(ShapeCounter::single_shape(T, 3) + &[O, O], ShapeCounter::from(vec![T, T, T, O, O]));
    /// ```
    fn add(self, rhs: &[Shape; N]) -> Self::Output {
        let mut new = self.counters;
        for &shape in rhs {
            new[shape as usize] += 1;
        }
        ShapeCounter::new(new)
    }
}

impl<const N: usize> ops::AddAssign<&[Shape; N]> for ShapeCounter {
    /// ```
    /// use bitris_commands::prelude::*;
    /// use Shape::*;
    /// let mut counter = ShapeCounter::single_shape(T, 3);
    /// counter += &[O, O];
    /// assert_eq!(counter, ShapeCounter::from(vec![T, T, T, O, O]));
    /// ```
    fn add_assign(&mut self, rhs: &[Shape; N]) {
        for &shape in rhs {
            self.counters[shape as usize] += 1;
        }
    }
}

impl<const N: usize> ops::Sub<&[Shape; N]> for ShapeCounter {
    type Output = ShapeCounter;

    /// ```
    /// use bitris_commands::prelude::*;
    /// use Shape::*;
    /// assert_eq!(ShapeCounter::single_shape(T, 3) - &[T, T], ShapeCounter::from(vec![T]));
    /// ```
    fn sub(self, rhs: &[Shape; N]) -> Self::Output {
        let mut new = self.counters;
        for &shape in rhs {
            new[shape as usize] -= 1;
        }
        ShapeCounter::new(new)
    }
}

impl<const N: usize> ops::SubAssign<&[Shape; N]> for ShapeCounter {
    /// ```
    /// use bitris_commands::prelude::*;
    /// use Shape::*;
    /// let mut counter = ShapeCounter::single_shape(T, 3);
    /// counter += &[T, T];
    /// assert_eq!(counter, ShapeCounter::from(vec![T; 5]));
    /// ```
    fn sub_assign(&mut self, rhs: &[Shape; N]) {
        for &shape in rhs {
            self.counters[shape as usize] -= 1;
        }
    }
}

impl ops::Mul<u8> for ShapeCounter {
    type Output = ShapeCounter;

    /// ```
    /// use bitris_commands::prelude::*;
    /// use Shape::*;
    /// let mut counter = ShapeCounter::single_shape(T, 2) + ShapeCounter::single_shape(I, 1);
    /// assert_eq!(counter * 2, ShapeCounter::from(vec![T, T, T, T, I, I]));
    /// ```
    fn mul(self, rhs: u8) -> Self::Output {
        let mut new = self.counters;
        for index in 0..7 {
            new[index] *= rhs;
        }
        ShapeCounter::new(new)
    }
}

impl ops::MulAssign<u8> for ShapeCounter {
    /// ```
    /// use bitris_commands::prelude::*;
    /// use Shape::*;
    /// let mut counter = ShapeCounter::single_shape(T, 2) + ShapeCounter::single_shape(I, 1);
    /// counter *= 2;
    /// assert_eq!(counter, ShapeCounter::from(vec![T, T, T, T, I, I]));
    /// ```
    fn mul_assign(&mut self, rhs: u8) {
        for index in 0..7 {
            self.counters[index] *= rhs;
        }
    }
}

forward_ref_op! { ShapeCounter, + ShapeCounter, = ShapeCounter }
forward_ref_op! { ShapeCounter, += ShapeCounter }
forward_ref_op! { ShapeCounter, - ShapeCounter, = ShapeCounter }
forward_ref_op! { ShapeCounter, -= ShapeCounter }
forward_ref_op! { ShapeCounter, + Shape, = ShapeCounter }
forward_ref_op! { ShapeCounter, += Shape }
forward_ref_op! { ShapeCounter, - Shape, = ShapeCounter }
forward_ref_op! { ShapeCounter, -= Shape }

#[cfg(test)]
mod tests {
    use bitris::prelude::Shape;

    use crate::ShapeCounter;

    #[test]
    fn one_of_each() {
        let counter = ShapeCounter::one_of_each();
        assert_eq!(counter.len(), 7);
        assert!(Shape::all_into_iter().any(|shape| counter[shape] == 1));
        assert!(counter.to_pairs().into_iter().all(|(_, count)| count == 1));
    }

    #[test]
    fn to_shape_counter_vec() {
        assert_eq!(ShapeCounter::empty().subset(0), vec![ShapeCounter::empty()]);

        assert_eq!(ShapeCounter::one_of_each().subset(0), vec![ShapeCounter::empty()]);
        assert_eq!(ShapeCounter::one_of_each().subset(1).len(), 7);
        assert_eq!(ShapeCounter::one_of_each().subset(2).len(), (7 * 6) / (2 * 1));
        assert_eq!(ShapeCounter::one_of_each().subset(3).len(), (7 * 6 * 5) / (3 * 2 * 1));
        assert_eq!(ShapeCounter::one_of_each().subset(4).len(), (7 * 6 * 5) / (3 * 2 * 1));
        assert_eq!(ShapeCounter::one_of_each().subset(5).len(), (7 * 6) / (2 * 1));
        assert_eq!(ShapeCounter::one_of_each().subset(6).len(), 7);
        assert_eq!(ShapeCounter::one_of_each().subset(7), vec![ShapeCounter::one_of_each()]);

        use Shape::*;
        let counter = ShapeCounter::single_shape(T, 3) + ShapeCounter::single_shape(O, 2);
        assert_eq!(counter.subset(3), vec![
            ShapeCounter::from(vec![T, O, O]),
            ShapeCounter::from(vec![T, T, O]),
            ShapeCounter::from(vec![T, T, T]),
        ]);

        let counter = ShapeCounter::single_shape(S, 2) + ShapeCounter::single_shape(Z, 4);
        assert_eq!(counter.subset(4), vec![
            ShapeCounter::from(vec![Z, Z, Z, Z]),
            ShapeCounter::from(vec![S, Z, Z, Z]),
            ShapeCounter::from(vec![S, S, Z, Z]),
        ]);

        let counter = ShapeCounter::single_shape(I, 1) + ShapeCounter::single_shape(L, 2) + ShapeCounter::single_shape(J, 1);
        assert_eq!(counter.subset(2), vec![
            ShapeCounter::from(vec![L, J]),
            ShapeCounter::from(vec![L, L]),
            ShapeCounter::from(vec![I, J]),
            ShapeCounter::from(vec![I, L]),
        ]);

        let counter = ShapeCounter::single_shape(I, 1) + ShapeCounter::single_shape(L, 2) + ShapeCounter::single_shape(J, 1);
        assert_eq!(counter.subset(3), vec![
            ShapeCounter::from(vec![L, L, J]),
            ShapeCounter::from(vec![I, L, J]),
            ShapeCounter::from(vec![I, L, L]),
        ]);
    }
}
