use bitris::prelude::*;
use thiserror::Error;

use crate::{ClippedBoard, ShapeCounter, ShapeOrder};
use crate::all_pcs::{Builder, PcSolutions};

/// A collection of errors that occur when making the executor.
#[derive(Error, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum AllPcsFromOrderExecutorCreationError {
    #[error("Unexpected the count of board spaces.")]
    UnexpectedBoardSpaces,
    #[error("The pattern is too short to take a PC.")]
    ShortPatternDimension,
    #[error("Board height exceeds the upper limit. Up to 56 are supported.")]
    BoardIsTooHigh,
}

/// The executor to find all PCs.
#[derive(Clone, PartialEq, PartialOrd, Hash, Debug)]
pub struct AllPcsFromOrderExecutor<'a, T: RotationSystem> {
    move_rules: &'a MoveRules<'a, T>,
    clipped_board: ClippedBoard,
    shape_order: &'a ShapeOrder,
    spawn_position: BlPosition,
    allows_hold: bool,
}

impl<'a, T: RotationSystem> AllPcsFromOrderExecutor<'a, T> {
    /// Make AllPcsFromOrderExecutor.
    ///
    /// Returns `Err()` if the setting is incorrect or restricted.
    /// See `AllPcsFromOrderExecutorCreationError` for error cases.
    /// ```
    /// use std::str::FromStr;
    /// use bitris::prelude::*;
    /// use bitris_commands::{ClippedBoard, Pattern, PatternElement, ShapeCounter, ShapeOrder};
    /// use bitris_commands::all_pcs::AllPcsFromOrderExecutor;
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
    /// let shape_order = ShapeOrder::new(vec![
    ///     Shape::I, Shape::T, Shape::S, Shape::Z,
    /// ]);
    ///
    /// let allows_hold = true;
    ///
    /// let executor = AllPcsFromOrderExecutor::try_new(&move_rules, clipped_board, &shape_order, allows_hold)
    ///     .expect("Failed to create an executor");
    ///
    /// let solutions = executor.execute();
    /// assert_eq!(solutions.len(), 2);
    /// ```
    pub fn try_new(
        move_rules: &'a MoveRules<'a, T>,
        clipped_board: ClippedBoard,
        shape_order: &'a ShapeOrder,
        allows_hold: bool,
    ) -> Result<Self, AllPcsFromOrderExecutorCreationError> {
        use AllPcsFromOrderExecutorCreationError::*;

        if 20 < clipped_board.height() {
            return Err(BoardIsTooHigh);
        }

        if clipped_board.spaces() % 4 != 0 {
            return Err(UnexpectedBoardSpaces);
        }

        if (shape_order.len() as u32) < clipped_board.spaces() / 4 {
            return Err(ShortPatternDimension);
        }

        debug_assert!(0 < clipped_board.spaces());

        // Spawn over the top of the well to avoid getting stuck.
        let spawn_position = bl(5, clipped_board.height() as i32 + 4);

        Ok(Self { move_rules, clipped_board, shape_order, spawn_position, allows_hold })
    }

    /// Start the search for all PCs.
    pub fn execute(&self) -> PcSolutions {
        let shape_counter: ShapeCounter = self.shape_order.shapes().into();

        let aggregator = Builder::new_and_make_placed_pieces(self.clipped_board, shape_counter, 10)
            .to_aggregator(self.spawn_position);

        aggregator.aggregate_with_sequence_order(self.shape_order, &self.move_rules, self.allows_hold)
    }
}
