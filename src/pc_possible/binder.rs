use std::rc::Rc;

use bitris::prelude::*;
use bitris::srs::SrsKickTable;
use itertools::Itertools;
use thiserror::Error;

use crate::{ClippedBoard, Pattern, PatternCreationError, PatternElement, ShapeOrder};
use crate::pc_possible::{PcPossibleBulkExecutor, PcPossibleExecutorBulkCreationError};

/// A collection of errors that occur when making the executor.
#[derive(Error, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum PcPossibleExecutorCreationError {
    #[error("Unexpected the count of board spaces.")]
    UnexpectedBoardSpaces,
    #[error("The order is too short to take a PC.")]
    ShortOrderDimension,
    #[error("Board height exceeds the upper limit. Up to 56 are supported.")]
    BoardIsTooHigh,
}

/// The binder to hold and tie settings for `PcPossibleExecutor`.
#[derive(Clone, PartialEq, PartialOrd, Hash, Debug)]
pub struct PcPossibleExecutorBinder<T: RotationSystem> {
    pub rotation_system: Rc<T>,
    pub allow_move: AllowMove,
    pub clipped_board: ClippedBoard,
    pub shape_order: Rc<ShapeOrder>,
    pub allows_hold: bool,
}

impl PcPossibleExecutorBinder<SrsKickTable> {
    /// Making the executor with SRS. See `PcPossibleExecutorBinder::default()` for more details.
    pub fn srs() -> Self {
        PcPossibleExecutorBinder::default(Rc::from(SrsKickTable))
    }
}

impl<T: RotationSystem> PcPossibleExecutorBinder<T> {
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

    // See `PcPossibleBulkExecutor::{try_new, execute}` for more details.
    pub fn try_execute(&self) -> Result<bool, PcPossibleExecutorCreationError> {
        use PcPossibleExecutorBulkCreationError as FromError;
        use PcPossibleExecutorCreationError as ToError;

        let move_rules = MoveRules::new(self.rotation_system.as_ref(), self.allow_move);
        let pattern = match Pattern::try_from(
            self.shape_order.shapes().iter().map(|&shape| PatternElement::One(shape)).collect_vec()
        ) {
            Ok(pattern) => pattern,
            Err(error) => return match error {
                PatternCreationError::NoShapeSequences => Err(ToError::ShortOrderDimension),
                PatternCreationError::ContainsInvalidPermutation => panic!("Unreachable assumption"),
            },
        };

        self.try_bind(&move_rules, &pattern)
            .map(|executor| {
                executor.execute_single()
            })
            .map_err(|error| {
                match error {
                    FromError::UnexpectedBoardSpaces => ToError::UnexpectedBoardSpaces,
                    FromError::ShortPatternDimension => ToError::ShortOrderDimension,
                    FromError::BoardIsTooHigh => ToError::BoardIsTooHigh,
                }
            })
    }

    fn try_bind<'a>(&'a self, move_rules: &'a MoveRules<T>, pattern: &'a Pattern) -> Result<PcPossibleBulkExecutor<T>, PcPossibleExecutorBulkCreationError> {
        PcPossibleBulkExecutor::try_new(
            move_rules,
            self.clipped_board,
            pattern,
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
    use crate::pc_possible::{PcPossibleExecutorBinder, PcPossibleExecutorCreationError};

    #[test]
    fn reuse() {
        use Shape::*;

        let mut binder = PcPossibleExecutorBinder::srs();
        let board = Board64::from_str("
            ..........
            ....####..
            ....######
            ....######
        ").unwrap();
        binder.clipped_board = ClippedBoard::try_new(board, 4).unwrap();

        binder.shape_order = Rc::new(ShapeOrder::new(vec![
            I, O, T, Z, S, J, L,
        ]));
        assert!(binder.try_execute().unwrap());

        binder.shape_order = Rc::new(ShapeOrder::new(vec![
            Z, S, I, O, L, J, T,
        ]));
        assert!(!binder.try_execute().unwrap());
    }

    #[test]
    fn error() {
        use Shape::*;
        use PcPossibleExecutorCreationError::*;

        let mut binder = PcPossibleExecutorBinder::srs();
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

        assert_eq!(binder.try_execute().unwrap_err(), ShortOrderDimension);

        binder.shape_order = Rc::new(ShapeOrder::new(vec![Z]));

        assert_eq!(binder.try_execute().unwrap_err(), ShortOrderDimension);
    }
}
