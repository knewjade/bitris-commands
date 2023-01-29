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
    goal_board: Board64,
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

        let goal_board = Board64::filled_up_to(clipped_board.height() as u8);

        Self { clipped_board, map_placed_piece_blocks, nodes, spawn_position, goal_board }
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

                let x = PlacedPieceBlocksFlow::find_one_stackable(
                    self.clipped_board.board(),
                    placed_piece_blocks_vec.clone(),
                    MoveRules::default(),
                    self.spawn_position,
                ).is_some();
                // // TODO
                // if !x {
                //     let y = PlacedPieceBlocksFlow::find_one_placeable(
                //         self.clipped_board.board(),
                //         placed_piece_blocks_vec.clone(),
                //     ).is_some();
                //     if !y {
                //         println!("SKIP");
                //         let x1: Vec<PlacedPiece> = placed_piece_blocks_vec.iter().map(|it| it.placed_piece).collect();
                //         dbg!(x1);
                //     }
                // }
                x
            }
        }

        let checker = PcAggregationCheckerImpl {
            shape_counters,
            clipped_board: self.clipped_board,
            spawn_position: self.spawn_position,
        };

        let filled = vec![Lines::blank(); self.clipped_board.height() as usize];
        let mut results = Vec::with_capacity((self.clipped_board.spaces() / 4) as usize);
        self.aggregate_recursively(self.nodes.head_index_id().unwrap(), filled, &mut results, &checker)
    }

    fn aggregate_recursively<'a>(
        &'a self,
        index_id: IndexId,
        filled: Vec<Lines>,
        placed_pieces: &mut Vec<&'a PlacedPieceBlocks>,
        checker: &'a impl PcAggregationChecker,
    ) -> u64 {
        match self.nodes.index(index_id).unwrap() {
            IndexNode::ToItem(next_item_id, item_length) => {
                let item_ids = (next_item_id.id..(next_item_id.id + *item_length as usize))
                    .map(|item_id| self.nodes.item(ItemId::new(item_id)).unwrap());

                let mut success = 0u64;
                for item in item_ids {
                    let current = &self.map_placed_piece_blocks[&item.placed_piece];

                    // 使っている行から、事前に揃えておく必要がある行を求める
                    // （=placed_pieceの行を揃えるために、使うことができない行を取り出す。）
                    // （他のミノの制約によって、いま注目している行を先に消さないと、そもそも置けないブロックが含まれている行）
                    let filled_rows = current.placed_piece.ys.iter()
                        .fold(Lines::blank(), |prev, &y| prev | filled[y as usize]);

                    // 注目しているミノを置く前の絶対に揃えられないラインが削除されていないといけないか
                    if current.intercepted_rows.overlaps(&filled_rows) {
                        // 使っている
                        continue;
                    }

                    // TODO 楽観的プレイスメントチェック
                    // すべてがうまった想定のボードから、
                    // 過去のピースのうち必ずカレントよりも後ろになるピースを抜いたものに対して配置できるかを確認する。
                    //
                    // もしかすると、現在のキーチェックも同時に行える可能性が高い

                    let mut inserted = placed_pieces.len();
                    for index in (0..placed_pieces.len()).rev() {
                        if placed_pieces[index].intercepted_rows.overlaps(&current.using_rows) {
                            // placed_pieceを置く前提となる行を、currentが使用している = placed_pieceはcurrentより先には置けない
                            inserted = index;
                        }
                    }

                    placed_pieces.insert(inserted, current);
                    let mut ok = true;

                    {
                        let index = inserted;
                        let current = placed_pieces[index];

                        let mut board = self.goal_board.clone();
                        let mut unset = false;
                        for &blocks in &placed_pieces[index+1..] {
                            if blocks.intercepted_rows.overlaps(&current.using_rows) {
                                blocks.unset_all(&mut board);
                                unset = true;
                            }
                        }

                        if unset {
                            current.unset_all(&mut board);
                            board.clear_lines_partially(current.intercepted_rows);

                            let bl_location = current.placed_piece.bottom_left();
                            let ground_placement = current.placed_piece.piece.with(bl(bl_location.x, bl_location.y));
                            if !ground_placement.is_landing(&board) {
                                ok = false;
                            }
                        }
                    }

                    if ok {
                        // intercepted_rowsを先に揃えないと、消すことができない行をマッピング
                        let next_filled = current.intercepted_rows.ys_iter()
                            .fold(filled.clone(), |mut filled, y| {
                                filled[y as usize] |= current.using_rows;
                                filled
                            });

                        success += self.aggregate_recursively(item.next_index_id, next_filled, placed_pieces, checker);
                    }

                    placed_pieces.remove(inserted);
                }

                success
            }
            IndexNode::ToNextIndex(next_index_id) => {
                self.aggregate_recursively(*next_index_id, filled, placed_pieces, checker)
            }
            IndexNode::Complete => {
                let mut ok = true;

                for index in 0..=placed_pieces.len() - 1 {
                    let current = placed_pieces[index];

                    let mut board = self.goal_board.clone();
                    let mut unset = false;
                    for &blocks in &placed_pieces[index+1..] {
                        if blocks.intercepted_rows.overlaps(&current.using_rows) {
                            blocks.unset_all(&mut board);
                            unset = true;
                        }
                    }

                    if unset {
                        current.unset_all(&mut board);
                        board.clear_lines_partially(current.intercepted_rows);

                        let bl_location = current.placed_piece.bottom_left();
                        let ground_placement = current.placed_piece.piece.with(bl(bl_location.x, bl_location.y));
                        if !ground_placement.is_landing(&board) {
                            ok = false;
                            break;
                        }
                    }
                }

                if ok {
                    if checker.checks(placed_pieces) { 1 } else { 0 }
                } else {
                    0
                }
            }
            IndexNode::Abort => { 0 }
        }
    }
}
