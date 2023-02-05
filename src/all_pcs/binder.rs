use std::rc::Rc;

use bitris::prelude::*;
use bitris::srs::SrsKickTable;

use crate::{ClippedBoard, ShapeOrder};
use crate::all_pcs::{AllPcsExecutor, AllPcsExecutorCreationError, PcSolutions};

/// The binder to hold and tie settings for `PcPossibleExecutor`.
#[derive(Clone, PartialEq, PartialOrd, Hash, Debug)]
pub struct AllPcsExecutorBinder<T: RotationSystem> {
    pub rotation_system: Rc<T>,
    pub allow_move: AllowMove,
    pub clipped_board: ClippedBoard,
    pub shape_order: Rc<ShapeOrder>,
    pub allows_hold: bool,
}

impl AllPcsExecutorBinder<SrsKickTable> {
    /// Making the executor with SRS. See `AllPcsExecutorBinder::default()` for more details.
    pub fn srs() -> Self {
        AllPcsExecutorBinder::default(Rc::from(SrsKickTable))
    }
}

impl<T: RotationSystem> AllPcsExecutorBinder<T> {
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
    ///   + [required] shape_order: empty order. You must set this.
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

    // TODO desc. See `AllPcsExecutorBinder::{try_new, execute}` for more details.
    pub fn try_execute(&self) -> Result<PcSolutions, AllPcsExecutorCreationError> {
        let move_rules = MoveRules::new(self.rotation_system.as_ref(), self.allow_move);
        let executor = self.try_bind(&move_rules)?;
        Ok(executor.execute())
    }

    fn try_bind<'a>(&'a self, move_rules: &'a MoveRules<T>) -> Result<AllPcsExecutor<T>, AllPcsExecutorCreationError> {
        AllPcsExecutor::try_new(
            move_rules,
            self.clipped_board,
            self.shape_order.as_ref(),
            self.allows_hold,
        )
    }
}
