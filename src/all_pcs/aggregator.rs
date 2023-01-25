use bitris::prelude::*;
use fxhash::FxHashMap;
use itertools::Itertools;

use crate::{ClippedBoard, ShapeCounter};
use crate::all_pcs::{IndexedPieces, IndexId, IndexNode, ItemId, Nodes, PieceKey, PredefinedPiece};

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
                let lx = piece_key.lx(width) as u8;
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
            fn checks(&self, placed_piece_blocks_vec: &Vec<&PlacedPieceBlocks>) -> bool {
                can_stack(placed_piece_blocks_vec, self.clipped_board.board(), &MoveRules::default()).is_some()
            }
        }

        let checker = PcAggregationCheckerImpl {
            clipped_board: self.clipped_board,
            spawn_position: self.spawn_position,
        };

        let filled = vec![Lines::blank(); self.clipped_board.height() as usize];
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
                        .map(|it| it.placed_piece.piece.shape)
                        .collect();
                    self.shape_counters.iter().any(|it| it.contains_all(&shape_counter))
                };
                if !succeed {
                    return false;
                }

                can_stack(placed_vec, self.clipped_board.board(), &MoveRules::default()).is_some()
            }
        }

        let checker = PcAggregationCheckerImpl {
            shape_counters,
            clipped_board: self.clipped_board,
            spawn_position: self.spawn_position,
        };

        let filled = vec![Lines::blank(); self.clipped_board.height() as usize];
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

                    let filled_rows = predefine.placed_piece.ys.iter()
                        .fold(Lines::blank(), |prev, &y| prev | filled[y as usize]);

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
                        let using_rows = predefine.using_rows();
                        while rows != 0 {
                            let bit_row = rows & (-(rows as i64)) as u64;  // TODO Linesに持っていきたい
                            rows -= bit_row;

                            let index = bit_row.trailing_zeros() as usize;
                            filled[index] |= using_rows;
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
