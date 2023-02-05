use bitris::prelude::*;

use crate::ClippedBoard;

// TODO
pub struct PcSolutions {
    clipped_board: ClippedBoard,
    placed_pieces: Vec<Vec<PlacedPiece>>,
}

impl PcSolutions {
    #[inline]
    pub fn new(clipped_board: ClippedBoard, placed_pieces: Vec<Vec<PlacedPiece>>) -> Self {
        Self { clipped_board, placed_pieces }
    }

    #[inline]
    pub fn empty(clipped_board: ClippedBoard) -> Self {
        Self { clipped_board, placed_pieces: Vec::new() }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.placed_pieces.len()
    }
}
