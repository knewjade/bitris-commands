use std::collections::BTreeSet;

use bitris::prelude::*;

use crate::{Pattern, PatternElement, ShapeMatcher2};

/// Based on the pattern, it extends to the sequence obtained in the hold.
#[derive(Clone, PartialEq, PartialOrd, Hash, Debug)]
pub struct HoldExpandedPattern<'a> {
    pub(crate) pattern: &'a Pattern,
    pub(crate) expanded_elements_vec: Vec<Vec<HoldExpandedPatternElement>>,
}

impl HoldExpandedPattern<'_> {
    /// Returns a matcher that determines whether a sequence of the shapes is contained in the expanded pattern with hold.
    pub fn new_matcher(&self) -> ShapeMatcher2 {
        self.into()
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub(crate) enum HoldExpandedPatternElement {
    Fixed(Shape),
    Shared(usize),
}

impl<'a> From<&'a Pattern> for HoldExpandedPattern<'a> {
    fn from(pattern: &'a Pattern) -> Self {
        fn to_hold_expanded_pattern_elements((element_index, element): (usize, &PatternElement)) -> Vec<HoldExpandedPatternElement> {
            use HoldExpandedPatternElement::*;
            match element {
                PatternElement::One(shape) => vec![Fixed(*shape)],
                PatternElement::Fixed(shapes) => shapes.to_vec().into_iter().map(|shape| Fixed(shape)).collect(),
                PatternElement::Wildcard => vec![Shared(element_index)],
                PatternElement::Permutation(_, pop) => vec![Shared(element_index)].repeat(*pop),
                PatternElement::Factorial(counter) => vec![Shared(element_index)].repeat(counter.len()),
            }
        }

        let element_sequence: Vec<HoldExpandedPatternElement> = pattern.elements.iter()
            .enumerate()
            .flat_map(to_hold_expanded_pattern_elements)
            .collect();

        assert!(!element_sequence.is_empty());

        struct Builder<'a> {
            results: BTreeSet<Vec<HoldExpandedPatternElement>>,
            buffer: Vec<&'a HoldExpandedPatternElement>,
        }

        impl<'a> Builder<'a> {
            fn build(&mut self, cursor: OrderCursor<'a, HoldExpandedPatternElement>) {
                if !cursor.has_next() {
                    self.results.insert(self.buffer.iter().map(|&it| *it).collect());
                    return;
                }

                {
                    let (popped, next_cursor) = cursor.pop(PopOp::First);
                    if let Some(pair) = popped {
                        self.buffer.push(pair);
                        self.build(next_cursor);
                        self.buffer.pop();
                    }
                }
                {
                    let (popped, next_cursor) = cursor.pop(PopOp::Second);
                    if let Some(pair) = popped {
                        self.buffer.push(pair);
                        self.build(next_cursor);
                        self.buffer.pop();
                    }
                }
            }
        }

        let mut builder = Builder {
            results: BTreeSet::default(),
            buffer: Vec::new(),
        };

        builder.build(OrderCursor::from(&element_sequence));

        Self {
            pattern,
            expanded_elements_vec: builder.results.into_iter().collect(),
        }
    }
}
