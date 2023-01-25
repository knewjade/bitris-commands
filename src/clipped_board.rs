use bitris::prelude::{Board64, BoardOp};
use thiserror::Error;

/// Holds a board and height.
#[derive(Copy, Clone, PartialEq, PartialOrd, Hash, Debug)]
pub struct ClippedBoard {
    board: Board64,
    height: u32,
}

/// A collection of errors that occur when making clipped board.
#[derive(Error, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum ClippedBoardCreationError {
    #[error("Height must be greater than 0.")]
    HeightIsZero,
    #[error("Height must be the well top of the board or higher.")]
    NotHeightEnough,
    #[error("Board is filled already.")]
    BoardFilledAlready,
}

impl ClippedBoard {
    #[inline]
    pub(crate) fn new_unsafe(board: Board64, height: u32) -> Self {
        Self { board, height }
    }

    /// ```
    /// use std::str::FromStr;
    /// use bitris_commands::prelude::*;
    /// assert!(ClippedBoard::try_new(Board64::blank(), 4).is_ok());
    ///
    /// assert_eq!(
    ///     ClippedBoard::try_new(Board64::blank(), 0),
    ///     Err(ClippedBoardCreationError::HeightIsZero),
    /// );
    ///
    /// let board = Board64::from_str("
    ///     X.........
    ///     ..........
    /// ").unwrap();
    /// assert_eq!(
    ///     ClippedBoard::try_new(board, 1),
    ///     Err(ClippedBoardCreationError::NotHeightEnough),
    /// );
    ///
    /// let board = Board64::from_str("
    ///     XXXXXXXXXX
    ///     XXXXXXXXXX
    /// ").unwrap();
    /// assert_eq!(
    ///     ClippedBoard::try_new(board, 2),
    ///     Err(ClippedBoardCreationError::BoardFilledAlready),
    /// );
    /// ```
    #[inline]
    pub fn try_new(board: Board64, height: u32) -> Result<Self, ClippedBoardCreationError> {
        use ClippedBoardCreationError::*;
        if height <= 0 {
            return Err(HeightIsZero);
        }

        if height < board.well_top() {
            return Err(NotHeightEnough);
        }

        let mut board = board.clone();
        let lines_cleared = board.clear_lines();
        let height = height - lines_cleared.count();

        if height <= 0 {
            return Err(BoardFilledAlready);
        }

        Ok(Self { board, height })
    }

    /// Returns the count of spaces in the range.
    /// ```
    /// use bitris_commands::prelude::*;
    /// let clipped = ClippedBoard::try_new(Board64::blank(), 4).unwrap();
    /// assert_eq!(clipped.spaces(), 40);
    /// ```
    #[inline]
    pub fn spaces(&self) -> u32 {
        self.height * 10 - self.board.count_blocks()
    }

    #[inline]
    pub fn board(self) -> Board64 {
        self.board
    }

    #[inline]
    pub fn board_ref(&self) -> &Board64 {
        &self.board
    }

    #[inline]
    pub fn height(self) -> u32 {
        self.height
    }
}
