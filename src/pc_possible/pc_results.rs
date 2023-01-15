use std::collections::hash_map::{IntoIter, Iter};

use fxhash::FxHashMap;

use crate::ShapeSequence;

/// Holds the results of Perfect Clears.
///
/// The results is managed in 3-states
/// * Succeed / PC possible: `Some(true)`
/// * Failed / PC impossible: `Some(false)`
/// * Pending: `None`
///
/// Therefore, the shape sequences to be searched (key) are established at `new()`.
#[derive(Clone, PartialEq, Default, Debug)]
pub struct PcResults {
    succeed: FxHashMap<ShapeSequence, Option<bool>>,
}

impl PcResults {
    #[inline]
    pub fn new(sequences: &Vec<ShapeSequence>) -> Self {
        let mut succeed = FxHashMap::<ShapeSequence, Option<bool>>::default();
        succeed.reserve(sequences.len());
        for order in sequences {
            succeed.insert(order.clone(), None);
        }
        Self { succeed }
    }

    #[inline]
    pub fn accept_if_present(&mut self, sequence: &ShapeSequence, succeed: bool) -> bool {
        if let Some(_) = self.succeed.get(&sequence) {
            self.succeed.insert(sequence.clone(), Some(succeed));
            true
        } else {
            false
        }
    }

    #[inline]
    #[allow(dead_code)]
    pub(crate) fn contains_key(&self, order: &ShapeSequence) -> bool {
        self.succeed.contains_key(order)
    }

    /// Returns the result of a shape sequence.
    /// ```
    /// use bitris_commands::prelude::*;
    /// use bitris_commands::pc_possible::PcResults;
    /// use Shape::*;
    ///
    /// let mut result = PcResults::new(&vec![
    ///     ShapeSequence::new(vec!(I, T, S)),
    ///     ShapeSequence::new(vec!(I, T, Z)),
    /// ]);
    ///
    /// assert_eq!(result.get(&ShapeSequence::new(vec!(I, T, S))), None);
    /// result.accept_if_present(&ShapeSequence::new(vec!(I, T, S)), true);
    /// assert_eq!(result.get(&ShapeSequence::new(vec!(I, T, S))), Some(true));
    ///
    /// assert_eq!(result.get(&ShapeSequence::new(vec!(I, T, Z))), None);
    /// result.accept_if_present(&ShapeSequence::new(vec!(I, T, Z)), false);
    /// assert_eq!(result.get(&ShapeSequence::new(vec!(I, T, Z))), Some(false));
    /// ```
    #[inline]
    pub fn get(&self, sequence: &ShapeSequence) -> Option<bool> {
        self.succeed.get(sequence).map(|it| *it).unwrap_or(None)
    }

    /// Returns accepted shape sequence. The order of the shape sequences is undefined.
    /// ```
    /// use itertools::Itertools;
    /// use bitris_commands::prelude::*;
    /// use bitris_commands::pc_possible::PcResults;
    /// use Shape::*;
    ///
    /// let mut result = PcResults::new(&vec![
    ///     ShapeSequence::new(vec!(I, T, S)),
    ///     ShapeSequence::new(vec!(I, T, Z)),
    ///     ShapeSequence::new(vec!(I, T, O)),
    /// ]);
    ///
    /// result.accept_if_present(&ShapeSequence::new(vec!(I, T, S)), true);
    /// result.accept_if_present(&ShapeSequence::new(vec!(I, T, Z)), false);
    ///
    /// assert_eq!(
    ///     // The order of the shape sequences is undefined.
    ///     result.accepted_shape_sequences().into_iter().sorted().collect_vec(),
    ///     vec![
    ///         &ShapeSequence::new(vec!(I, T, S)),
    ///         &ShapeSequence::new(vec!(I, T, Z)),
    ///     ],
    /// );
    /// ```
    #[inline]
    pub fn accepted_shape_sequences(&self) -> Vec<&ShapeSequence> {
        self.succeed.iter()
            .filter(|(_, value)| value.is_some())
            .map(|(key, _)| key)
            .collect()
    }

    /// Returns the pair of shape sequence and result. The order of the shape sequences is undefined.
    /// ```
    /// use itertools::Itertools;
    /// use bitris_commands::prelude::*;
    /// use bitris_commands::pc_possible::PcResults;
    /// use Shape::*;
    ///
    /// let mut result = PcResults::new(&vec![
    ///     ShapeSequence::new(vec!(I, T, S)),
    ///     ShapeSequence::new(vec!(I, T, Z)),
    ///     ShapeSequence::new(vec!(I, T, O)),
    /// ]);
    ///
    /// result.accept_if_present(&ShapeSequence::new(vec!(I, T, S)), true);
    /// result.accept_if_present(&ShapeSequence::new(vec!(I, T, Z)), false);
    ///
    /// assert_eq!(
    ///     // The order of the shape sequences is undefined.
    ///     result.iter().sorted().collect_vec(),
    ///     vec![
    ///         (&ShapeSequence::new(vec!(I, T, O)), &None),
    ///         (&ShapeSequence::new(vec!(I, T, S)), &Some(true)),
    ///         (&ShapeSequence::new(vec!(I, T, Z)), &Some(false)),
    ///     ],
    /// );
    /// ```
    #[inline]
    pub fn iter(&self) -> Iter<'_, ShapeSequence, Option<bool>> {
        self.succeed.iter()
    }

    /// Returns the pair of shape sequence and result. The order of the shape sequences is undefined.
    /// ```
    /// use itertools::Itertools;
    /// use bitris_commands::prelude::*;
    /// use bitris_commands::pc_possible::PcResults;
    /// use Shape::*;
    ///
    /// let mut result = PcResults::new(&vec![
    ///     ShapeSequence::new(vec!(I, T, S)),
    ///     ShapeSequence::new(vec!(I, T, Z)),
    ///     ShapeSequence::new(vec!(I, T, O)),
    /// ]);
    ///
    /// result.accept_if_present(&ShapeSequence::new(vec!(I, T, S)), true);
    /// result.accept_if_present(&ShapeSequence::new(vec!(I, T, Z)), false);
    ///
    /// assert_eq!(
    ///     // The order of the shape sequences is undefined.
    ///     result.into_iter().sorted().collect_vec(),
    ///     vec![
    ///         (ShapeSequence::new(vec!(I, T, O)), None),
    ///         (ShapeSequence::new(vec!(I, T, S)), Some(true)),
    ///         (ShapeSequence::new(vec!(I, T, Z)), Some(false)),
    ///     ],
    /// );
    /// ```
    #[inline]
    pub fn into_iter(self) -> IntoIter<ShapeSequence, Option<bool>> {
        self.succeed.into_iter()
    }

    /// Returns the count of shape sequences found to be succeed.
    /// ```
    /// use bitris_commands::prelude::*;
    /// use bitris_commands::pc_possible::PcResults;
    /// use Shape::*;
    ///
    /// let mut result = PcResults::new(&vec![
    ///     ShapeSequence::new(vec!(O, L)),
    ///     ShapeSequence::new(vec!(O, J)),
    ///     ShapeSequence::new(vec!(O, T)),
    /// ]);
    ///
    /// result.accept_if_present(&ShapeSequence::new(vec!(O, L)), true);
    /// result.accept_if_present(&ShapeSequence::new(vec!(O, J)), true);
    ///
    /// assert_eq!(result.count_succeed(), 2);
    /// ```
    #[inline]
    pub fn count_succeed(&self) -> u64 {
        self.succeed.values()
            .filter(|value| value.unwrap_or(false))
            .count() as u64
    }

    /// Returns the count of shape sequences found to be failed.
    /// ```
    /// use bitris_commands::prelude::*;
    /// use bitris_commands::pc_possible::PcResults;
    /// use Shape::*;
    ///
    /// let mut result = PcResults::new(&vec![
    ///     ShapeSequence::new(vec!(O, S)),
    ///     ShapeSequence::new(vec!(O, Z)),
    ///     ShapeSequence::new(vec!(O, T)),
    /// ]);
    ///
    /// result.accept_if_present(&ShapeSequence::new(vec!(O, S)), false);
    /// result.accept_if_present(&ShapeSequence::new(vec!(O, Z)), false);
    ///
    /// assert_eq!(result.count_failed(), 2);
    /// ```
    #[inline]
    pub fn count_failed(&self) -> u64 {
        self.succeed.values()
            .filter(|value| value.map(|flag| !flag).unwrap_or(false))
            .count() as u64
    }

    /// Returns the count of shape sequences for which results were found.
    /// ```
    /// use bitris_commands::prelude::*;
    /// use bitris_commands::pc_possible::PcResults;
    /// use Shape::*;
    ///
    /// let mut result = PcResults::new(&vec![
    ///     ShapeSequence::new(vec!(O, I)),
    ///     ShapeSequence::new(vec!(O, S)),
    ///     ShapeSequence::new(vec!(O, Z)),
    ///     ShapeSequence::new(vec!(O, T)),
    /// ]);
    ///
    /// result.accept_if_present(&ShapeSequence::new(vec!(O, I)), true);
    /// result.accept_if_present(&ShapeSequence::new(vec!(O, S)), false);
    /// result.accept_if_present(&ShapeSequence::new(vec!(O, Z)), false);
    ///
    /// assert_eq!(result.count_accepted(), 3);
    /// ```
    #[inline]
    pub fn count_accepted(&self) -> u64 {
        self.succeed.values()
            .filter(|value| value.is_some())
            .count() as u64
    }

    /// Returns the count of shape sequences for which results are not yet found.
    /// ```
    /// use bitris_commands::prelude::*;
    /// use bitris_commands::pc_possible::PcResults;
    /// use Shape::*;
    ///
    /// let mut result = PcResults::new(&vec![
    ///     ShapeSequence::new(vec!(O, I)),
    ///     ShapeSequence::new(vec!(O, S)),
    ///     ShapeSequence::new(vec!(O, Z)),
    ///     ShapeSequence::new(vec!(O, T)),
    /// ]);
    ///
    /// result.accept_if_present(&ShapeSequence::new(vec!(O, I)), true);
    /// result.accept_if_present(&ShapeSequence::new(vec!(O, S)), false);
    /// result.accept_if_present(&ShapeSequence::new(vec!(O, Z)), false);
    ///
    /// assert_eq!(result.count_pending(), 1);
    /// ```
    #[inline]
    pub fn count_pending(&self) -> u64 {
        self.succeed.values()
            .filter(|value| value.is_none())
            .count() as u64
    }

    /// Return the count of all shape sequences independent of the result.
    /// ```
    /// use bitris_commands::prelude::*;
    /// use bitris_commands::pc_possible::PcResults;
    /// use Shape::*;
    ///
    /// let mut result = PcResults::new(&vec![
    ///     ShapeSequence::new(vec!(O, I)),
    ///     ShapeSequence::new(vec!(O, S)),
    ///     ShapeSequence::new(vec!(O, Z)),
    ///     ShapeSequence::new(vec!(O, T)),
    /// ]);
    ///
    /// assert_eq!(result.count_keys(), 4);
    /// ```
    #[inline]
    pub fn count_keys(&self) -> usize {
        self.succeed.len()
    }
}


#[cfg(test)]
mod tests {
    use bitris::prelude::*;

    use crate::pc_possible::PcResults;
    use crate::ShapeSequence;

    #[test]
    fn pc_rate_result() {
        use Shape::*;
        let mut result = PcResults::new(&vec![
            ShapeSequence::new(vec!(I, T, O)),
            ShapeSequence::new(vec!(I, T, S)),
            ShapeSequence::new(vec!(I, T, Z)),
        ]);

        assert_eq!(result.accepted_shape_sequences().len(), 0);
        assert_eq!(result.count_succeed(), 0);
        assert_eq!(result.count_failed(), 0);
        assert_eq!(result.count_accepted(), 0);
        assert_eq!(result.count_pending(), 3);
        assert_eq!(result.count_keys(), 3);

        assert!(result.accept_if_present(&ShapeSequence::new(vec!(I, T, S)), true));

        assert_eq!(result.accepted_shape_sequences().len(), 1);
        assert_eq!(result.count_succeed(), 1);
        assert_eq!(result.count_failed(), 0);
        assert_eq!(result.count_accepted(), 1);
        assert_eq!(result.count_pending(), 2);
        assert_eq!(result.count_keys(), 3);

        assert!(result.accept_if_present(&ShapeSequence::new(vec!(I, T, Z)), false));

        assert_eq!(result.accepted_shape_sequences().len(), 2);
        assert_eq!(result.count_succeed(), 1);
        assert_eq!(result.count_failed(), 1);
        assert_eq!(result.count_accepted(), 2);
        assert_eq!(result.count_pending(), 1);
        assert_eq!(result.count_keys(), 3);

        assert!(!result.accept_if_present(&ShapeSequence::new(vec!(Z, Z, Z)), true));

        assert_eq!(result.accepted_shape_sequences().len(), 2);
        assert_eq!(result.count_succeed(), 1);
        assert_eq!(result.count_failed(), 1);
        assert_eq!(result.count_accepted(), 2);
        assert_eq!(result.count_pending(), 1);
        assert_eq!(result.count_keys(), 3);

        {
            let sequence = ShapeSequence::new(vec!(I, T, S));
            assert!(result.contains_key(&sequence));
            assert_eq!(result.get(&sequence), Some(true));
        }

        {
            let sequence = ShapeSequence::new(vec!(I, T, Z));
            assert!(result.contains_key(&sequence));
            assert_eq!(result.get(&sequence), Some(false));
        }

        {
            let sequence = ShapeSequence::new(vec!(I, T, O));
            assert!(result.contains_key(&sequence));
            assert_eq!(result.get(&sequence), None);
        }

        {
            let sequence = ShapeSequence::new(vec!(Z, Z, Z));
            assert!(!result.contains_key(&sequence));
            assert_eq!(result.get(&sequence), None);
        }
    }
}
