use std::rc::Rc;

use bitris::prelude::*;
use bitris::srs::SrsKickTable;

use crate::{ClippedBoard, ShapeOrder};
use crate::all_pcs::{AllPcsFromOrderExecutor, AllPcsFromOrderExecutorCreationError, PcSolutions};

/// The binder to hold and tie settings for `AllPcsExecutor`.
#[derive(Clone, PartialEq, PartialOrd, Hash, Debug)]
pub struct AllPcsFromOrderExecutorBinder<T: RotationSystem> {
    pub rotation_system: Rc<T>,
    pub allow_move: AllowMove,
    pub clipped_board: ClippedBoard,
    pub shape_order: Rc<ShapeOrder>,
    pub allows_hold: bool,
}

impl AllPcsFromOrderExecutorBinder<SrsKickTable> {
    /// Making the executor with SRS. See `AllPcsFromOrderExecutorBinder::default()` for more details.
    pub fn srs() -> Self {
        AllPcsFromOrderExecutorBinder::default(Rc::from(SrsKickTable))
    }
}

impl<T: RotationSystem> AllPcsFromOrderExecutorBinder<T> {
    pub fn new(
        rotation_system: Rc<T>,
        allow_move: AllowMove,
        clipped_board: ClippedBoard,
        shape_order: Rc<ShapeOrder>,
        allows_hold: bool,
    ) -> Self {
        Self {
            rotation_system,
            allow_move,
            clipped_board,
            shape_order,
            allows_hold,
        }
    }

    /// Making the executor with default.
    ///
    /// The default values are as follows:
    ///   + [required] rotation_system: set an argument (wrapped by Rc)
    ///   + [required] shape_order: empty order. You must set this.  (wrapped by Rc)
    ///   + allow move: softdrop
    ///   + board: blank
    ///   + height: 4 lines
    ///   + allows hold: yes
    pub fn default(rotation_system: Rc<T>) -> Self {
        Self {
            rotation_system,
            allow_move: AllowMove::Softdrop,
            clipped_board: ClippedBoard::try_new(Board64::blank(), 4).unwrap(),
            shape_order: Rc::from(ShapeOrder::new(vec![])),
            allows_hold: true,
        }
    }

    // See `AllPcsFromOrderExecutorBinder::{try_new, execute}` for more details.
    pub fn try_execute(&self) -> Result<PcSolutions, AllPcsFromOrderExecutorCreationError> {
        let move_rules = MoveRules::new(self.rotation_system.as_ref(), self.allow_move);
        let executor = self.try_bind(&move_rules)?;
        Ok(executor.execute())
    }

    fn try_bind<'a>(&'a self, move_rules: &'a MoveRules<T>) -> Result<AllPcsFromOrderExecutor<T>, AllPcsFromOrderExecutorCreationError> {
        AllPcsFromOrderExecutor::try_new(
            move_rules,
            self.clipped_board,
            self.shape_order.as_ref(),
            self.allows_hold,
        )
    }
}


#[cfg(test)]
mod tests {
    use std::rc::Rc;
    use std::str::FromStr;

    use bitris::prelude::*;

    use crate::{ClippedBoard, ShapeOrder};
    use crate::all_pcs::{AllPcsFromOrderExecutorBinder, AllPcsFromOrderExecutorCreationError};

    #[test]
    fn reuse() {
        use Shape::*;

        let mut binder = AllPcsFromOrderExecutorBinder::srs();
        let board = Board64::from_str("
            ..........
            ....####..
            ....######
            ....######
        ").unwrap();
        binder.clipped_board = ClippedBoard::try_new(board, 4).unwrap();

        binder.shape_order = Rc::new(ShapeOrder::new(vec![
            Z, S, I, O, L, J, T,
        ]));
        let solutions = binder.try_execute().unwrap();
        assert_eq!(solutions.len(), 0);

        binder.shape_order = Rc::new(ShapeOrder::new(vec![
            I, O, T, Z, S, J, L,
        ]));
        let solutions = binder.try_execute().unwrap();
        assert_eq!(solutions.len(), 5);
    }

    #[test]
    fn error() {
        use Shape::*;
        use AllPcsFromOrderExecutorCreationError::*;

        let mut binder = AllPcsFromOrderExecutorBinder::srs();
        let board = Board64::from_str("
            ..........
            ....####.#
            ....######
            ....######
        ").unwrap();
        binder.clipped_board = ClippedBoard::try_new(board, 4).unwrap();

        binder.shape_order = Rc::new(ShapeOrder::new(vec![
            Z, S, I, O, L, J, T,
        ]));

        assert_eq!(binder.try_execute().unwrap_err(), UnexpectedBoardSpaces);

        let board = Board64::from_str("
            ..........
            ....####..
            ....######
            ....######
        ").unwrap();
        binder.clipped_board = ClippedBoard::try_new(board, 4).unwrap();

        binder.shape_order = Rc::new(ShapeOrder::default());

        assert_eq!(binder.try_execute().unwrap_err(), ShortPatternDimension);

        binder.shape_order = Rc::new(ShapeOrder::new(vec![Z]));

        assert_eq!(binder.try_execute().unwrap_err(), ShortPatternDimension);
    }
}
