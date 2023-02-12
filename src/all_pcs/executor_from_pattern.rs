use bitris::prelude::*;
use thiserror::Error;

use crate::{ClippedBoard, Pattern, ShapeCounter};
use crate::all_pcs::{Builder, PcSolutions};

/// A collection of errors that occur when making the executor.
#[derive(Error, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum AllPcsFromPatternExecutorCreationError {
    #[error("Unexpected the count of board spaces.")]
    UnexpectedBoardSpaces,
    #[error("The pattern is too short to take a PC.")]
    ShortPatternDimension,
    #[error("Board height exceeds the upper limit. Up to 56 are supported.")]
    BoardIsTooHigh,
}

/// The executor to find all PCs.
#[derive(Clone, PartialEq, PartialOrd, Hash, Debug)]
pub struct AllPcsFromPatternExecutor<'a, T: RotationSystem> {
    move_rules: &'a MoveRules<'a, T>,
    clipped_board: ClippedBoard,
    pattern: &'a Pattern,
    spawn_position: BlPosition,
    allows_hold: bool,
}

impl<'a, T: RotationSystem> AllPcsFromPatternExecutor<'a, T> {
    /// Make AllPcsFromPatternExecutor.
    ///
    /// Returns `Err()` if the setting is incorrect or restricted.
    /// See `AllPcsFromPatternExecutor` for error cases.
    /// ```
    /// use std::str::FromStr;
    /// use bitris::prelude::*;
    /// use bitris_commands::{ClippedBoard, Pattern, PatternElement, ShapeCounter, ShapeOrder};
    /// use bitris_commands::all_pcs::AllPcsFromPatternExecutor;
    ///
    /// let move_rules = MoveRules::srs(AllowMove::Softdrop);
    ///
    /// let board = Board64::from_str("
    ///     XXX.....XX
    ///     XXX....XXX
    ///     XXX...XXXX
    ///     XXX....XXX
    /// ").expect("Failed to create a board");
    /// let height = 4;
    /// let clipped_board = ClippedBoard::try_new(board, height).expect("Failed to clip");
    ///
    /// let pattern = Pattern::try_new(vec![
    ///     PatternElement::One(Shape::I),
    ///     PatternElement::Permutation(vec![Shape::T, Shape::O, Shape::J, Shape::L].into(), 3),
    /// ]).unwrap();
    ///
    /// let allows_hold = true;
    ///
    /// let executor = AllPcsFromPatternExecutor::try_new(&move_rules, clipped_board, &pattern, allows_hold)
    ///     .expect("Failed to create an executor");
    ///
    /// let solutions = executor.execute();
    /// assert_eq!(solutions.len(), 10);
    /// ```
    pub fn try_new(
        move_rules: &'a MoveRules<'a, T>,
        clipped_board: ClippedBoard,
        pattern: &'a Pattern,
        allows_hold: bool,
    ) -> Result<Self, AllPcsFromPatternExecutorCreationError> {
        use AllPcsFromPatternExecutorCreationError::*;

        if 20 < clipped_board.height() {
            return Err(BoardIsTooHigh);
        }

        if clipped_board.spaces() % 4 != 0 {
            return Err(UnexpectedBoardSpaces);
        }

        if (pattern.dim_shapes() as u32) < clipped_board.spaces() / 4 {
            return Err(ShortPatternDimension);
        }

        debug_assert!(0 < clipped_board.spaces());

        // Spawn over the top of the well to avoid getting stuck.
        let spawn_position = bl(5, clipped_board.height() as i32 + 4);

        Ok(Self { move_rules, clipped_board, pattern, spawn_position, allows_hold })
    }

    /// Start the search for all PCs.
    pub fn execute(&self) -> PcSolutions {
        let shape_counters = self.pattern.to_shape_counter_vec();
        let max_shape_counter = shape_counters.iter()
            .fold(ShapeCounter::empty(), |prev, shape_counter| {
                prev.merge_by_max(shape_counter)
            });

        let aggregator = Builder::new_and_make_placed_pieces(self.clipped_board, max_shape_counter, 10)
            .to_aggregator(self.spawn_position);

        if self.allows_hold {
            aggregator.aggregate_with_pattern_allows_hold(self.pattern, &self.move_rules)
        } else {
            aggregator.aggregate_with_pattern_allows_no_hold(self.pattern, &self.move_rules)
        }
    }
}


#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use bitris::prelude::*;

    use crate::{ClippedBoard, PatternElement, ShapeCounter};
    use crate::all_pcs::AllPcsFromPatternExecutor;

    #[test]
    fn small_test_case_pattern() {
        let move_rules = MoveRules::srs(AllowMove::Softdrop);
        let board = Board64::from_str(
            "
            #..#######
            #..#######
        ").unwrap();
        let clipped_board = ClippedBoard::try_new(board, 2).unwrap();
        let pattern = vec![
            PatternElement::One(Shape::O),
        ].try_into().unwrap();
        let executor = AllPcsFromPatternExecutor::try_new(
            &move_rules, clipped_board, &pattern, true,
        ).unwrap();
        let result = executor.execute();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn small_test_case_pattern2() {
        let move_rules = MoveRules::srs(AllowMove::Softdrop);
        let board = Board64::from_str("
            #######...
            ########.#
            #..#######
            #..#######
        ").unwrap();
        let clipped_board = ClippedBoard::try_new(board, 4).unwrap();

        {
            let pattern = vec![
                PatternElement::Fixed(vec![Shape::O, Shape::Z, Shape::T].try_into().unwrap()),
            ].try_into().unwrap();
            let executor = AllPcsFromPatternExecutor::try_new(
                &move_rules, clipped_board, &pattern, true,
            ).unwrap();
            let result = executor.execute();
            assert_eq!(result.len(), 0);
        }
    }

    #[test]
    fn pco_with_i_allows_hold() {
        let move_rules = MoveRules::srs(AllowMove::Softdrop);
        let board = Board64::from_str("
            ###.....##
            ###....###
            ###...####
            ###....###
        ").unwrap();
        let clipped_board = ClippedBoard::try_new(board, 4).unwrap();
        let pattern = vec![
            PatternElement::One(Shape::I),
            PatternElement::Permutation(ShapeCounter::one_of_each(), 4),
        ].try_into().unwrap();
        let executor = AllPcsFromPatternExecutor::try_new(
            &move_rules, clipped_board, &pattern, true,
        ).unwrap();
        let result = executor.execute();
        assert_eq!(result.len(), 63);
    }

    #[test]
    fn pco_with_i_allows_no_hold() {
        let move_rules = MoveRules::srs(AllowMove::Softdrop);
        let board = Board64::from_str("
            ###.....##
            ###....###
            ###...####
            ###....###
        ").unwrap();
        let clipped_board = ClippedBoard::try_new(board, 4).unwrap();
        let pattern = vec![
            PatternElement::One(Shape::I),
            PatternElement::Permutation(ShapeCounter::one_of_each(), 3),
        ].try_into().unwrap();
        let executor = AllPcsFromPatternExecutor::try_new(
            &move_rules, clipped_board, &pattern, false,
        ).unwrap();
        let result = executor.execute();
        assert_eq!(result.len(), 46);
    }

    #[test]
    fn pco_with_iz() {
        let move_rules = MoveRules::srs(AllowMove::Softdrop);
        let board = Board64::from_str("
            ###.....##
            ###....###
            ###.....##
            ###......#
        ").unwrap();
        let clipped_board = ClippedBoard::try_new(board, 4).unwrap();
        let pattern = vec![
            PatternElement::Factorial(vec![Shape::I, Shape::Z].try_into().unwrap()),
            PatternElement::Permutation(ShapeCounter::one_of_each(), 4),
        ].try_into().unwrap();
        let executor = AllPcsFromPatternExecutor::try_new(
            &move_rules, clipped_board, &pattern, true,
        ).unwrap();
        let result = executor.execute();
        assert_eq!(result.len(), 118);
    }

    #[test]
    fn test1() {
        let move_rules = MoveRules::srs(AllowMove::Softdrop);
        let board = Board64::from_str("
            ###....###
            ###....###
            ###....###
            ###....###
        ").unwrap();
        let clipped_board = ClippedBoard::try_new(board, 4).unwrap();
        let pattern = vec![
            PatternElement::Factorial(vec![Shape::T, Shape::Z].try_into().unwrap()),
            PatternElement::Factorial(vec![Shape::T, Shape::S].try_into().unwrap()),
        ].try_into().unwrap();
        let executor = AllPcsFromPatternExecutor::try_new(
            &move_rules, clipped_board, &pattern, true,
        ).unwrap();
        let result = executor.execute();
        assert_eq!(result.len(), 3);
    }
}
