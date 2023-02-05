use std::rc::Rc;

use bitris::prelude::*;
use bitris::srs::SrsKickTable;

use crate::{ClippedBoard, Pattern, PatternElement, ShapeCounter};
use crate::all_pcs::{AllPcsFromPatternExecutor, AllPcsFromPatternExecutorCreationError, PcSolutions};

/// The binder to hold and tie settings for `PcPossibleExecutor`.
#[derive(Clone, PartialEq, PartialOrd, Hash, Debug)]
pub struct AllPcsFromPatternExecutorBinder<T: RotationSystem> {
    pub rotation_system: Rc<T>,
    pub allow_move: AllowMove,
    pub clipped_board: ClippedBoard,
    pub pattern: Rc<Pattern>,
    pub allows_hold: bool,
}

impl AllPcsFromPatternExecutorBinder<SrsKickTable> {
    /// Making the executor with SRS. See `AllPcsFromPatternExecutorBinder::default()` for more details.
    pub fn srs() -> Self {
        AllPcsFromPatternExecutorBinder::default(Rc::from(SrsKickTable))
    }
}

impl<T: RotationSystem> AllPcsFromPatternExecutorBinder<T> {
    pub fn new(
        rotation_system: Rc<T>,
        allow_move: AllowMove,
        clipped_board: ClippedBoard,
        pattern: Rc<Pattern>,
        allows_hold: bool,
    ) -> Self {
        Self {
            rotation_system,
            allow_move,
            clipped_board,
            pattern,
            allows_hold,
        }
    }

    /// Making the executor with default.
    ///
    /// The default values are as follows:
    ///   + [required] rotation_system: set an argument (wrapped by Rc)
    ///   + allow move: softdrop
    ///   + board: blank
    ///   + height: 4 lines
    ///   + pattern: factorial of all shapes (like `*p7`)
    ///   + allows hold: yes
    pub fn default(rotation_system: Rc<T>) -> Self {
        Self {
            rotation_system,
            allow_move: AllowMove::Softdrop,
            clipped_board: ClippedBoard::try_new(Board64::blank(), 4).unwrap(),
            pattern: Rc::from(Pattern::try_from(vec![
                PatternElement::Factorial(ShapeCounter::one_of_each()),
            ]).unwrap()),
            allows_hold: true,
        }
    }

    // See `AllPcsFromPatternExecutorBinder::{try_new, execute}` for more details.
    pub fn try_execute(&self) -> Result<PcSolutions, AllPcsFromPatternExecutorCreationError> {
        let move_rules = MoveRules::new(self.rotation_system.as_ref(), self.allow_move);
        let executor = self.try_bind(&move_rules)?;
        Ok(executor.execute())
    }

    fn try_bind<'a>(&'a self, move_rules: &'a MoveRules<T>) -> Result<AllPcsFromPatternExecutor<T>, AllPcsFromPatternExecutorCreationError> {
        AllPcsFromPatternExecutor::try_new(
            move_rules,
            self.clipped_board,
            self.pattern.as_ref(),
            self.allows_hold,
        )
    }
}
