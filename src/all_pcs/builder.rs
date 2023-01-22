use bitris::prelude::*;
use fxhash::FxHashMap;
use itertools::Itertools;
use tap::Conv;

use crate::all_pcs::{Aggregator, IndexedPieces, Nodes, PredefinedPiece, PredefinedPieceToBuild};
use crate::{ClippedBoard, ShapeCounter};

pub(crate) struct Builder {
    clipped_board: ClippedBoard,
    indexed_pieces: IndexedPieces<PredefinedPiece>,
    available: ShapeCounter,
    width: usize,
}

// TODO
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default, Debug)]
struct Frontier {
    board: u64,
    available: ShapeCounter,
}

impl Builder {
    pub(crate) fn new(
        clipped_board: ClippedBoard,
        indexed_pieces: IndexedPieces<PredefinedPiece>,
        available: ShapeCounter,
        width: usize,
    ) -> Self {
        Self { clipped_board, indexed_pieces, available, width }
    }

    pub(crate) fn to_aggregator(self, spawn_position: BlPosition) -> Aggregator {
        let nodes = self.build();
        Aggregator::new(self.clipped_board, self.indexed_pieces, self.width, nodes, spawn_position)
    }

    fn build(&self) -> Nodes {
        assert!(!self.indexed_pieces.is_empty());

        let predefines = (&self.indexed_pieces).conv::<IndexedPieces<PredefinedPieceToBuild>>();

        let height = self.clipped_board.height() as usize;

        let mut nodes = Nodes::empty();
        let mut frontiers = Vec::<Frontier>::new();

        frontiers.push(Frontier { board: 0, available: self.available });

        let mut hash_map = FxHashMap::<Frontier, usize>::default();

        for lx in 0..self.width {
            for y in 0..height {
                if self.clipped_board.board().is_occupied_at(xy(lx as i32, y as i32)) {
                    // TODO あっている？
                    for tail in (nodes.index_serial())..(frontiers.len()) {
                        frontiers[tail].board >>= 1; // TODO sliceで置き換えられる?
                    }
                    continue;
                }

                let minos = predefines.iter()
                    .filter(|(_, mino)| mino.min_vertical_index == y)
                    .filter(|(_, mino)| lx as u32 + mino.piece.width() <= self.width as u32)
                    .map(|(mino_index, mino)| (mino_index * self.width + lx, mino))
                    .collect_vec();

                if minos.is_empty() {
                    continue;
                }

                hash_map.clear();

                let board_mask = {
                    let board = self.clipped_board.board();
                    let mut m = 0;
                    let mut x = lx as i32;
                    let mut y = y as i32;
                    for shift in 0..(3 * height + 1) {
                        if board.is_occupied_at(xy(x, y)) {
                            m |= 1 << shift;
                        }
                        y += 1;
                        if y == height as i32 {
                            x += 1;
                            y = 0;
                            if x == self.width as i32 {
                                break;
                            }
                        }
                    }
                    assert_eq!(m & 1, 0);
                    m
                };

                // Number of remaining search blocks, including the block at `index`
                let rest: usize = height * (self.width - lx - 1) + (height - y);
                let fill_block_mask = if rest <= 3 * height + 1 {
                    // All remaining blocks are filled, including the block at `index`
                    (1u64 << rest) - 1
                } else {
                    u64::MAX
                };

                debug_assert!(nodes.index_serial() < frontiers.len());

                for tail in (nodes.index_serial())..(frontiers.len()) {
                    let frontier = &frontiers[tail];
                    let available = frontier.available;
                    let current_bits = frontier.board | board_mask; // TODO sliceで置き換えられる?

                    if current_bits & 1 == 0 {
                        // No block at `index`
                        let start_item_node_index = nodes.item_serial();
                        let mut item_size = 0usize;

                        let available = frontier.available;
                        for (mino_index, mino) in &minos {
                            if available[mino.piece.shape] == 0 {
                                continue;
                            }

                            let mino_index = *mino_index as u32;
                            if (current_bits & mino.vertical_relative_block) == 0 {
                                // Can put mino

                                item_size += 1;
                                let next_block = current_bits | mino.vertical_relative_block;

                                if next_block == fill_block_mask {
                                    // Filled all
                                    nodes.complete(mino_index);
                                    continue;
                                }

                                let hash_key = next_block >> 1;
                                let next_available = available - mino.piece.shape;
                                let next_frontier = Frontier { board: hash_key, available: next_available };

                                if let Some(next_index_id) = hash_map.get(&next_frontier) {
                                    nodes.put(mino_index, *next_index_id);
                                } else {
                                    // Found new state

                                    // [Future reference] If `head` exceeds the `frontiers` size and rotate index, becomes nextIndexId != head
                                    let next_index_id = frontiers.len();
                                    hash_map.insert(next_frontier, next_index_id);
                                    frontiers.push(next_frontier);

                                    nodes.put(mino_index, next_index_id);
                                }
                            }
                        }

                        // frontier[i]にあるものから、index[i]がつくられる
                        // つまり、開始時はfrontier[0]に初期状態をおき、tail, boun
                        assert_eq!(tail, nodes.index_serial());
                        nodes.jump(start_item_node_index as u32, item_size as u32);
                    } else {
                        // Filled block at `index`
                        let hash_key = current_bits >> 1;
                        let next_frontier = Frontier { board: hash_key, available };

                        if let Some(next_index_id) = hash_map.get(&next_frontier) {
                            assert_eq!(tail, nodes.index_serial());
                            nodes.skip(*next_index_id as u32);
                        } else {
                            // Found new state

                            // [Future reference] If `head` exceeds the `frontiers` size and rotate index, becomes nextIndexId != head
                            let next_index_id = frontiers.len();
                            hash_map.insert(next_frontier, next_index_id);
                            frontiers.push(next_frontier);

                            assert_eq!(tail, nodes.index_serial());
                            nodes.skip(next_index_id as u32);
                        }
                    }
                }

                if nodes.index_serial() == frontiers.len() {
                    return Nodes::empty();
                }
            }
        }

        for _ in (nodes.index_serial())..(frontiers.len()) {
            nodes.complete2();
        }

        assert_eq!(nodes.index_serial(), frontiers.len());

        nodes
    }
}
