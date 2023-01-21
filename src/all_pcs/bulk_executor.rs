use bitris::prelude::*;
use thiserror::Error;

use crate::{ClippedBoard, Pattern};
use crate::all_pcs::{IndexedPieces, PredefinedPiece, Builder};

// TODO SequenceやOrderをcollect()したい
// TODO FromIteratorをじっそうする？
// TODO assert! > debug_assert!

/// A collection of errors that occur when making the executor.
#[derive(Error, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum AllPcsExecutorBulkCreationError {
    #[error("Unexpected the count of board spaces.")]
    UnexpectedBoardSpaces,
    #[error("The pattern is too short to take a PC.")]
    ShortPatternDimension,
    #[error("Board height exceeds the upper limit. Up to 20 are supported.")]
    BoardIsTooHigh,
}

/// The executor to find PC possibles.
#[derive(Clone, PartialEq, PartialOrd, Hash, Debug)]
pub struct AllPcsBulkExecutor<'a, T: RotationSystem> {
    move_rules: &'a MoveRules<'a, T>,
    clipped_board: ClippedBoard,
    pattern: &'a Pattern,
    allows_hold: bool,
    has_extra_shapes: bool,
    spawn_position: BlPosition,
}

impl<'a, T: RotationSystem> AllPcsBulkExecutor<'a, T> {
    // TODO desc
    pub fn try_new(
        move_rules: &'a MoveRules<T>,
        clipped_board: ClippedBoard,
        pattern: &'a Pattern,
        allows_hold: bool,
    ) -> Result<Self, AllPcsExecutorBulkCreationError> {
        use AllPcsExecutorBulkCreationError::*;

        if 20 < clipped_board.height() {
            return Err(BoardIsTooHigh);
        }

        if clipped_board.spaces() % 4 != 0 {
            return Err(UnexpectedBoardSpaces);
        }

        let dimension = pattern.dim_shapes() as u32;
        if dimension < clipped_board.spaces() / 4 {
            return Err(ShortPatternDimension);
        }

        debug_assert!(0 < clipped_board.spaces());

        let has_extra_shapes = clipped_board.spaces() / 4 < dimension;

        // Spawn over the top of the well to avoid getting stuck.
        let spawn_position = bl(5, clipped_board.height() as i32 + 4);

        Ok(Self { move_rules, clipped_board, pattern, allows_hold, has_extra_shapes, spawn_position })
    }

    /// TODO desc Start the search for PC possible in bulk.
    pub fn execute(&self) -> u64 {
        let indexed_pieces = IndexedPieces::<PredefinedPiece>::new(self.clipped_board.height() as usize);
        Builder::new(self.clipped_board, indexed_pieces, 10)
            .to_aggregator()
            .aggregate()
    }
}
