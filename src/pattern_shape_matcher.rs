use bitris::prelude::*;

use crate::{Pattern, PatternElement, ShapeCounter, ShapeMatcher};

/// Determines if a sequence of shapes matches a pattern element.
#[derive(Copy, Clone, PartialEq, PartialOrd, Hash, Debug)]
struct PatternShapeMatcherBuffer {
    consumed: ShapeCounter,
    index: usize,
}

impl PatternShapeMatcherBuffer {
    #[inline]
    fn empty() -> Self {
        Self { consumed: ShapeCounter::empty(), index: 0 }
    }

    fn match_shape(self, element: &PatternElement, target: Shape) -> (bool, Option<Self>) {
        #[inline]
        fn new_next_matcher(
            buffer: PatternShapeMatcherBuffer,
            element_dim_shapes: usize,
            target: Shape,
        ) -> Option<PatternShapeMatcherBuffer> {
            if buffer.index + 1 < element_dim_shapes {
                Some(PatternShapeMatcherBuffer {
                    consumed: buffer.consumed + target,
                    index: buffer.index + 1,
                })
            } else {
                None
            }
        }

        match element {
            PatternElement::One(shape) => (target == *shape, None),
            PatternElement::Fixed(shapes) => {
                match shapes.get(self.index) {
                    None => (false, None),
                    Some(shape) => {
                        if target == shape {
                            (true, new_next_matcher(self, shapes.len(), target))
                        } else {
                            (false, None)
                        }
                    }
                }
            }
            PatternElement::Wildcard => (true, None),
            PatternElement::Permutation(shape_counter, pop) => {
                let remaining = shape_counter - self.consumed;
                if remaining.contains(target) {
                    (true, new_next_matcher(self, *pop, target))
                } else {
                    (false, None)
                }
            }
            PatternElement::Factorial(shape_counter) => {
                let remaining = shape_counter - self.consumed;
                if remaining.contains(target) {
                    (true, new_next_matcher(self, shape_counter.len(), target))
                } else {
                    (false, None)
                }
            }
        }
    }
}

/// Determines if a sequence of shapes matches a pattern.
#[derive(Copy, Clone, PartialEq, PartialOrd, Hash, Debug)]
pub struct PatternShapeMatcher<'a> {
    pattern: &'a Pattern,
    current: Option<(usize, PatternShapeMatcherBuffer)>,
    next: Option<usize>,
    failed: bool,
}

impl<'a> PatternShapeMatcher<'a> {
    #[inline]
    fn new(pattern: &'a Pattern) -> Self {
        assert!(!pattern.elements.is_empty());
        match pattern.elements.len() {
            1 => Self { pattern, current: Some((0, PatternShapeMatcherBuffer::empty())), next: None, failed: false },
            _ => Self { pattern, current: Some((0, PatternShapeMatcherBuffer::empty())), next: Some(1), failed: false },
        }
    }
}

impl<'a> ShapeMatcher for PatternShapeMatcher<'a> {
    /// Returns `true` if a remaining shape exists next.
    #[inline]
    fn has_next(&self) -> bool {
        self.current.is_some()
    }

    /// Returns the result of the match and the next matcher.
    /// If the next shape is contained in the pattern, returns `true` and a matcher to match the next shape.
    /// If not contained, returns `false` and an empty matcher with no next shape.
    ///
    /// If the pattern has successfully matched until the end, the returned matcher will always return `true` regardless of the length of the sequence.
    /// For example, if the pattern represents "TSO", the sequence "TSOZ..." will also be considered `true`.
    /// Note that to take the length of the sequence into account, use `has_next()` as well.
    /// ```
    /// use bitris_commands::prelude::*;
    /// use Shape::*;
    /// use bitris_commands::PatternShapeMatcher;
    /// let pattern = Pattern::try_from(vec![
    ///     PatternElement::Fixed(vec![T].try_into().unwrap()),
    ///     PatternElement::Permutation(vec![L, J].try_into().unwrap(), 1),
    /// ]).unwrap();
    /// {
    ///     // Succeed
    ///     let matcher = PatternShapeMatcher::from(&pattern);
    ///
    ///     let (matched, matcher) = matcher.match_shape(T);
    ///     assert!(matched);
    ///     assert!(matcher.has_next());
    ///
    ///     let (matched, matcher) = matcher.match_shape(L);
    ///     assert!(matched);
    ///     assert!(!matcher.has_next());
    ///
    ///     // If all pattern elements are consumed, return true afterwards.
    ///     let (matched, matcher) = matcher.match_shape(O);
    ///     assert!(matched);
    ///     assert!(!matcher.has_next());
    /// }
    /// {
    ///     // Failure
    ///     let matcher = PatternShapeMatcher::from(&pattern);
    ///
    ///     // Patterns don't start with I.
    ///     let (matched, matcher) = matcher.match_shape(I);
    ///     assert!(!matched);
    ///     assert!(!matcher.has_next());
    /// }
    /// ```
    fn match_shape(&self, target: Shape) -> (bool, PatternShapeMatcher<'a>) {
        if self.failed {
            return (false, self.clone());
        }

        return match self.current {
            None => (true, self.clone()),
            Some((current_index, current_buffer)) => {
                let (matched, next_buffer) = current_buffer.match_shape(&self.pattern.elements[current_index], target);
                if !matched {
                    return (false, Self {
                        pattern: self.pattern,
                        current: None,
                        next: None,
                        failed: true,
                    });
                }

                (true, match next_buffer {
                    Some(buffer) => Self {
                        pattern: self.pattern,
                        current: Some((current_index, buffer)),
                        next: self.next,
                        failed: false,
                    },
                    None => match self.next {
                        None => Self {
                            pattern: self.pattern,
                            current: None,
                            next: None,
                            failed: false,
                        },
                        Some(next_index) => Self {
                            pattern: self.pattern,
                            current: Some((next_index, PatternShapeMatcherBuffer::empty())),
                            next: if next_index + 1 < self.pattern.elements.len() {
                                Some(next_index + 1)
                            } else {
                                None
                            },
                            failed: false,
                        },
                    },
                })
            }
        };
    }
}

impl<'a> From<&'a Pattern> for PatternShapeMatcher<'a> {
    fn from(pattern: &'a Pattern) -> Self {
        Self::new(pattern)
    }
}


#[cfg(test)]
mod tests {
    use bitris::prelude::Shape;

    use crate::{Pattern, PatternElement, ShapeMatcher};

    #[test]
    fn matcher_one() {
        use Shape::*;
        let pattern = Pattern::try_from(vec![
            PatternElement::One(T),
            PatternElement::One(I),
            PatternElement::One(O),
        ]).unwrap();

        {
            let mut matcher = pattern.new_matcher();

            for shape in [T, I, O] {
                assert!(matcher.has_next());

                let (matched, next) = matcher.match_shape(shape);
                assert!(matched);

                matcher = next;
            }

            assert!(!matcher.has_next());

            let (matched, matcher) = matcher.match_shape(Z);
            assert!(matched);
            assert!(!matcher.has_next());
        }

        {
            let matcher = pattern.new_matcher();
            assert!(matcher.has_next());

            let (matched, matcher) = matcher.match_shape(O);
            assert!(!matched);
            assert!(!matcher.has_next());

            let (matched, matcher) = matcher.match_shape(T);
            assert!(!matched);
            assert!(!matcher.has_next());
        }
    }

    #[test]
    fn matcher_fixed() {
        use Shape::*;
        let pattern = Pattern::try_from(vec![
            PatternElement::Fixed(vec![S].try_into().unwrap()),
            PatternElement::Fixed(vec![T, I, O].try_into().unwrap()),
        ]).unwrap();

        {
            let mut matcher = pattern.new_matcher();

            for shape in [S, T, I, O] {
                assert!(matcher.has_next());

                let (matched, next) = matcher.match_shape(shape);
                assert!(matched);

                matcher = next;
            }

            assert!(!matcher.has_next());

            let (matched, matcher) = matcher.match_shape(Z);
            assert!(matched);
            assert!(!matcher.has_next());
        }

        {
            let matcher = pattern.new_matcher();
            assert!(matcher.has_next());

            let (matched, matcher) = matcher.match_shape(O);
            assert!(!matched);
            assert!(!matcher.has_next());

            let (matched, matcher) = matcher.match_shape(S);
            assert!(!matched);
            assert!(!matcher.has_next());
        }
    }

    #[test]
    fn matcher_wildcard() {
        use Shape::*;
        let pattern = Pattern::try_from(vec![
            PatternElement::Wildcard,
            PatternElement::Wildcard,
        ]).unwrap();

        {
            for shapes in [[S, T], [T, T]] {
                let mut matcher = pattern.new_matcher();

                for shape in shapes {
                    assert!(matcher.has_next());

                    let (matched, next) = matcher.match_shape(shape);
                    assert!(matched);

                    matcher = next;
                }

                assert!(!matcher.has_next());

                let (matched, matcher) = matcher.match_shape(Z);
                assert!(matched);
                assert!(!matcher.has_next());
            }
        }
    }

    #[test]
    fn matcher_permutation() {
        use Shape::*;
        let pattern = Pattern::try_from(vec![
            PatternElement::Permutation(vec![T, T, I, O].try_into().unwrap(), 2),
            PatternElement::Permutation(vec![L, J].try_into().unwrap(), 2),
        ]).unwrap();

        {
            for shapes in [[T, T, L, J], [I, O, J, L]] {
                let mut matcher = pattern.new_matcher();

                for shape in shapes {
                    assert!(matcher.has_next());

                    let (matched, next) = matcher.match_shape(shape);
                    assert!(matched);

                    matcher = next;
                }

                assert!(!matcher.has_next());

                let (matched, matcher) = matcher.match_shape(Z);
                assert!(matched);
                assert!(!matcher.has_next());
            }
        }

        {
            let matcher = pattern.new_matcher();
            assert!(matcher.has_next());

            let (matched, matcher) = matcher.match_shape(T);
            assert!(matched);
            assert!(matcher.has_next());

            let (matched, matcher) = matcher.match_shape(T);
            assert!(matched);
            assert!(matcher.has_next());

            let (matched, matcher) = matcher.match_shape(T);
            assert!(!matched);
            assert!(!matcher.has_next());

            let (matched, matcher) = matcher.match_shape(L);
            assert!(!matched);
            assert!(!matcher.has_next());
        }

        {
            let matcher = pattern.new_matcher();
            assert!(matcher.has_next());

            let (matched, matcher) = matcher.match_shape(I);
            assert!(matched);
            assert!(matcher.has_next());

            let (matched, matcher) = matcher.match_shape(I);
            assert!(!matched);
            assert!(!matcher.has_next());

            let (matched, matcher) = matcher.match_shape(T);
            assert!(!matched);
            assert!(!matcher.has_next());
        }
    }

    #[test]
    fn matcher_factorial() {
        use Shape::*;
        let pattern = Pattern::try_from(vec![
            PatternElement::Factorial(vec![T, T, O].try_into().unwrap()),
            PatternElement::Factorial(vec![L, J].try_into().unwrap()),
        ]).unwrap();

        {
            for shapes in [[T, T, O, J, L], [T, O, T, L, J], [O, T, T, L, J]] {
                let mut matcher = pattern.new_matcher();

                for shape in shapes {
                    assert!(matcher.has_next());

                    let (matched, next) = matcher.match_shape(shape);
                    assert!(matched);

                    matcher = next;
                }

                assert!(!matcher.has_next());

                let (matched, matcher) = matcher.match_shape(Z);
                assert!(matched);
                assert!(!matcher.has_next());
            }
        }

        {
            let matcher = pattern.new_matcher();
            assert!(matcher.has_next());

            let (matched, matcher) = matcher.match_shape(T);
            assert!(matched);
            assert!(matcher.has_next());

            let (matched, matcher) = matcher.match_shape(T);
            assert!(matched);
            assert!(matcher.has_next());

            let (matched, matcher) = matcher.match_shape(T);
            assert!(!matched);
            assert!(!matcher.has_next());

            let (matched, matcher) = matcher.match_shape(L);
            assert!(!matched);
            assert!(!matcher.has_next());
        }

        {
            let matcher = pattern.new_matcher();
            assert!(matcher.has_next());

            let (matched, matcher) = matcher.match_shape(O);
            assert!(matched);
            assert!(matcher.has_next());

            let (matched, matcher) = matcher.match_shape(O);
            assert!(!matched);
            assert!(!matcher.has_next());

            let (matched, matcher) = matcher.match_shape(T);
            assert!(!matched);
            assert!(!matcher.has_next());
        }
    }
}
