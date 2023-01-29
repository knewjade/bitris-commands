use bitris::prelude::*;
use fxhash::FxHashMap;

use crate::{ClippedBoard, ShapeCounter};
use crate::all_pcs::{IndexId, IndexNode, ItemId, Nodes};

trait PcAggregationChecker {
    fn checks(&self, placed_piece_blocks_vec: &Vec<&PlacedPieceBlocks>) -> bool;
}

pub(crate) struct Aggregator {
    clipped_board: ClippedBoard,
    map_placed_piece_blocks: FxHashMap<PlacedPiece, PlacedPieceBlocks>,
    nodes: Nodes,
    spawn_position: BlPosition,
}

impl Aggregator {
    pub(crate) fn new(
        clipped_board: ClippedBoard,
        placed_pieces: Vec<PlacedPiece>,
        nodes: Nodes,
        spawn_position: BlPosition,
    ) -> Self {
        let map_placed_piece_blocks = placed_pieces.into_iter()
            .fold(FxHashMap::default(), |mut map, placed_piece| {
                map.insert(placed_piece, PlacedPieceBlocks::make(placed_piece));
                map
            });

        Self { clipped_board, map_placed_piece_blocks, nodes, spawn_position }
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
            fn checks(&self, placed_piece_blocks_vec: &Vec<&PlacedPieceBlocks>) -> bool {
                let succeed = {
                    let shape_counter: ShapeCounter = placed_piece_blocks_vec.iter()
                        .map(|it| it.placed_piece.piece.shape)
                        .collect();
                    self.shape_counters.iter().any(|it| it.contains_all(&shape_counter))
                };
                if !succeed {
                    return false;
                }

                PlacedPieceBlocksFlow::find_one_stackable(
                    self.clipped_board.board(),
                    placed_piece_blocks_vec.clone(),
                    MoveRules::default(),
                    self.spawn_position,
                ).is_some()
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
        placed_pieces: &mut Vec<PlacedPiece>,
        checker: &impl PcAggregationChecker,
    ) -> u64 {
        match self.nodes.index(index_id).unwrap() {
            IndexNode::ToItem(next_item_id, item_length) => {
                let item_ids = (next_item_id.id..(next_item_id.id + *item_length as usize))
                    .map(|item_id| self.nodes.item(ItemId::new(item_id)).unwrap());

                let mut success = 0u64;
                for item in item_ids {
                    let blocks = &self.map_placed_piece_blocks[&item.placed_piece];

                    let filled_rows = blocks.placed_piece.ys.iter()
                        .fold(Lines::blank(), |prev, &y| prev | filled[y as usize]);

                    // 注目しているミノを置く前の絶対に揃えられないラインが削除されていないといけないか
                    if !(filled_rows & blocks.intercepted_rows).is_blank() {
                        // 使っている
                        continue;
                    }

                    let next_filled = {
                        let mut filled = filled.clone();

                        // 揃えられないラインを更新
                        // temp = [y]ラインにブロックがあると、使用できないライン一覧が記録されている
                        // ミノXの[y]がdeletedKeyに指定されていると、Xのブロックのあるラインは先に揃えられなくなる
                        for y in blocks.intercepted_rows.ys_iter() {
                            filled[y as usize] |= blocks.using_rows;
                        }

                        filled
                    };

                    placed_pieces.push(item.placed_piece);
                    success += self.aggregate_recursively(item.next_index_id, next_filled, placed_pieces, checker);
                    placed_pieces.pop();
                }

                success
            }
            IndexNode::ToNextIndex(next_index_id) => {
                self.aggregate_recursively(*next_index_id, filled, placed_pieces, checker)
            }
            IndexNode::Complete => {
                let placed_vec = placed_pieces.iter()
                    .map(|it| &self.map_placed_piece_blocks[it])
                    .collect();
                if checker.checks(&placed_vec) { 1 } else { 0 }
            }
            IndexNode::Abort => { 0 }
        }
    }
}
