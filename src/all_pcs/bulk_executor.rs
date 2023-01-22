use bitris::prelude::*;
use thiserror::Error;

use crate::{ClippedBoard, ShapeCounter};
use crate::all_pcs::{Builder, IndexedPieces, PredefinedPiece};

// TODO SequenceやOrderをcollect()したい
// TODO FromIteratorをじっそうする？
// TODO assert! > debug_assert!
// TODO &[]に対応させてたい

/// A collection of errors that occur when making the executor.
#[derive(Error, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum AllPcsFromCounterExecutorBulkCreationError {
    #[error("Unexpected the count of board spaces.")]
    UnexpectedBoardSpaces,
    #[error("The max of the counter dimension is too short to take a PC.")]
    ShortCounterDimension,
    #[error("Board height exceeds the upper limit. Up to 20 are supported.")]
    BoardIsTooHigh,
    #[error("The shape counters are empty.")]
    CountersAreEmpty,
}

/// The executor to find PC possibles.
#[derive(Clone, PartialEq, PartialOrd, Hash, Debug)]
pub struct AllPcsFromCounterBulkExecutor<'a, T: RotationSystem> {
    move_rules: &'a MoveRules<'a, T>,
    clipped_board: ClippedBoard,
    shape_counters: &'a Vec<ShapeCounter>,
    spawn_position: BlPosition,
}

impl<'a, T: RotationSystem> AllPcsFromCounterBulkExecutor<'a, T> {
    // TODO desc
    pub fn try_new(
        move_rules: &'a MoveRules<T>,
        clipped_board: ClippedBoard,
        shape_counters: &'a Vec<ShapeCounter>,
    ) -> Result<Self, AllPcsFromCounterExecutorBulkCreationError> {
        use AllPcsFromCounterExecutorBulkCreationError::*;

        if 20 < clipped_board.height() {
            return Err(BoardIsTooHigh);
        }

        if clipped_board.spaces() % 4 != 0 {
            return Err(UnexpectedBoardSpaces);
        }

        if shape_counters.is_empty() {
            return Err(CountersAreEmpty);
        }

        let max_dimension = shape_counters.iter()
            .map(|shape_counter| shape_counter.len())
            .max()
            .unwrap();
        if max_dimension < (clipped_board.spaces() / 4) as usize {
            return Err(ShortCounterDimension);
        }

        debug_assert!(0 < clipped_board.spaces());

        // Spawn over the top of the well to avoid getting stuck.
        let spawn_position = bl(5, clipped_board.height() as i32 + 4);

        Ok(Self { move_rules, clipped_board, shape_counters, spawn_position })
    }

    /// TODO desc Start the search for PC possible in bulk.
    pub fn execute(&self) -> u64 {
        let indexed_pieces = IndexedPieces::<PredefinedPiece>::new(self.clipped_board.height() as usize);
        let max_shape_counter = self.shape_counters.iter()
            .fold(ShapeCounter::empty(), |prev, shape_counter| {
                prev.merge_by_max(shape_counter)
            });

        let aggregator = Builder::new(self.clipped_board, indexed_pieces, max_shape_counter, 10)
            .to_aggregator(self.spawn_position);
        aggregator.aggregate_with_shape_counters(self.shape_counters)
    }
}


#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use bitris::prelude::*;

    use crate::{ClippedBoard, ShapeCounter};
    use crate::all_pcs::{AllPcsFromCounterBulkExecutor, AllPcsFromCounterExecutorBulkCreationError};

    #[test]
    fn wildcard3() {
        let move_rules = MoveRules::srs(AllowMove::Softdrop);
        let board = Board64::from_str(
            "
            ...#######
            ...#######
            ...#######
            ...#######
        ").unwrap();
        let clipped_board = ClippedBoard::try_new(board, 4).unwrap();
        let shape_counters = vec![
            ShapeCounter::one_of_each() * 3,
        ];
        let executor = AllPcsFromCounterBulkExecutor::try_new(
            &move_rules, clipped_board, &shape_counters,
        ).unwrap();
        let result = executor.execute();
        assert_eq!(result, 79);
    }

    #[test]
    fn partial_wildcard3() {
        use Shape::*;
        let move_rules = MoveRules::srs(AllowMove::Softdrop);
        let board = Board64::from_str(
            "
            ...#######
            ...#######
            ...#######
            ...#######
        ").unwrap();
        let clipped_board = ClippedBoard::try_new(board, 4).unwrap();
        let shape_counters = vec![
            ShapeCounter::from(vec![T, L, J, O, I]) * 3,
        ];
        let executor = AllPcsFromCounterBulkExecutor::try_new(
            &move_rules, clipped_board, &shape_counters,
        ).unwrap();
        let result = executor.execute();
        assert_eq!(result, 57);
    }

    #[test]
    fn one_of_each() {
        let move_rules = MoveRules::srs(AllowMove::Softdrop);
        let board = Board64::from_str(
            "
            ...#######
            ...#######
            ...#######
            ...#######
        ").unwrap();
        let clipped_board = ClippedBoard::try_new(board, 4).unwrap();
        let shape_counters = vec![
            ShapeCounter::one_of_each(),
        ];
        let executor = AllPcsFromCounterBulkExecutor::try_new(
            &move_rules, clipped_board, &shape_counters,
        ).unwrap();
        let result = executor.execute();
        assert_eq!(result, 38);
    }

    #[test]
    fn partial_one_of_each() {
        use Shape::*;
        let move_rules = MoveRules::srs(AllowMove::Softdrop);
        let board = Board64::from_str(
            "
            ...#######
            ...#######
            ...#######
            ...#######
        ").unwrap();
        let clipped_board = ClippedBoard::try_new(board, 4).unwrap();
        let shape_counters = vec![
            ShapeCounter::one_of_each() - S - Z,
        ];
        let executor = AllPcsFromCounterBulkExecutor::try_new(
            &move_rules, clipped_board, &shape_counters,
        ).unwrap();
        let result = executor.execute();
        assert_eq!(result, 26);
    }

    #[test]
    fn some_shape_counters() {
        use Shape::*;
        let move_rules = MoveRules::srs(AllowMove::Softdrop);
        let board = Board64::from_str(
            "
            ...#######
            ...#######
            ...#######
            ...#######
        ").unwrap();
        let clipped_board = ClippedBoard::try_new(board, 4).unwrap();
        let shape_counters = vec![
            ShapeCounter::from(vec![S, T, L]),
            ShapeCounter::from(vec![Z, T, J]),
        ];
        let executor = AllPcsFromCounterBulkExecutor::try_new(
            &move_rules, clipped_board, &shape_counters,
        ).unwrap();
        let result = executor.execute();
        assert_eq!(result, 2);
    }

    #[test]
    fn no_solutions() {
        let move_rules = MoveRules::srs(AllowMove::Softdrop);
        let board = Board64::from_str(
            "
            ..#.......
            ..#.......
            ..#.......
            ...#......
        ").unwrap();
        let clipped_board = ClippedBoard::try_new(board, 4).unwrap();
        let shape_counters = vec![
            ShapeCounter::one_of_each() * 10,
        ];
        let executor = AllPcsFromCounterBulkExecutor::try_new(
            &move_rules, clipped_board, &shape_counters,
        ).unwrap();
        let result = executor.execute();
        assert_eq!(result, 0);
    }

    #[test]
    fn error_unexpected_spaces() {
        let move_rules = MoveRules::srs(AllowMove::Softdrop);
        let board = Board64::from_str(
            "
            ...#######
            ..########
        ").unwrap();
        let clipped_board = ClippedBoard::try_new(board, 2).unwrap();
        let shape_counters = vec![
            ShapeCounter::one_of_each(),
        ];
        assert_eq!(
            AllPcsFromCounterBulkExecutor::try_new(&move_rules, clipped_board, &shape_counters).unwrap_err(),
            AllPcsFromCounterExecutorBulkCreationError::UnexpectedBoardSpaces,
        );
    }

    #[test]
    fn error_short_counter_dimension() {
        let move_rules = MoveRules::srs(AllowMove::Softdrop);
        let board = Board64::from_str(
            "
            ....######
            ....######
        ").unwrap();
        let clipped_board = ClippedBoard::try_new(board, 2).unwrap();
        let shape_counters = vec![
            ShapeCounter::one(Shape::T),
            ShapeCounter::one(Shape::I),
            ShapeCounter::one(Shape::O),
        ];
        assert_eq!(
            AllPcsFromCounterBulkExecutor::try_new(&move_rules, clipped_board, &shape_counters).unwrap_err(),
            AllPcsFromCounterExecutorBulkCreationError::ShortCounterDimension,
        );
    }

    #[test]
    fn error_board_is_too_high() {
        let move_rules = MoveRules::srs(AllowMove::Softdrop);
        let clipped_board = ClippedBoard::try_new(Board64::blank(), 21).unwrap();
        let shape_counters = vec![
            ShapeCounter::one_of_each() * 10,
        ];
        assert_eq!(
            AllPcsFromCounterBulkExecutor::try_new(&move_rules, clipped_board, &shape_counters).unwrap_err(),
            AllPcsFromCounterExecutorBulkCreationError::BoardIsTooHigh,
        );
    }
}
