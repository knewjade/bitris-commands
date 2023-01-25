use bitris::prelude::Shape;
use itertools::Itertools;

/// A collection of operations to take one from a shape order.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub enum PopOp {
    #[default] First,
    Second,
}

/// Preserves the reference status of the order.
/// The next items to be manipulated can be identified.
#[derive(Copy, Clone, PartialEq, PartialOrd, Hash, Debug)]
pub struct OrderCursor<'a> {
    sequence: &'a ShapeOrder,
    head: Option<usize>,
    tails: usize,
}

impl<'a> OrderCursor<'a> {
    #[inline]
    fn new(sequence: &'a ShapeOrder) -> Self {
        if 0 < sequence.shapes.len() {
            Self { sequence, head: Some(0), tails: 1 }
        } else {
            Self { sequence, head: None, tails: 0 }
        }
    }

    /// Returns `true` if a pop-able shape exists next.
    #[inline]
    pub fn has_next(&self) -> bool {
        self.head.is_some()
    }

    /// Returns the count of shapes not used.
    #[inline]
    pub fn len_unused(&self) -> usize {
        self.sequence.shapes.len() - self.tails + self.head.and(Some(1)).unwrap_or(0)
    }

    /// Returns shapes that have not been used as an order.
    #[inline]
    pub fn unused_shapes(&self) -> ShapeOrder {
        ShapeOrder::new(if let Some(first) = self.head {
            let shapes = &self.sequence.shapes;
            let n = [shapes[first]];
            let x = &shapes[self.tails..shapes.len()];
            n.into_iter().chain(x.into_iter().map(|it| *it)).collect_vec()
        } else {
            Vec::new()
        })
    }

    /// Returns a popped shape and a next cursor.
    /// If nothing that can be popped next, None is returned for the shape.
    /// The next cursor is always returned as available.
    ///
    /// The shape returned by the first is that received before the second.
    /// Therefore, this function ensures the following behaviors.
    ///
    /// * If the first returns None, the second is always None.
    ///   The last shape is always assigned to the first.
    ///
    /// * If only the first is used, it's equivalent to consuming from the head of the order.
    ///   In other words, equivalent to not using a hold.
    ///   Note, however, this means that "The second is not always the hold because the last one is assigned to the first, regardless of the hold".
    #[inline]
    pub fn pop(&self, op: PopOp) -> (Option<Shape>, OrderCursor) {
        return match op {
            PopOp::First => {
                return if let Some(head) = self.head {
                    let freeze = if self.tails < self.sequence.shapes.len() {
                        // The tails exist
                        OrderCursor {
                            sequence: self.sequence,
                            head: Some(self.tails),
                            tails: self.tails + 1,
                        }
                    } else {
                        // The tails don't exist: It's the last
                        OrderCursor {
                            sequence: self.sequence,
                            head: None,
                            tails: self.tails,
                        }
                    };
                    (Some(self.sequence.shapes[head]), freeze)
                } else {
                    (None, *self)
                };
            }
            PopOp::Second => {
                if self.tails < self.sequence.shapes.len() {
                    let freeze = OrderCursor {
                        sequence: self.sequence,
                        head: self.head,
                        tails: self.tails + 1,
                    };
                    return (Some(self.sequence.shapes[self.tails]), freeze);
                }

                (None, *self)
            }
        };
    }

    /// Returns the first shape.
    #[inline]
    pub fn peek(&self, op: PopOp) -> Option<Shape> {
        match op {
            PopOp::First => self.first(),
            PopOp::Second => self.second(),
        }
    }

    /// Returns the first shape.
    #[inline]
    pub fn first(&self) -> Option<Shape> {
        self.head.map(|index| self.sequence.shapes[index])
    }

    /// Returns the second shape.
    #[inline]
    pub fn second(&self) -> Option<Shape> {
        if self.tails < self.sequence.shapes.len() {
            Some(self.sequence.shapes[self.tails])
        } else {
            None
        }
    }
}

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
    pub fn new_cursor(&self) -> OrderCursor {
        OrderCursor::new(self)
    }

    #[inline]
    pub fn shapes(&self) -> &[Shape] {
        self.shapes.as_slice()
    }
}


#[cfg(test)]
mod tests {
    use bitris::prelude::*;

    use crate::{PopOp, ShapeOrder};

    #[test]
    fn empty() {
        let sequence = ShapeOrder::new(vec![]);
        let cursor = sequence.new_cursor();

        // []()
        assert!(!cursor.has_next());
        assert_eq!(cursor.len_unused(), 0);
        assert_eq!(cursor.unused_shapes().shapes(), vec![]);
        assert_eq!(cursor.first(), None);
        assert_eq!(cursor.second(), None);

        let (shape, cursor) = cursor.pop(PopOp::First);
        assert!(!cursor.has_next());
        assert_eq!(shape, None);

        let (shape, cursor) = cursor.pop(PopOp::Second);
        assert!(!cursor.has_next());
        assert_eq!(shape, None);
    }

    #[test]
    fn one() {
        use Shape::*;

        let sequence = ShapeOrder::new(vec![T]);
        let cursor = sequence.new_cursor();

        // [](T)
        assert!(cursor.has_next());
        assert_eq!(cursor.len_unused(), 1);
        assert_eq!(cursor.unused_shapes().shapes(), vec![T]);
        assert_eq!(cursor.first(), Some(T));
        assert_eq!(cursor.second(), None);
        let (shape, cursor) = cursor.pop(PopOp::Second);
        assert_eq!(shape, None);

        // [](T)
        assert!(cursor.has_next());
        assert_eq!(cursor.len_unused(), 1);
        assert_eq!(cursor.unused_shapes().shapes(), vec![T]);
        assert_eq!(cursor.first(), Some(T));
        assert_eq!(cursor.second(), None);
        let (shape, cursor) = cursor.pop(PopOp::First);
        assert_eq!(shape, Some(T));

        assert!(!cursor.has_next());
        assert_eq!(cursor.len_unused(), 0);
        assert_eq!(cursor.unused_shapes().shapes(), vec![]);
        assert_eq!(cursor.first(), None);
        assert_eq!(cursor.second(), None);
    }

    #[test]
    fn pop_first() {
        use Shape::*;

        let sequence = ShapeOrder::new(vec![O, S]);
        let cursor = sequence.new_cursor();

        // [](O)S
        assert!(cursor.has_next());
        assert_eq!(cursor.len_unused(), 2);
        assert_eq!(cursor.unused_shapes().shapes(), vec![O, S]);
        assert_eq!(cursor.peek(PopOp::First), Some(O));
        assert_eq!(cursor.peek(PopOp::Second), Some(S));
        let (shape, cursor) = cursor.pop(PopOp::First);
        assert_eq!(shape, Some(O));

        // [](S)
        assert!(cursor.has_next());
        assert_eq!(cursor.len_unused(), 1);
        assert_eq!(cursor.unused_shapes().shapes(), vec![S]);
        assert_eq!(cursor.peek(PopOp::First), Some(S));
        assert_eq!(cursor.peek(PopOp::Second), None);
        let (shape, cursor) = cursor.pop(PopOp::First);
        assert_eq!(shape, Some(S));

        // []()
        assert!(!cursor.has_next());
        assert_eq!(cursor.len_unused(), 0);
        assert_eq!(cursor.unused_shapes().shapes(), vec![]);
        assert_eq!(cursor.peek(PopOp::First), None);
        assert_eq!(cursor.peek(PopOp::Second), None);
        let (shape, cursor) = cursor.pop(PopOp::First);
        assert_eq!(shape, None);

        assert!(!cursor.has_next());
        assert_eq!(cursor.len_unused(), 0);
        assert_eq!(cursor.unused_shapes().shapes(), vec![]);
        assert_eq!(cursor.peek(PopOp::First), None);
        assert_eq!(cursor.peek(PopOp::Second), None);
    }

    #[test]
    fn pop_second() {
        use Shape::*;

        let sequence = ShapeOrder::new(vec![O, S, T]);
        let cursor = sequence.new_cursor();

        // [](O)ST
        assert!(cursor.has_next());
        assert_eq!(cursor.len_unused(), 3);
        assert_eq!(cursor.unused_shapes().shapes(), vec![O, S, T]);
        assert_eq!(cursor.peek(PopOp::First), Some(O));
        assert_eq!(cursor.peek(PopOp::Second), Some(S));
        let (shape, cursor) = cursor.pop(PopOp::Second);
        assert_eq!(shape, Some(S));

        // [O](T)
        assert!(cursor.has_next());
        assert_eq!(cursor.len_unused(), 2);
        assert_eq!(cursor.unused_shapes().shapes(), vec![O, T]);
        assert_eq!(cursor.peek(PopOp::First), Some(O));
        assert_eq!(cursor.peek(PopOp::Second), Some(T));
        let (shape, cursor) = cursor.pop(PopOp::Second);
        assert_eq!(shape, Some(T));

        // [](O)
        assert!(cursor.has_next());
        assert_eq!(cursor.len_unused(), 1);
        assert_eq!(cursor.unused_shapes().shapes(), vec![O]);
        assert_eq!(cursor.peek(PopOp::First), Some(O));
        assert_eq!(cursor.peek(PopOp::Second), None);
        let (shape, cursor) = cursor.pop(PopOp::Second);
        assert_eq!(shape, None);

        // [](O)
        assert!(cursor.has_next());
        assert_eq!(cursor.len_unused(), 1);
        assert_eq!(cursor.unused_shapes().shapes(), vec![O]);
        assert_eq!(cursor.peek(PopOp::First), Some(O));
        assert_eq!(cursor.peek(PopOp::Second), None);
        let (shape, cursor) = cursor.pop(PopOp::First);
        assert_eq!(shape, Some(O));

        // []()
        assert!(!cursor.has_next());
        assert_eq!(cursor.len_unused(), 0);
        assert_eq!(cursor.unused_shapes().shapes(), vec![]);
        assert_eq!(cursor.peek(PopOp::First), None);
        assert_eq!(cursor.peek(PopOp::Second), None);
        let (index, cursor) = cursor.pop(PopOp::Second);
        assert_eq!(index, None);

        assert!(!cursor.has_next());
        assert_eq!(cursor.len_unused(), 0);
        assert_eq!(cursor.unused_shapes().shapes(), vec![]);
        assert_eq!(cursor.peek(PopOp::First), None);
        assert_eq!(cursor.peek(PopOp::Second), None);
    }
}
