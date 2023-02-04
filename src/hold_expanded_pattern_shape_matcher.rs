use bitris::prelude::*;
use tinyvec::ArrayVec;

use crate::{HoldExpandedPattern, PatternElement, ShapeCounter};
use crate::hold_expanded_pattern::HoldExpandedPatternElement;

/// Determines if a sequence of shapes matches a expanded pattern with hold.
#[derive(Copy, Clone, PartialEq, PartialOrd, Hash, Debug)]
pub struct HoldExpandedPatternShapeMatcher {
    expanded_elements_vec_index: usize,

    // Arrays of pairs (index of a pattern element, consumed shapes).
    // Holds make it possible to proceed to the next element before the element is completely consumed,
    // allowing up to 2 elements to be stored.
    stored: ArrayVec<[(usize, ShapeCounter); 2]>,

    // Index of a expanded pattern element in candidates.
    // If the last expanded pattern element is reached and there is no next element, it will be None.
    current: Option<usize>,
}

/// Determines if a sequence of shapes matches a pattern.
#[derive(Clone, PartialEq, PartialOrd, Hash, Debug)]
pub struct ShapeMatcher2<'a> {
    pattern: &'a HoldExpandedPattern<'a>,
    candidates: Vec<HoldExpandedPatternShapeMatcher>,
    succeed_always: bool,
}

impl<'a> ShapeMatcher2<'a> {
    #[inline]
    fn new(pattern: &'a HoldExpandedPattern) -> Self {
        assert!(!pattern.expanded_elements_vec.is_empty());
        assert!(pattern.expanded_elements_vec.iter().all(|it| !it.is_empty()));

        let candidates = pattern.expanded_elements_vec.iter()
            .enumerate()
            .filter(|(_, elements)| !elements.is_empty())
            .map(|(p_index, _)| HoldExpandedPatternShapeMatcher {
                expanded_elements_vec_index: p_index,
                stored: ArrayVec::new(),
                current: Some(0),
            })
            .collect();

        Self {
            pattern,
            candidates,
            succeed_always: false,
        }
    }

    /// Returns `true` if a remaining shape exists next.
    #[inline]
    pub fn has_next(&self) -> bool {
        !self.candidates.is_empty() && self.candidates.iter().any(|candidate| candidate.current.is_some())
    }

    /// Returns the result of the match and the next matcher.
    /// If the next shape is contained in the pattern, returns `true` and a matcher to match the next shape.
    /// If not contained, returns `false` and an empty matcher with no next shape.
    ///
    /// If the pattern has successfully matched until the end, the returned matcher will always return `true` regardless of the length of the sequence.
    /// For example, if the pattern represents "TSO", the sequence "TSOZ..." will also be considered `true`.
    /// Note that to take the length of the sequence into account, use `has_next()` as well.
    #[inline]
    pub fn match_shape(&self, target: Shape) -> (bool, ShapeMatcher2<'a>) {
        if self.succeed_always {
            return (true, self.clone());
        }

        if self.candidates.is_empty() {
            return (false, self.clone());
        }

        let mut next_candidates = Vec::<HoldExpandedPatternShapeMatcher>::new();
        'loop_candidates: for candidate in &self.candidates {
            let expanded_pattern_element_index = if let Some(index) = candidate.current {
                index
            } else {
                continue 'loop_candidates;
            };

            let expanded_pattern_element = &self.pattern.expanded_elements_vec[candidate.expanded_elements_vec_index][expanded_pattern_element_index];
            let next_stored = match expanded_pattern_element {
                HoldExpandedPatternElement::Fixed(shape) => {
                    if target != *shape {
                        continue 'loop_candidates;
                    }
                    candidate.stored
                }
                HoldExpandedPatternElement::Shared(pattern_element_index) => {
                    let make_next_stored = || -> Option<ArrayVec<[(usize, ShapeCounter); 2]>> {
                        let pattern_element = self.pattern.pattern.elements[*pattern_element_index];
                        let mut next_stored = candidate.stored.clone();

                        // Check if the previous state is still in the stores.
                        let mut store_index = None;
                        for index in 0..=1 {
                            if candidate.stored.get(index).filter(|it| it.0 == *pattern_element_index).is_some() {
                                store_index = Some(index);
                                break;
                            }
                        }

                        // Initialize the store since it is the first element to appear.
                        let store_index = if let Some(index) = store_index {
                            index
                        } else {
                            let new_index = next_stored.len();
                            next_stored.push((*pattern_element_index, ShapeCounter::empty()));
                            new_index
                        };

                        next_stored[store_index].1 += target;
                        let next_consumed = &next_stored[store_index].1;

                        let matches = match pattern_element {
                            PatternElement::Wildcard => true,
                            PatternElement::Permutation(counter, _) => counter.contains_all(next_consumed),
                            PatternElement::Factorial(counter) => counter.contains_all(next_consumed),
                            _ => panic!("Unreachable"),
                        };

                        if !matches {
                            return None;
                        }

                        // If it is the last element in that pattern element, remove it from the store.
                        if next_consumed.len() == pattern_element.dim_shapes() {
                            next_stored.remove(store_index);
                        }

                        Some(next_stored)
                    };

                    if let Some(next_stored) = make_next_stored() {
                        next_stored
                    } else {
                        continue 'loop_candidates;
                    }
                }
            };

            next_candidates.push(HoldExpandedPatternShapeMatcher {
                expanded_elements_vec_index: candidate.expanded_elements_vec_index,
                stored: next_stored,
                current: if expanded_pattern_element_index + 1 < self.pattern.expanded_elements_vec[candidate.expanded_elements_vec_index].len() {
                    Some(expanded_pattern_element_index + 1)
                } else {
                    None
                },
            });
        }

        let succeed = !next_candidates.is_empty();
        let succeed_always = succeed && next_candidates.iter().any(|candidate| candidate.current.is_none());
        (succeed, Self {
            pattern: self.pattern,
            candidates: if succeed_always {
                Vec::new()
            } else {
                next_candidates
            },
            succeed_always,
        })
    }
}

impl<'a> From<&'a HoldExpandedPattern<'a>> for ShapeMatcher2<'a> {
    fn from(pattern: &'a HoldExpandedPattern) -> Self {
        Self::new(pattern)
    }
}


#[cfg(test)]
mod tests {
    use bitris::prelude::Shape;

    use crate::{HoldExpandedPattern, Pattern, PatternElement};

    #[test]
    fn matcher_case1() {
        use Shape::*;
        let pattern = Pattern::try_from(vec![
            PatternElement::Fixed(vec![T, I].try_into().unwrap()),
            PatternElement::Factorial(vec![L, J].try_into().unwrap()),
            PatternElement::One(O),
        ]).unwrap();

        let hold_expanded_pattern = HoldExpandedPattern::from(&pattern);

        for shapes in [[T, I, J, L, O], [I, L, J, O, T], [I, J, T, O, L]] {
            let mut matcher = hold_expanded_pattern.new_matcher();
            for shape in shapes {
                assert!(matcher.has_next());

                let (matched, next) = matcher.match_shape(shape);
                assert!(matched);

                matcher = next;
            }

            assert!(!matcher.has_next());

            let (matched, matcher) = matcher.match_shape(S);
            assert!(matched);
            assert!(!matcher.has_next());
        }
        {
            let matcher = hold_expanded_pattern.new_matcher();
            assert!(matcher.has_next());

            let (matched, matcher) = matcher.match_shape(O);
            assert!(!matched);
            assert!(!matcher.has_next());

            let (matched, matcher) = matcher.match_shape(O);
            assert!(!matched);
            assert!(!matcher.has_next());
        }
    }

    #[test]
    fn matcher_case2() {
        use Shape::*;
        let pattern = vec![
            PatternElement::Factorial(vec![T, Z].try_into().unwrap()),
            PatternElement::Factorial(vec![T, S].try_into().unwrap()),
        ].try_into().unwrap();

        let hold_expanded_pattern = HoldExpandedPattern::from(&pattern);

        for shapes in [[T, T, Z, S]] {
            let mut matcher = hold_expanded_pattern.new_matcher();
            for shape in shapes {
                assert!(matcher.has_next());

                let (matched, next) = matcher.match_shape(shape);
                assert!(matched);

                matcher = next;
            }

            assert!(!matcher.has_next());

            let (matched, matcher) = matcher.match_shape(O);
            assert!(matched);
            assert!(!matcher.has_next());
        }
        {
            let matcher = hold_expanded_pattern.new_matcher();
            assert!(matcher.has_next());

            let (matched, matcher) = matcher.match_shape(O);
            assert!(!matched);
            assert!(!matcher.has_next());

            let (matched, matcher) = matcher.match_shape(O);
            assert!(!matched);
            assert!(!matcher.has_next());
        }
    }

    #[test]
    fn matcher_case3() {
        use Shape::*;
        let pattern = vec![
            PatternElement::Permutation(vec![S, Z].try_into().unwrap(), 1),
            PatternElement::Permutation(vec![L, J].try_into().unwrap(), 2),
        ].try_into().unwrap();

        let hold_expanded_pattern = HoldExpandedPattern::from(&pattern);

        for shapes in [[S, L, J], [Z, L, J], [L, S, J], [L, J, Z]] {
            let mut matcher = hold_expanded_pattern.new_matcher();
            for shape in shapes {
                assert!(matcher.has_next());

                let (matched, next) = matcher.match_shape(shape);
                assert!(matched);

                matcher = next;
            }

            assert!(!matcher.has_next());

            let (matched, matcher) = matcher.match_shape(O);
            assert!(matched);
            assert!(!matcher.has_next());
        }
        {
            let matcher = hold_expanded_pattern.new_matcher();
            assert!(matcher.has_next());

            let (matched, matcher) = matcher.match_shape(O);
            assert!(!matched);
            assert!(!matcher.has_next());

            let (matched, matcher) = matcher.match_shape(O);
            assert!(!matched);
            assert!(!matcher.has_next());
        }
    }
}
