use bitris::prelude::*;
use fxhash::{FxHashMap, FxHashSet};
use itertools::Itertools;

use crate::{ClippedBoard, ShapeCounter};
use crate::all_pcs::{IndexedPieces, IndexId, IndexNode, ItemId, Nodes, PieceKey, PlacedPieceBlocks, PredefinedPiece};

fn can_stack(
    clipped_board: ClippedBoard,
    spawn_position: BlPosition,
    placed_vec: &Vec<&PlacedPieceBlocks>,
) -> bool {
    struct Runner<'a> {
        predefined_pieces: &'a Vec<&'a PlacedPieceBlocks>,
        spawn_placements: Vec<BlPlacement>,
        move_rules: &'a MoveRules<'a, SrsKickTable>,
    }

    impl Runner<'_> {
        fn run(
            &self,
            board: Board64,
            remaining: u64,
            visited: &mut FxHashSet<u64>,
        ) -> bool {
            let (board_after_clearing, lines_cleared) = {
                let mut board_after_clearing = board.clone();
                let lines_cleared = board_after_clearing.clear_lines();
                (board_after_clearing, lines_cleared)
            };

            let mut candidate_shapes = remaining;
            while candidate_shapes != 0 {
                let bit_shape = candidate_shapes & (-(candidate_shapes as i64)) as u64;
                candidate_shapes -= bit_shape;

                let (current, spawn) = {
                    let index = bit_shape.trailing_zeros() as usize;
                    (&self.predefined_pieces[index], self.spawn_placements[index])
                };

                // ミノを置くためのラインがすべて削除されている
                if (current.intercepted_rows & lines_cleared) == current.intercepted_rows {
                    let piece_by = current.ys[0];

                    let lines_cleared_below_piece = lines_cleared & Lines::filled_up_to(piece_by);
                    let by = piece_by as i32 - lines_cleared_below_piece.count() as i32;

                    let placement = current.piece.with(bl(current.lx as i32, by));
                    if self.move_rules.can_reach(placement, board_after_clearing, spawn) {
                        let next_remaining = remaining - bit_shape;
                        if next_remaining == 0 {
                            return true;
                        }

                        if !visited.insert(next_remaining) {
                            continue;
                        }

                        let next_board = {
                            let mut next_board = board.clone();
                            next_board.merge(&current.board);
                            next_board
                        };

                        if self.run(next_board, next_remaining, visited) {
                            return true;
                        }
                    }
                }
            }

            return false;
        }
    }

    let mut hash_set = FxHashSet::<u64>::default();
    hash_set.reserve((1u64 << placed_vec.len()) as usize);

    let move_rules = MoveRules::srs(AllowMove::Softdrop);

    // Spawn on top of the well to avoid getting stuck.
    let spawn_placements = placed_vec.iter()
        .map(|result| result.piece.shape.with(Orientation::North).with(spawn_position))
        .collect();

    let buffer = Runner { predefined_pieces: placed_vec, spawn_placements, move_rules: &move_rules };

    buffer.run(clipped_board.board(), (1u64 << placed_vec.len()) - 1, &mut hash_set)
}

trait PcAggregationChecker {
    fn checks(&self, placed_vec: &Vec<&PlacedPieceBlocks>) -> bool;
}

pub(crate) struct Aggregator {
    clipped_board: ClippedBoard,
    map_placed_piece_blocks: FxHashMap<PieceKey, PlacedPieceBlocks>,
    width: usize,
    nodes: Nodes,
    spawn_position: BlPosition,
}

impl Aggregator {
    pub(crate) fn new(
        clipped_board: ClippedBoard,
        indexed_pieces: IndexedPieces<PredefinedPiece>,
        width: usize,
        nodes: Nodes,
        spawn_position: BlPosition,
    ) -> Self {
        let piece_keys = nodes.items.iter()
            .map(|item| item.piece_key)
            .sorted()
            .dedup()
            .collect_vec();

        let map_placed_piece_blocks = piece_keys.iter()
            .fold(FxHashMap::default(), |mut map, &piece_key| {
                let predefined_piece = &indexed_pieces[piece_key.piece_index(width)];
                let lx = piece_key.lx(width);
                map.insert(piece_key, predefined_piece.to_aggregate(lx));
                map
            });

        Self { clipped_board, map_placed_piece_blocks, nodes, width, spawn_position }
    }

    pub(crate) fn aggregate(&self) -> u64 {
        if self.nodes.indexes.is_empty() {
            return 0;
        }

        struct PcAggregationCheckerImpl {
            clipped_board: ClippedBoard,
            spawn_position: BlPosition,
        }

        impl PcAggregationChecker for PcAggregationCheckerImpl {
            fn checks(&self, placed_vec: &Vec<&PlacedPieceBlocks>) -> bool {
                can_stack(self.clipped_board, self.spawn_position, &placed_vec)
            }
        }

        let checker = PcAggregationCheckerImpl {
            clipped_board: self.clipped_board,
            spawn_position: self.spawn_position,
        };

        let mut filled = vec![Lines::blank(); self.clipped_board.height() as usize];
        let mut results = Vec::new();
        self.aggregate_recursively(self.nodes.head_index_id().unwrap(), filled, &mut results, &checker)
    }

    pub(crate) fn aggregate_with_shape_counters(&self, shape_counters: &Vec<ShapeCounter>) -> u64 {
        if self.nodes.indexes.is_empty() {
            return 0;
        }

        struct PcAggregationCheckerImpl<'a> {
            shape_counters: &'a Vec<ShapeCounter>,
            clipped_board: ClippedBoard,
            spawn_position: BlPosition,
        }

        impl PcAggregationChecker for PcAggregationCheckerImpl<'_> {
            fn checks(&self, placed_vec: &Vec<&PlacedPieceBlocks>) -> bool {
                let succeed = {
                    let shape_counter: ShapeCounter = placed_vec.iter()
                        .map(|it| it.piece.shape)
                        .collect();

                    self.shape_counters.iter().any(|it| it.contains_all(&shape_counter))
                };
                if !succeed {
                    return false;
                }

                let succeed = can_stack(self.clipped_board, self.spawn_position, &placed_vec);
                if !succeed {
                    return false;
                }

                true
            }
        }

        let checker = PcAggregationCheckerImpl {
            shape_counters,
            clipped_board: self.clipped_board,
            spawn_position: self.spawn_position,
        };

        let mut filled = vec![Lines::blank(); self.clipped_board.height() as usize];
        let mut results = Vec::new();
        self.aggregate_recursively(self.nodes.head_index_id().unwrap(), filled, &mut results, &checker)
    }

    fn aggregate_recursively(
        &self,
        index_id: IndexId,
        filled: Vec<Lines>,
        placed: &mut Vec<PieceKey>,
        checker: &impl PcAggregationChecker,
    ) -> u64 {
        match self.nodes.index(index_id).unwrap() {
            IndexNode::ToItem(next_item_id, item_length) => {
                let item_ids = (next_item_id.id..(next_item_id.id + *item_length as usize))
                    .map(|item_id| self.nodes.item(ItemId::new(item_id)).unwrap());

                let mut success = 0u64;
                for item in item_ids {
                    let predefine = &self.map_placed_piece_blocks[&item.piece_key];

                    let filled_rows = predefine.ys.iter()
                        .fold(Lines::blank(), |prev, &y| prev | filled[y]);

                    // 注目しているミノを置く前の絶対に揃えられないラインが削除されていないといけないか
                    if !(filled_rows & predefine.intercepted_rows).is_blank() {
                        // 使っている
                        continue;
                    }

                    let next_filled = {
                        let mut filled = filled.clone();

                        // 揃えられないラインを更新
                        // temp = [y]ラインにブロックがあると、使用できないライン一覧が記録されている
                        // ミノXの[y]がdeletedKeyに指定されていると、Xのブロックのあるラインは先に揃えられなくなる
                        let mut rows = predefine.intercepted_rows.key;
                        while rows != 0 {
                            let bit_row = rows & (-(rows as i64)) as u64;  // TODO Linesに持っていきたい
                            rows -= bit_row;

                            let index = bit_row.trailing_zeros() as usize;
                            filled[index] |= predefine.using_rows;
                        }

                        filled
                    };

                    placed.push(item.piece_key);
                    success += self.aggregate_recursively(item.next_index_id, next_filled, placed, checker);
                    placed.pop();
                }

                success
            }
            IndexNode::ToNextIndex(next_index_id) => {
                self.aggregate_recursively(*next_index_id, filled, placed, checker)
            }
            IndexNode::Complete => {
                let placed_vec = placed.iter()
                    .map(|it| &self.map_placed_piece_blocks[it])
                    .collect();
                if checker.checks(&placed_vec) { 1 } else { 0 }
            }
            IndexNode::Abort => { 0 }
        }
    }
}
