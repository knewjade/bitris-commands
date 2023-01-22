use std::ops::Range;

use bitris::prelude::*;
use fxhash::FxHashMap;
use itertools::Itertools;
use tap::Conv;

use crate::{ClippedBoard, ShapeCounter};
use crate::all_pcs::{Aggregator, IndexedPieces, Nodes, PredefinedPiece, PredefinedPieceToBuild};

pub(crate) struct Builder {
    clipped_board: ClippedBoard,
    indexed_pieces: IndexedPieces<PredefinedPiece>,
    available: ShapeCounter,
    width: usize,
}

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
        assert!(!indexed_pieces.is_empty());
        Self { clipped_board, indexed_pieces, available, width }
    }

    pub(crate) fn to_aggregator(self, spawn_position: BlPosition) -> Aggregator {
        let nodes = self.build();
        Aggregator::new(self.clipped_board, self.indexed_pieces, self.width, nodes, spawn_position)
    }

    fn build(&self) -> Nodes {
        struct Buffer {
            nodes: Nodes,
            frontiers: Vec<Frontier>,
            hash_map: FxHashMap::<Frontier, usize>,
        }
        impl Buffer {}

        let mut buffer = Buffer {
            nodes: Nodes::empty(),
            frontiers: vec![Frontier { board: 0, available: self.available }],
            hash_map: FxHashMap::<Frontier, usize>::default(),
        };

        impl Buffer {
            fn setup_at_each_block(&mut self) {
                self.hash_map.clear();
            }

            // Advance current search ranges by one block.
            fn increment_all(&mut self) {
                let search_range = self.next_candidates_range();
                for frontier in &mut self.frontiers[search_range] {
                    frontier.board >>= 1;
                }
            }

            // Make current search ranges complete.
            fn complete_all(&mut self) {
                for _ in self.next_candidates_range() {
                    self.nodes.complete();
                }
            }

            // `node.indexes[frontier_index]` is made from `frontier[frontier_index]` and puts a piece to `node.items`.
            fn put_one_piece(
                &mut self,
                frontier_index: usize,
                current_bits: u64,
                predefines: &Vec<(usize, &PredefinedPieceToBuild)>,
            ) {
                let available = self.frontiers[frontier_index].available;
                let head_item_node_id = self.nodes.next_item_id();

                let mut item_size = 0usize;
                for (mino_index, mino) in predefines {
                    if available[mino.piece.shape] == 0 {
                        continue;
                    }

                    let mino_index = *mino_index as u32;
                    if (current_bits & mino.vertical_relative_block) == 0 {
                        let next_block = current_bits | mino.vertical_relative_block;

                        let next_frontier = Frontier {
                            board: next_block >> 1,
                            available: available - mino.piece.shape,
                        };

                        let next_index_id = self.get_next_index_id(next_frontier);
                        self.nodes.push_item(mino_index, next_index_id);
                        item_size += 1;
                    }
                }

                debug_assert_eq!(frontier_index, self.nodes.next_index_id());
                debug_assert_eq!(head_item_node_id + item_size, self.nodes.next_item_id());

                if 0 < item_size {
                    self.nodes.go_to_items(head_item_node_id, item_size as u32);
                } else {
                    self.nodes.abort();
                }
            }

            // Do not place a piece.
            fn skip_one_block(
                &mut self,
                frontier_index: usize,
                current_bits: u64,
            ) {
                let next_frontier = Frontier {
                    board: current_bits >> 1,
                    available: self.frontiers[frontier_index].available,
                };
                let next_index_id = self.get_next_index_id(next_frontier);
                self.nodes.skip_to_next_index(next_index_id);
            }

            fn abort_one(&mut self) {
                self.nodes.abort();
            }

            // Get the next index id.
            // If the same state already exists, it's retrieved from the cache.
            fn get_next_index_id(&mut self, next_frontier: Frontier) -> usize {
                if let Some(next_index_id) = self.hash_map.get(&next_frontier) {
                    *next_index_id
                } else {
                    let next_index_id = self.frontiers.len();
                    self.hash_map.insert(next_frontier, next_index_id);
                    self.frontiers.push(next_frontier);
                    next_index_id
                }
            }

            // Get current all search ranges.
            fn next_candidates_range(&self) -> Range<usize> {
                let start = self.nodes.next_index_id();
                let end = self.frontiers.len();
                start..end
            }

            // Returns true if the current search range is still remaining.
            fn has_candidates(&self) -> bool {
                !self.next_candidates_range().is_empty()
            }
        }

        let indexed_pieces = (&self.indexed_pieces).conv::<IndexedPieces<PredefinedPieceToBuild>>();
        let (height, board) = (self.clipped_board.height() as usize, self.clipped_board.board());

        for lx in 0..self.width {
            for y in 0..height {
                if board.is_occupied_at(xy(lx as i32, y as i32)) {
                    buffer.increment_all();
                    continue;
                }

                // In scanning order, take out blocks up to four columns ahead of the current.
                // (Take out the area reached by the I-piece.)
                // Include wall blocks to filter the predefines that can be placed.
                let board_mask = {
                    let col4 = {
                        let mut mask = 0u64;
                        let col_mask = (1u64 << height) - 1;
                        for x in 0..4 {
                            let col = board.cols.get(lx + x).unwrap_or(&u64::MAX);
                            mask |= (col & col_mask) << (height * x);
                        }
                        mask
                    };

                    // Match the current location to the LSB.
                    let mask = col4 >> y;

                    // The current location has no block always.
                    debug_assert_eq!(mask & 1, 0);

                    mask
                };

                let predefines = indexed_pieces.iter()
                    .filter(|(_, mino)| mino.min_vertical_index == y)
                    .filter(|(_, mino)| (board_mask & mino.vertical_relative_block) == 0)
                    .map(|(mino_index, mino)| (mino_index * self.width + lx, mino))
                    .collect_vec();

                buffer.setup_at_each_block();

                for frontier_index in buffer.next_candidates_range() {
                    let current_bits = buffer.frontiers[frontier_index].board;
                    if current_bits & 1 == 0 {
                        if !predefines.is_empty() {
                            buffer.put_one_piece(frontier_index, current_bits, &predefines);
                        } else {
                            buffer.abort_one();
                        }
                    } else {
                        debug_assert_eq!(frontier_index, buffer.nodes.next_index_id());
                        buffer.skip_one_block(frontier_index, current_bits);
                    }
                }

                if !buffer.has_candidates() {
                    return Nodes::empty();
                }
            }
        }

        buffer.complete_all();
        debug_assert!(!buffer.has_candidates());

        buffer.nodes
    }
}
