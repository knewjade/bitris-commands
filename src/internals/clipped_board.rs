use bitris::{Board64, BoardOp};

/// Holds a board and height.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub(crate) struct ClippedBoard {
    pub board: Board64,
    pub height: u32,
}

impl ClippedBoard {
    #[inline]
    pub fn new(board: Board64, height: u32) -> Self {
        assert!(board.well_top() <= height, "Height must be the well top of the board or higher.");
        Self { board, height }
    }

    #[inline]
    pub fn spaces(&self) -> u32 {
        assert!(self.board.well_top() <= self.height);
        self.height * 10 - self.board.count_blocks()
    }
}


#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use bitris::Board64;

    use crate::internals::ClippedBoard;

    #[test]
    fn clipped_board() {
        let clipped = ClippedBoard::new(Board64::blank(), 4);
        assert_eq!(clipped.board, Board64::blank());
        assert_eq!(clipped.height, 4);
        assert_eq!(clipped.spaces(), 40);
    }

    #[test]
    #[should_panic]
    fn clipped_board_assert() {
        let board = Board64::from_str("
            #.........
            ..........
            ..........
            ..........
            ..........
        ").unwrap();
        ClippedBoard::new(board, 4);
    }
}
