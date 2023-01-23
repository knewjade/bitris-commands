use bitris::prelude::*;
use fxhash::{FxHashMap, FxHashSet};
use itertools::Itertools;

use crate::{ClippedBoard, ShapeCounter};
use crate::all_pcs::{IndexedPieces, IndexId, IndexNode, ItemId, Nodes, PieceKey, PlacedPieceBlocks, PredefinedPiece};

trait PcAggregationChecker {
    fn checks(&self, _shapes: Vec<Shape>) -> bool {
        true
    }
}

pub(crate) struct Aggregator {
    clipped_board: ClippedBoard,
    map_placed_piece_blocks: FxHashMap<PieceKey, PlacedPieceBlocks>,
    using_rows_each_y: Vec<Lines>,
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
        let using_rows_each_y = {
            let len_pieces = indexed_pieces.len();
            let height = clipped_board.height() as usize;

            let mut vec = Vec::<Lines>::with_capacity(len_pieces * height);

            for index in 0..len_pieces {
                let predefined_piece = &indexed_pieces[index];
                vec.extend((0..height).into_iter().map(|y| {
                    if predefined_piece.intercepted_rows.test_at(y) {
                        predefined_piece.using_rows
                    } else {
                        Lines::blank()
                    }
                }));
            }

            vec
        };

        let map_placed_piece_blocks = nodes.items.iter()
            .map(|item| item.piece_key)
            .sorted()
            .dedup()
            .fold(FxHashMap::default(), |mut map, piece_key| {
                let index = piece_key.piece_index(width);
                let lx = piece_key.lx(width);
                map.insert(piece_key, indexed_pieces[index].to_aggregate(lx));
                map
            });

        Self { clipped_board, map_placed_piece_blocks, using_rows_each_y, nodes, width, spawn_position }
    }

    pub(crate) fn aggregate(&self) -> u64 {
        if self.nodes.indexes.is_empty() {
            return 0;
        }

        struct PcAggregationCheckerImpl;
        impl PcAggregationChecker for PcAggregationCheckerImpl {}

        let mut filled = [Lines::blank(); 100];
        let mut results = Vec::new();
        self.aggregate_(IndexId::head(), 0, &mut filled, &mut results, &PcAggregationCheckerImpl)
    }

    pub(crate) fn aggregate_with_shape_counters(&self, shape_counters: &Vec<ShapeCounter>) -> u64 {
        if self.nodes.indexes.is_empty() {
            return 0;
        }

        struct PcAggregationCheckerImpl<'a> {
            shape_counters: &'a Vec<ShapeCounter>,
        }

        impl PcAggregationChecker for PcAggregationCheckerImpl<'_> {
            fn checks(&self, shapes: Vec<Shape>) -> bool {
                let counter = ShapeCounter::from(shapes);
                self.shape_counters.iter().any(|it| {
                    it.contains_all(&counter)
                })
            }
        }

        let checker = PcAggregationCheckerImpl { shape_counters };

        let mut filled = [Lines::blank(); 100];
        let mut results = Vec::new();
        self.aggregate_(IndexId::head(), 0, &mut filled, &mut results, &checker)
    }

    fn aggregate_(
        &self,
        index_id: IndexId,
        depth: usize,
        filled: &mut [Lines; 100],
        results2: &mut Vec<PieceKey>,
        checker: &impl PcAggregationChecker,
    ) -> u64 {
        match self.nodes.index(index_id) {
            IndexNode::ToItem(next_item_id, item_length) => {
                let mut success = 0u64;

                let height = self.clipped_board.height() as usize;
                let (next_item_id, item_length) = (*next_item_id, *item_length as usize);
                for item_id in next_item_id.id..(next_item_id.id + item_length) {
                    let item_id = ItemId::new(item_id);
                    let item = &self.nodes.item(item_id);
                    let piece_key = item.piece_key;
                    let predefine = &self.map_placed_piece_blocks[&piece_key];

                    let s = predefine.ys.iter().fold(Lines::blank(), |prev, y| {
                        prev | filled[depth * height + *y]
                    });

                    // 注目しているミノを置く前の絶対に揃えられないラインが削除されていないといけないか
                    if !(s & predefine.intercepted_rows).is_blank() {
                        // 使っている
                        continue;
                    }

                    results2.push(piece_key);

                    let next_depth = depth + 1;

                    let ni = next_depth * height;
                    let ci = depth * height;
                    let hi = piece_key.piece_index(self.width) * height;

                    // 揃えられないラインを更新
                    // temp = [y]ラインにブロックがあると、使用できないライン一覧が記録されている
                    // ミノXの[y]がdeletedKeyに指定されていると、Xのブロックのあるラインは先に揃えられなくなる
                    for j in 0..height {
                        filled[ni + j] = filled[ci + j] | self.using_rows_each_y[hi + j];
                    }

                    success += self.aggregate_(item.next_index_id, next_depth, filled, results2, checker);

                    results2.pop();
                }

                success
            }
            IndexNode::ToNextIndex(next_index_id) => {
                self.aggregate_(*next_index_id, depth, filled, results2, checker)
            }
            IndexNode::Complete => {
                let shapes = results2.iter()
                    .map(|it| {
                        self.map_placed_piece_blocks[it].piece.shape
                    })
                    .collect_vec();
                let s = if checker.checks(shapes) {
                    let x = results2.iter()
                        .map(|it| &self.map_placed_piece_blocks[it])
                        .collect_vec();
                    self.ok(&x)
                } else {
                    false
                };
                if s { 1 } else { 0 }
            }
            IndexNode::Abort => {
                0
            }
        }
    }

    fn ok(&self, results: &Vec<&PlacedPieceBlocks>) -> bool {
        struct Buffer<'a> {
            predefined_pieces: &'a Vec<&'a PlacedPieceBlocks>,
            spawn_placements: Vec<BlPlacement>,
            move_rules: &'a MoveRules<'a, SrsKickTable>,
        }

        impl Buffer<'_> {
            fn can_stack2(
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

                            if self.can_stack2(next_board, next_remaining, visited) {
                                return true;
                            }
                        }
                    }
                }

                return false;
            }
        }

        let mut hash_set = FxHashSet::<u64>::default();
        hash_set.reserve((1u64 << results.len()) as usize);

        let move_rules = MoveRules::srs(AllowMove::Softdrop);

        // Spawn on top of the well to avoid getting stuck.
        let spawn_placements = results.iter()
            .map(|result| {
                result.piece.shape.with(Orientation::North).with(self.spawn_position)
            })
            .collect_vec();

        let buffer = Buffer { predefined_pieces: results, spawn_placements, move_rules: &move_rules };

        buffer.can_stack2(self.clipped_board.board(), (1u64 << results.len()) - 1, &mut hash_set)
    }
}
