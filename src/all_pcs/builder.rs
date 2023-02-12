use std::ops::Range;

use bitris::prelude::*;
use fxhash::FxHashMap;
use itertools::Itertools;

use crate::{ClippedBoard, ShapeCounter};
use crate::all_pcs::{Aggregator, IndexId, Nodes};

#[derive(Clone)]
struct Predefine {
    placed_piece: PlacedPiece,
    min_y_on_left_x: usize,
    relative_vertical_blocks: u64,
}

impl Predefine {
    fn new(placed_piece: PlacedPiece, height: u8) -> Self {
        let lx = placed_piece.lx as i32;

        let locations = placed_piece.locations();
        let min_y_on_left_x = locations.into_iter()
            .filter(|location| location.x == lx)
            .map(|location| location.y as usize)
            .min()
            .unwrap();

        let height = height as i32;
        let relative_vertical_blocks = locations.iter()
            .fold(0u64, |prev, location| {
                let shift = (location.x - lx) * height + location.y - min_y_on_left_x as i32;
                prev | (1u64 << shift)
            });

        Self { placed_piece, min_y_on_left_x, relative_vertical_blocks }
    }
}

pub(crate) struct Builder {
    clipped_board: ClippedBoard,
    placed_pieces: Vec<PlacedPiece>,
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
        placed_pieces: Vec<PlacedPiece>,
        available: ShapeCounter,
        width: usize,
    ) -> Self {
        assert!(!placed_pieces.is_empty());
        Self { clipped_board, placed_pieces, available, width }
    }

    pub(crate) fn new_and_make_placed_pieces(
        clipped_board: ClippedBoard,
        available: ShapeCounter,
        width: usize,
    ) -> Self {
        let placed_pieces: Vec<PlacedPiece> = PlacedPiece::make_canonical_on_board_iter(clipped_board.board(), clipped_board.height() as usize)
            .filter(|placed_piece| 0 < available[placed_piece.piece.shape])
            .collect();
        Self::new(clipped_board, placed_pieces, available, width)
    }

    pub(crate) fn to_aggregator(self, spawn_position: BlPosition) -> Aggregator {
        let nodes = self.build();
        Aggregator::new(self.clipped_board, self.placed_pieces, nodes, spawn_position)
    }

    fn build(&self) -> Nodes {
        struct Buffer {
            nodes: Nodes,
            frontiers: Vec<Frontier>,
            hash_map: FxHashMap<Frontier, IndexId>,
        }
        impl Buffer {}

        let mut buffer = Buffer {
            nodes: Nodes::empty(),
            frontiers: vec![Frontier { board: 0, available: self.available }],
            hash_map: FxHashMap::<Frontier, IndexId>::default(),
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
                predefines: &Vec<Predefine>,
            ) {
                let available = self.frontiers[frontier_index].available;
                let head_item_node_id = self.nodes.next_item_id();

                let mut item_size = 0usize;
                for predefine in predefines.into_iter() {
                    let shape = predefine.placed_piece.piece.shape;
                    if available[shape] == 0 {
                        continue;
                    }

                    let relative_vertical_blocks = predefine.relative_vertical_blocks;
                    if (current_bits & relative_vertical_blocks) == 0 {
                        let next_block = current_bits | relative_vertical_blocks;

                        let next_frontier = Frontier {
                            board: next_block >> 1,
                            available: available - shape,
                        };

                        let next_index_id = self.get_next_index_id(next_frontier);
                        self.nodes.push_item(predefine.placed_piece, next_index_id);
                        item_size += 1;
                    }
                }

                debug_assert_eq!(frontier_index, self.nodes.next_index_id().id);
                debug_assert_eq!(head_item_node_id.id + item_size, self.nodes.next_item_id().id);

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
            fn get_next_index_id(&mut self, next_frontier: Frontier) -> IndexId {
                if let Some(next_index_id) = self.hash_map.get(&next_frontier) {
                    *next_index_id
                } else {
                    let next_index_id = IndexId::new(self.frontiers.len());
                    self.hash_map.insert(next_frontier, next_index_id);
                    self.frontiers.push(next_frontier);
                    next_index_id
                }
            }

            // Get current all search ranges.
            fn next_candidates_range(&self) -> Range<usize> {
                let start = self.nodes.next_index_id().id;
                let end = self.frontiers.len();
                start..end
            }

            // Returns true if the current search range is still remaining.
            fn has_candidates(&self) -> bool {
                !self.next_candidates_range().is_empty()
            }
        }

        let height = self.clipped_board.height() as usize;
        let board = self.clipped_board.board();

        let all_predefines = self.placed_pieces.iter()
            .map(|&placed_piece| Predefine::new(placed_piece, height as u8))
            .collect_vec();

        for lx in 0..self.width {
            for y in 0..height {
                if board.is_occupied_at(xy(lx as i32, y as i32)) {
                    buffer.increment_all();
                    continue;
                }

                let predefines: Vec<Predefine> = all_predefines.iter()
                    .filter(|&predefined| predefined.placed_piece.lx == lx as u8)
                    .filter(|&predefined| predefined.min_y_on_left_x == y)
                    .map(|it| it.clone())
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
                        debug_assert_eq!(frontier_index, buffer.nodes.next_index_id().id);
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
