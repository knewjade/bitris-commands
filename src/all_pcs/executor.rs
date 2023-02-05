use bitris::prelude::*;
use thiserror::Error;

use crate::{ClippedBoard, ShapeCounter, ShapeOrder};
use crate::all_pcs::{Builder, PcSolutions};

/// A collection of errors that occur when making the executor.
#[derive(Error, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum AllPcsExecutorCreationError {
    #[error("Unexpected the count of board spaces.")]
    UnexpectedBoardSpaces,
    #[error("The pattern is too short to take a PC.")]
    ShortPatternDimension,
    #[error("Board height exceeds the upper limit. Up to 56 are supported.")]
    BoardIsTooHigh,
}

/// The executor to find PC possibles.
#[derive(Clone, PartialEq, PartialOrd, Hash, Debug)]
pub struct AllPcsExecutor<'a, T: RotationSystem> {
    move_rules: &'a MoveRules<'a, T>,
    clipped_board: ClippedBoard,
    shape_order: &'a ShapeOrder,
    spawn_position: BlPosition,
    allows_hold: bool,
}

impl<'a, T: RotationSystem> AllPcsExecutor<'a, T> {
    // TODO desc
    pub fn try_new(
        move_rules: &'a MoveRules<'a, T>,
        clipped_board: ClippedBoard,
        shape_order: &'a ShapeOrder,
        allows_hold: bool,
    ) -> Result<Self, AllPcsExecutorCreationError> {
        use AllPcsExecutorCreationError::*;

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

    /// TODO desc Start the search for PC possible in bulk.
    pub fn execute(&self) -> PcSolutions {
        let shape_counter: ShapeCounter = self.shape_order.shapes().into();

        let aggregator = Builder::new_and_make_placed_pieces(self.clipped_board, shape_counter, 10)
            .to_aggregator(self.spawn_position);

        aggregator.aggregate_with_sequence_order(self.shape_order, &self.move_rules, self.allows_hold)
    }
}
