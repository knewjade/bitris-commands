use std::ops::Index;
use std::slice::Iter;
use bitris::prelude::*;
use itertools::Itertools;

use crate::internals::DynArray4;

#[derive(Clone, PartialEq, PartialOrd, Hash, Default, Debug)]
pub(crate) struct IndexedPieces<T> {
    pub pieces: Vec<(usize, T)>,
    pub height: usize,
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
    pub ys: DynArray4<usize>,
    pub locations: DynArray4<Location>,
}

#[derive(Clone, PartialEq, PartialOrd, Hash, Default, Debug)]
pub(crate) struct PredefinedPieceToBuild {
    pub piece: Piece,
    pub min_vertical_index: usize,
    pub vertical_relative_block: u64,
}

#[derive(PartialEq, PartialOrd, Hash, Debug)]
pub(crate) struct PredefinedPieceToAggregate {
    pub piece: Piece,
    pub ys: DynArray4<usize>,
    pub using_rows: Lines,
    pub deleted_rows: Lines,
    pub locations: DynArray4<Location>,
}

impl IndexedPieces<PredefinedPiece> {
    pub(crate) fn new(height: usize) -> Self {
        fn make(piece: Piece, height: usize) -> Vec<PredefinedPiece> {
            let piece_blocks = piece.to_piece_blocks();
            (0..height).combinations(piece_blocks.height as usize)
                .map(|mut ys| {
                    ys.sort();
                    DynArray4::try_from(ys).unwrap()
                })
                .map(|ys| {
                    let locations = piece_blocks.offsets
                        .into_iter()
                        .map(|offset| { offset - piece_blocks.bottom_left })
                        .map(|offset| { Location::new(offset.dx, ys[offset.dy as usize] as i32) })
                        .collect_vec()
                        .try_into()
                        .unwrap();
                    PredefinedPiece { piece, ys, locations }
                })
                .collect()
        }

        let pieces = Piece::all_vec()
            .into_iter()
            .filter(|piece| piece.canonical().is_none())
            .flat_map(|piece| make(piece, height))
            .enumerate()
            .collect();

        Self { pieces, height }
    }
}

impl From<&IndexedPieces<PredefinedPiece>> for IndexedPieces<PredefinedPieceToBuild> {
    fn from(value: &IndexedPieces<PredefinedPiece>) -> Self {
        fn make(predefined_piece: &PredefinedPiece, height: usize) -> PredefinedPieceToBuild {
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
                min_vertical_index,
                vertical_relative_block,
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


impl From<&IndexedPieces<PredefinedPiece>> for IndexedPieces<PredefinedPieceToAggregate> {
    fn from(value: &IndexedPieces<PredefinedPiece>) -> Self {
        fn make(predefined_piece: &PredefinedPiece) -> PredefinedPieceToAggregate {
            let deleted_rows = predefined_piece.ys.iter()
                .skip(1)
                .fold((predefined_piece.ys[0], 0u64), |(prev_y, merge), y| {
                    let a = (1u64 << y) - 1;
                    let b = (1u64 << (prev_y + 1)) - 1;
                    let i = a ^ b;
                    (*y, merge | (i))
                }).1;

            let using_rows = predefined_piece.ys.iter()
                .fold(0u64, |merge, y| {
                    merge | (1u64 << y)
                });

            PredefinedPieceToAggregate {
                piece: predefined_piece.piece,
                ys: predefined_piece.ys,
                locations: predefined_piece.locations,
                using_rows: Lines::new(using_rows),
                deleted_rows: Lines::new(deleted_rows),
            }
        }

        let pieces = value.pieces.iter()
            .map(|(index, predefined_piece)| {
                (*index, make(predefined_piece))
            })
            .collect_vec();

        Self { pieces, height: value.height }
    }
}
