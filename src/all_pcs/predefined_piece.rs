use std::ops::Index;
use std::slice::Iter;

use bitris::prelude::*;
use itertools::Itertools;
use tinyvec::ArrayVec;

#[derive(Clone, PartialEq, PartialOrd, Hash, Default, Debug)]
pub(crate) struct IndexedPieces<T> {
    pub pieces: Vec<(usize, T)>,
    pub height: u8,
}

impl<T> IndexedPieces<T> {
    pub(crate) fn len(&self) -> usize {
        self.pieces.len()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.pieces.is_empty()
    }

    pub(crate) fn iter(&self) -> Iter<'_, (usize, T)> {
        self.pieces.iter()
    }
}

impl<T> Index<usize> for IndexedPieces<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.pieces[index].1
    }
}


#[derive(Clone, PartialEq, PartialOrd, Hash, Debug)]
pub(crate) struct PredefinedPiece {
    pub piece: Piece,
    pub ys: ArrayVec<[u8; 4]>,
    pub locations: [Location; 4],
    pub using_rows: Lines,
    pub intercepted_rows: Lines,
}

#[derive(Clone, PartialEq, PartialOrd, Hash, Default, Debug)]
pub(crate) struct PredefinedPieceToBuild {
    pub piece: Piece,
    pub min_y_at_x0: usize,
    pub relative_vertical_blocks: u64,
}

impl PredefinedPiece {
    // TODO 作成回数を少なくしたい
    pub(crate) fn to_aggregate(&self, lx: u8) -> PlacedPieceBlocks {
        PlacedPiece::new(self.piece, lx, self.ys).into()
    }
}

impl IndexedPieces<PredefinedPiece> {
    pub(crate) fn new(height: u8) -> Self {
        fn make(piece: Piece, height: u8) -> Vec<PredefinedPiece> {
            let piece_blocks = piece.to_piece_blocks();
            (0..height).combinations(piece_blocks.height as usize)
                .map(|ys| ys.into_iter().sorted().collect())
                .map(|ys: ArrayVec<[u8; 4]>| {
                    let locations = piece_blocks.offsets
                        .into_iter()
                        .map(|offset| { offset - piece_blocks.bottom_left })
                        .map(|offset| { Location::new(offset.dx, ys[offset.dy as usize] as i32) })
                        .collect_vec()
                        .try_into()
                        .unwrap();

                    let using_rows = ys.iter()
                        .fold(0u64, |merge, y| {
                            merge | (1u64 << y)
                        });

                    let intercepted_rows = ys.iter()
                        .skip(1)
                        .fold((ys[0], 0u64), |(prev_y, merge), y| {
                            let a = (1u64 << y) - 1;
                            let b = (1u64 << (prev_y + 1)) - 1;
                            let i = a ^ b;
                            (*y, merge | (i))
                        })
                        .1;

                    PredefinedPiece {
                        piece,
                        ys,
                        locations,
                        using_rows: Lines::new(using_rows),
                        intercepted_rows: Lines::new(intercepted_rows),
                    }
                })
                .collect()
        }

        let pieces = Piece::all_iter()
            .filter(|piece| piece.canonical().is_none())
            .flat_map(|piece| make(piece, height))
            .enumerate()
            .collect();

        Self { pieces, height }
    }
}

impl From<&IndexedPieces<PredefinedPiece>> for IndexedPieces<PredefinedPieceToBuild> {
    fn from(value: &IndexedPieces<PredefinedPiece>) -> Self {
        fn make(predefined_piece: &PredefinedPiece, height: u8) -> PredefinedPieceToBuild {
            let min_vertical_index = predefined_piece.locations
                .iter()
                .filter(|location| { location.x == 0 })
                .map(|location| location.y as usize)
                .min()
                .unwrap();

            let height = height as i32;
            let vertical_relative_block = predefined_piece.locations
                .iter()
                .fold(0u64, |prev, location| {
                    let shift = location.x * height + location.y - min_vertical_index as i32;
                    prev | (1u64 << shift)
                });

            PredefinedPieceToBuild {
                piece: predefined_piece.piece,
                min_y_at_x0: min_vertical_index,
                relative_vertical_blocks: vertical_relative_block,
            }
        }

        let pieces = value.pieces.iter()
            .map(|(index, predefined_piece)| {
                (*index, make(predefined_piece, value.height))
            })
            .collect_vec();

        Self { pieces, height: value.height }
    }
}
