use bitris::prelude::*;
use fxhash::FxHashMap;

use crate::{ClippedBoard, HoldExpandedPattern, Pattern, PatternShapeMatcher, ShapeCounter, ShapeMatcher};
use crate::all_pcs::{IndexId, IndexNode, ItemId, Nodes};

trait PcAggregationChecker {
    fn save_if_need(&mut self, placed_piece_blocks_vec: &Vec<&PlacedPieceBlocks>);
}

fn visit_all_dyn<'a, T: ShapeMatcher>(
    initial_board: Board64,
    refs: &Vec<&'a PlacedPieceBlocks>,
    validator: impl Fn(&Board64, BlPlacement) -> SearchResult,
    make_matcher: impl FnOnce() -> T,
) -> Option<PlacedPieceBlocksFlow<'a>> {
    if refs.is_empty() {
        return None;
    }

    assert!(refs.len() < 64, "refs length supports up to 64.");

    struct Builder<'a, 'b> {
        refs: &'a Vec<&'b PlacedPieceBlocks>,
        results: Vec<&'b PlacedPieceBlocks>,
    }

    impl Builder<'_, '_> {
        fn build(
            &mut self,
            board: Board64,
            remaining: u64,
            matcher: impl ShapeMatcher,
            validator: &impl Fn(&Board64, BlPlacement) -> SearchResult,
        ) -> bool {
            let mut candidates = remaining;
            while 0 < candidates {
                let next_candidates = candidates & (candidates - 1);
                let bit = candidates - next_candidates;
                let next_remaining = remaining - bit;

                candidates = next_candidates;

                let placed_piece_blocks = self.refs[bit.trailing_zeros() as usize];
                let shape = placed_piece_blocks.placed_piece.piece.shape;
                let (matched, next_matcher) = matcher.match_shape(shape);
                if !matched {
                    continue;
                }

                if let Some(placement) = placed_piece_blocks.place_according_to(board) {
                    if validator(&board, placement) == SearchResult::Pruned {
                        continue;
                    }

                    self.results.push(placed_piece_blocks);

                    if next_remaining == 0 {
                        return true;
                    }

                    let mut next_board = board;
                    next_board.set_all(&placed_piece_blocks.locations);

                    if self.build(next_board, next_remaining, next_matcher, validator) {
                        return true;
                    }
                    self.results.pop();
                }
            }

            false
        }
    }

    let len = refs.len();
    let mut builder = Builder {
        refs,
        results: Vec::with_capacity(len),
    };

    if builder.build(initial_board, (1u64 << len) - 1, make_matcher(), &validator) {
        Some(PlacedPieceBlocksFlow::new(initial_board, builder.results))
    } else {
        None
    }
}

pub(crate) struct Aggregator {
    clipped_board: ClippedBoard,
    map_placed_piece_blocks: FxHashMap<PlacedPiece, PlacedPieceBlocks>,
    nodes: Nodes,
    spawn_position: BlPosition,
    goal_board: Board64,
}

// TODO
pub struct PcSolutions {
    clipped_board: ClippedBoard,
    placed_pieces: Vec<Vec<PlacedPiece>>,
}

impl PcSolutions {
    #[inline]
    pub fn empty(clipped_board: ClippedBoard) -> Self {
        Self { clipped_board, placed_pieces: Vec::new() }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.placed_pieces.len()
    }
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

    pub(crate) fn aggregate_with_shape_counters<T: RotationSystem>(&self, shape_counters: &Vec<ShapeCounter>, move_rules: &MoveRules<T>) -> PcSolutions {
        if self.nodes.indexes.is_empty() {
            return PcSolutions::empty(self.clipped_board);
        }

        struct PcAggregationCheckerImpl<'a, T: RotationSystem> {
            shape_counters: &'a Vec<ShapeCounter>,
            clipped_board: ClippedBoard,
            spawn_position: BlPosition,
            move_rules: &'a MoveRules<'a, T>,
            solutions: Vec<Vec<PlacedPiece>>,
        }

        impl<T: RotationSystem> PcAggregationChecker for PcAggregationCheckerImpl<'_, T> {
            fn save_if_need(&mut self, placed_piece_blocks_vec: &Vec<&PlacedPieceBlocks>) {
                let succeed = {
                    let shape_counter: ShapeCounter = placed_piece_blocks_vec.iter()
                        .map(|it| it.placed_piece.piece.shape)
                        .collect();
                    self.shape_counters.iter().any(|it| it.contains_all(&shape_counter))
                };
                if !succeed {
                    return;
                }

                let succeed = PlacedPieceBlocksFlow::find_one_stackable(
                    self.clipped_board.board(),
                    placed_piece_blocks_vec,
                    self.move_rules,
                    self.spawn_position,
                );

                if let Some(flow) = succeed {
                    self.solutions.push(flow.refs.iter().map(|it| it.placed_piece).collect())
                }
            }
        }

        let mut checker = PcAggregationCheckerImpl {
            shape_counters,
            clipped_board: self.clipped_board,
            spawn_position: self.spawn_position,
            move_rules,
            solutions: Vec::new(),
        };

        let mut results = Vec::with_capacity((self.clipped_board.spaces() / 4) as usize);
        self.aggregate_recursively(self.nodes.head_index_id().unwrap(), &mut results, &mut checker);
        PcSolutions {
            clipped_board: self.clipped_board,
            placed_pieces: checker.solutions,
        }
    }

    pub(crate) fn aggregate_with_pattern_allows_hold<T: RotationSystem>(&self, pattern: &Pattern, move_rules: &MoveRules<T>) -> PcSolutions {
        if self.nodes.indexes.is_empty() {
            return PcSolutions::empty(self.clipped_board);
        }

        let shape_counters: Vec<ShapeCounter> = pattern.to_shape_counter_vec();

        struct PcAggregationCheckerImpl<'a, T: RotationSystem> {
            hold_extended_pattern: HoldExpandedPattern<'a>,
            shape_counters: Vec<ShapeCounter>,
            clipped_board: ClippedBoard,
            spawn_position: BlPosition,
            move_rules: &'a MoveRules<'a, T>,
            solutions: Vec<Vec<PlacedPiece>>,
        }

        impl <T: RotationSystem> PcAggregationChecker for PcAggregationCheckerImpl<'_, T> {
            fn save_if_need(&mut self, placed_piece_blocks_vec: &Vec<&PlacedPieceBlocks>) {
                let succeed = {
                    let shape_counter: ShapeCounter = placed_piece_blocks_vec.iter()
                        .map(|it| it.placed_piece.piece.shape)
                        .collect();
                    self.shape_counters.iter().any(|it| it.contains_all(&shape_counter))
                };
                if !succeed {
                    return;
                }

                let succeed = visit_all_dyn(
                    self.clipped_board.board(),
                    placed_piece_blocks_vec,
                    |board, placement| {
                        let board_to_place = board.after_clearing();
                        if self.move_rules.can_reach(placement, board_to_place, placement.piece.with(self.spawn_position)) {
                            return SearchResult::Success;
                        }
                        SearchResult::Pruned
                    },
                    || self.hold_extended_pattern.new_matcher(),
                );

                if let Some(flow) = succeed {
                    self.solutions.push(flow.refs.iter().map(|it| it.placed_piece).collect())
                }
            }
        }

        let mut checker = PcAggregationCheckerImpl {
            hold_extended_pattern: HoldExpandedPattern::from(pattern),
            shape_counters,
            clipped_board: self.clipped_board,
            spawn_position: self.spawn_position,
            move_rules,
            solutions: Vec::new(),
        };

        let mut results = Vec::with_capacity((self.clipped_board.spaces() / 4) as usize);
        self.aggregate_recursively(self.nodes.head_index_id().unwrap(), &mut results, &mut checker);
        PcSolutions {
            clipped_board: self.clipped_board,
            placed_pieces: checker.solutions,
        }
    }

    pub(crate) fn aggregate_with_pattern_allows_no_hold<T: RotationSystem>(&self, pattern: &Pattern, move_rules: &MoveRules<T>) -> PcSolutions {
        if self.nodes.indexes.is_empty() {
            return PcSolutions::empty(self.clipped_board);
        }

        let shape_counters: Vec<ShapeCounter> = pattern.to_shape_counter_vec();

        struct PcAggregationCheckerImpl<'a, T: RotationSystem> {
            pattern: &'a Pattern,
            shape_counters: Vec<ShapeCounter>,
            clipped_board: ClippedBoard,
            spawn_position: BlPosition,
            move_rules: &'a MoveRules<'a, T>,
            solutions: Vec<Vec<PlacedPiece>>,
        }

        impl <T: RotationSystem> PcAggregationChecker for PcAggregationCheckerImpl<'_, T> {
            fn save_if_need(&mut self, placed_piece_blocks_vec: &Vec<&PlacedPieceBlocks>) {
                let succeed = {
                    let shape_counter: ShapeCounter = placed_piece_blocks_vec.iter()
                        .map(|it| it.placed_piece.piece.shape)
                        .collect();
                    self.shape_counters.iter().any(|it| it.contains_all(&shape_counter))
                };
                if !succeed {
                    return;
                }

                let succeed = visit_all_dyn(
                    self.clipped_board.board(),
                    placed_piece_blocks_vec,
                    |board, placement| {
                        let board_to_place = board.after_clearing();
                        if self.move_rules.can_reach(placement, board_to_place, placement.piece.with(self.spawn_position)) {
                            return SearchResult::Success;
                        }
                        SearchResult::Pruned
                    },
                    || self.pattern.new_matcher(),
                );

                if let Some(flow) = succeed {
                    self.solutions.push(flow.refs.iter().map(|it| it.placed_piece).collect())
                }
            }
        }

        let mut checker = PcAggregationCheckerImpl {
            pattern,
            shape_counters,
            clipped_board: self.clipped_board,
            spawn_position: self.spawn_position,
            move_rules,
            solutions: Vec::new(),
        };

        let mut results = Vec::with_capacity((self.clipped_board.spaces() / 4) as usize);
        self.aggregate_recursively(self.nodes.head_index_id().unwrap(), &mut results, &mut checker);
        PcSolutions {
            clipped_board: self.clipped_board,
            placed_pieces: checker.solutions,
        }
    }

    fn aggregate_recursively<'a>(
        &'a self,
        index_id: IndexId,
        placed_pieces: &mut Vec<&'a PlacedPieceBlocks>,
        checker: &mut impl PcAggregationChecker,
    ) {
        match self.nodes.index(index_id).unwrap() {
            IndexNode::ToItem(next_item_id, item_length) => {
                let item_ids = (next_item_id.id..(next_item_id.id + *item_length as usize))
                    .map(|item_id| self.nodes.item(ItemId::new(item_id)).unwrap());

                for item in item_ids {
                    let current = &self.map_placed_piece_blocks[&item.placed_piece];

                    let mut filled_rows = Lines::blank(); // currentより後に使われることが確定している行
                    // 次に挿入する位置。依存関係があるピースが必ず後ろにくるようにする。
                    // 依存関係がない場合は任意。つまり、「後ろにあるから、後で置く」が常に成り立つわけではないので注意
                    let mut inserted = placed_pieces.len();
                    for index in (0..placed_pieces.len()).rev() {
                        if placed_pieces[index].intercepted_rows.overlaps(&current.using_rows) {
                            // placed_pieceを置く前提となる行を、currentが使用している = placed_pieceはcurrentより先には置けない
                            inserted = index;

                            // つまり、placed_pieceが使っている行を、currentより前に揃えることはできない
                            filled_rows |= placed_pieces[index].using_rows;
                        }
                    }

                    if current.intercepted_rows.overlaps(&filled_rows) {
                        // currentの後のピースで使われる行が消えていないと、currentが置けない場合は、絶対に配置できないのでスキップ
                        continue;
                    }

                    placed_pieces.insert(inserted, current);
                    self.aggregate_recursively(item.next_index_id, placed_pieces, checker);
                    placed_pieces.remove(inserted);
                }
            }
            IndexNode::ToNextIndex(next_index_id) => {
                self.aggregate_recursively(*next_index_id, placed_pieces, checker)
            }
            IndexNode::Complete => {
                let mut ok = true;

                for index in 0..=placed_pieces.len() - 1 {
                    let current = placed_pieces[index];

                    let mut board = self.goal_board.clone();
                    let mut unset = false;
                    for &blocks in &placed_pieces[index + 1..] {
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
                    checker.save_if_need(placed_pieces);
                }
            }
            IndexNode::Abort => {}
        }
    }
}
