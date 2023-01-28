use std::rc::Rc;

use bitris::prelude::*;
use bitris::srs::SrsKickTable;

use crate::{ClippedBoard, Pattern, PatternElement, ShapeCounter};
use crate::pc_possible::{ExecuteInstruction, PcPossibleBulkExecutor, PcPossibleExecutorBulkCreationError, PcResults};

/// The binder to hold and tie settings for `PcPossibleBulkExecutor`.
#[derive(Clone, PartialEq, PartialOrd, Hash, Debug)]
pub struct PcPossibleBulkExecutorBinder<T: RotationSystem> {
    pub rotation_system: Rc<T>,
    pub allow_move: AllowMove,
    pub clipped_board: ClippedBoard,
    pub pattern: Rc<Pattern>,
    pub allows_hold: bool,
}

impl PcPossibleBulkExecutorBinder<SrsKickTable> {
    /// Making the executor with SRS. See `PcPossibleBulkExecutorBinder::default()` for more details.
    pub fn srs() -> Self {
        PcPossibleBulkExecutorBinder::default(Rc::from(SrsKickTable))
    }
}

impl<T: RotationSystem> PcPossibleBulkExecutorBinder<T> {
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

    // See `PcPossibleBulkExecutor::{try_new, execute}` for more details.
    pub fn try_execute(&self) -> Result<PcResults, PcPossibleExecutorBulkCreationError> {
        let move_rules = MoveRules::new(self.rotation_system.as_ref(), self.allow_move);
        let executor = self.try_bind(move_rules)?;
        Ok(executor.execute())
    }

    // See `PcPossibleBulkExecutor::{try_new, execute_with_early_stopping}` for more details.
    pub fn try_execute_with_early_stopping(&self, early_stopping: impl Fn(&PcResults) -> ExecuteInstruction) -> Result<PcResults, PcPossibleExecutorBulkCreationError> {
        let move_rules = MoveRules::new(self.rotation_system.as_ref(), self.allow_move);
        let executor = self.try_bind(move_rules)?;
        Ok(executor.execute_with_early_stopping(early_stopping))
    }

    fn try_bind<'a>(&'a self, move_rules: MoveRules<'a, T>) -> Result<PcPossibleBulkExecutor<T>, PcPossibleExecutorBulkCreationError> {
        PcPossibleBulkExecutor::try_new(
            move_rules,
            self.clipped_board,
            self.pattern.as_ref(),
            self.allows_hold,
        )
    }
}


#[cfg(test)]
mod tests {
    use std::rc::Rc;
    use std::str::FromStr;

    use bitris::prelude::*;

    use crate::{ClippedBoard, Pattern, PatternElement, ShapeCounter};
    use crate::pc_possible::PcPossibleBulkExecutorBinder;

    #[test]
    fn reuse() {
        use PatternElement::*;

        let mut binder = PcPossibleBulkExecutorBinder::srs();
        let board = Board64::from_str("
            ####......
            ####......
            ####......
            ####......
        ").unwrap();
        binder.clipped_board = ClippedBoard::try_new(board, 4).unwrap();

        let result = binder.try_execute().unwrap();
        assert_eq!(result.count_succeed(), 5040);

        let mut binder = binder.clone();
        let board = Board64::from_str("
            ####....##
            ###.....##
            ##......##
            ###.....##
        ").unwrap();
        binder.clipped_board = ClippedBoard::try_new(board, 4).unwrap();
        binder.pattern = Rc::from(Pattern::try_from(vec![
            Permutation(ShapeCounter::one_of_each(), 6),
        ]).unwrap());
        let result = binder.try_execute().unwrap();
        assert_eq!(result.count_succeed(), 4088);
    }
}
