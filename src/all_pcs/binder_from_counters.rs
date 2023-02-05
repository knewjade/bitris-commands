use std::rc::Rc;

use bitris::prelude::*;
use bitris::srs::SrsKickTable;

use crate::{ClippedBoard, ShapeCounter};
use crate::all_pcs::{AllPcsFromCountersExecutor, AllPcsFromCountersExecutorCreationError, PcSolutions};

/// The binder to hold and tie settings for `AllPcsFromCountersExecutor`.
#[derive(Clone, PartialEq, PartialOrd, Hash, Debug)]
pub struct AllPcsFromCountersExecutorBinder<T: RotationSystem> {
    pub rotation_system: Rc<T>,
    pub allow_move: AllowMove,
    pub clipped_board: ClippedBoard,
    pub shape_counters: Rc<Vec<ShapeCounter>>,
}

impl AllPcsFromCountersExecutorBinder<SrsKickTable> {
    /// Making the executor with SRS. See `AllPcsFromCountersExecutorBinder::default()` for more details.
    pub fn srs() -> Self {
        AllPcsFromCountersExecutorBinder::default(Rc::from(SrsKickTable))
    }
}

impl<T: RotationSystem> AllPcsFromCountersExecutorBinder<T> {
    pub fn new(
        rotation_system: Rc<T>,
        allow_move: AllowMove,
        clipped_board: ClippedBoard,
        shape_counters: Rc<Vec<ShapeCounter>>,
    ) -> Self {
        Self {
            rotation_system,
            allow_move,
            clipped_board,
            shape_counters,
        }
    }

    /// Making the executor with default.
    ///
    /// The default values are as follows:
    ///   + [required] rotation_system: set an argument (wrapped by Rc)
    ///   + [required] shape_counters: empty counter. You must set this. (wrapped by Rc)
    ///   + allow move: softdrop
    ///   + board: blank
    ///   + height: 4 lines
    pub fn default(rotation_system: Rc<T>) -> Self {
        Self {
            rotation_system,
            allow_move: AllowMove::Softdrop,
            clipped_board: ClippedBoard::try_new(Board64::blank(), 4).unwrap(),
            shape_counters: Rc::from(Vec::new()),
        }
    }

    // See `AllPcsFromCountersExecutorBinder::{try_new, execute}` for more details.
    pub fn try_execute(&self) -> Result<PcSolutions, AllPcsFromCountersExecutorCreationError> {
        let move_rules = MoveRules::new(self.rotation_system.as_ref(), self.allow_move);
        let executor = self.try_bind(&move_rules)?;
        Ok(executor.execute())
    }

    fn try_bind<'a>(&'a self, move_rules: &'a MoveRules<T>) -> Result<AllPcsFromCountersExecutor<T>, AllPcsFromCountersExecutorCreationError> {
        AllPcsFromCountersExecutor::try_new(
            move_rules,
            self.clipped_board,
            self.shape_counters.as_ref(),
        )
    }
}


#[cfg(test)]
mod tests {
    use std::rc::Rc;
    use std::str::FromStr;

    use bitris::prelude::*;

    use crate::{ClippedBoard, ShapeCounter};
    use crate::all_pcs::{AllPcsFromCountersExecutorBinder, AllPcsFromCountersExecutorCreationError};

    #[test]
    fn reuse() {
        use Shape::*;

        let mut binder = AllPcsFromCountersExecutorBinder::srs();
        let board = Board64::from_str("
            ..........
            ....####..
            ....######
            ....######
        ").unwrap();
        binder.clipped_board = ClippedBoard::try_new(board, 4).unwrap();

        binder.shape_counters = Rc::new(vec![
            ShapeCounter::one_of_each() - I,
        ]);
        let solutions = binder.try_execute().unwrap();
        assert_eq!(solutions.len(), 0);

        binder.shape_counters = Rc::new(vec![
            ShapeCounter::one_of_each() - T,
        ]);
        let solutions = binder.try_execute().unwrap();
        assert_eq!(solutions.len(), 4);
    }

    #[test]
    fn error() {
        use Shape::*;
        use AllPcsFromCountersExecutorCreationError::*;

        let mut binder = AllPcsFromCountersExecutorBinder::srs();
        let board = Board64::from_str("
            ..........
            ....####.#
            ....######
            ....######
        ").unwrap();
        binder.clipped_board = ClippedBoard::try_new(board, 4).unwrap();

        binder.shape_counters = Rc::new(vec![
            ShapeCounter::one_of_each(),
        ]);

        assert_eq!(binder.try_execute().unwrap_err(), UnexpectedBoardSpaces);

        let board = Board64::from_str("
            ..........
            ....####..
            ....######
            ....######
        ").unwrap();
        binder.clipped_board = ClippedBoard::try_new(board, 4).unwrap();

        binder.shape_counters = Rc::new(vec![
            ShapeCounter::single_shape(T, 1),
        ]);

        assert_eq!(binder.try_execute().unwrap_err(), ShortCounterDimension);

        binder.shape_counters = Rc::new(vec![]);
        assert_eq!(binder.try_execute().unwrap_err(), CountersAreEmpty);
    }
}
