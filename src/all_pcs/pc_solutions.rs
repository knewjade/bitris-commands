use bitris::prelude::*;

use crate::ClippedBoard;

#[derive(Clone, PartialEq, PartialOrd, Hash, Debug)]
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

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.placed_pieces.is_empty()
    }

    #[inline]
    pub fn first(&self) -> Option<&Vec<PlacedPiece>> {
        self.placed_pieces.first()
    }
}
