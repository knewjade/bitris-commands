use std::ops;

use bitris::Shape;
use derive_more::Constructor;

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
    /// assert_eq!(counter.total_size(), 0);
    /// ```
    #[inline]
    pub fn empty() -> Self {
        ShapeCounter::new([0; 7])
    }

    /// ```
    /// use bitris_commands::prelude::*;
    /// use Shape::*;
    /// let counter = ShapeCounter::one_of_each();
    /// assert_eq!(counter.total_size(), 7);
    /// assert_eq!(counter, ShapeCounter::from(vec![T, I, O, L, J, S, Z]));
    /// ```
    #[inline]
    pub fn one_of_each() -> Self {
        ShapeCounter::new([1; 7])
    }

    #[inline]
    pub fn total_size(&self) -> usize {
        self.counters.into_iter()
            .map(|it| it as usize)
            .fold(0, |sum, it| sum + it)
    }

    /// Returns a pair of each shape and its count.
    /// ```
    /// use bitris_commands::prelude::*;
    /// let counter = ShapeCounter::from(vec![Shape::O, Shape::O, Shape::S]);
    /// assert_eq!(counter.total_size(), 3);
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
}

impl From<Vec<Shape>> for ShapeCounter {
    fn from(shapes: Vec<Shape>) -> Self {
        let mut counters: [u8; 7] = [0; 7];
        for shape in shapes {
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


#[cfg(test)]
mod tests {
    use bitris::Shape;

    use crate::ShapeCounter;

    #[test]
    fn one_of_each() {
        let counter = ShapeCounter::one_of_each();
        assert_eq!(counter.total_size(), 7);
        assert!(Shape::all_into_iter().any(|shape| counter[shape] == 1));
        assert!(counter.to_pairs().into_iter().all(|(_, count)| count == 1));
    }
}
