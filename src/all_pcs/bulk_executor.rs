use bitris::AllowMove::Softdrop;
use bitris::prelude::*;
use fxhash::{FxHashMap, FxHashSet};
use itertools::Itertools;
use thiserror::Error;

use crate::{ClippedBoard, Pattern};
use crate::all_pcs::nodes::{IndexNode, ItemNode, Nodes};
use crate::internals::Array4;

const WIDTH: usize = 10;

struct Aggregator {
    clipped_board: ClippedBoard,
    nodes: Nodes,
    predefines: Vec<(usize, PiecePredefines2)>,
}

impl Aggregator {
    fn aggregate(&self) -> u64 {
        let predefines: Vec<PiecePredefines3> = self.predefines.iter()
            .map(|(_, p2)| { p2.to_piece_predefines3() })
            .collect();

        let height = self.clipped_board.height() as usize;

        let mut using_rows_each_y = Vec::<Lines>::new();
        using_rows_each_y.resize(predefines.len() * height, Lines::blank());
        for mino_index in 0..predefines.len() {
            let mino = &predefines[mino_index];
            let head_index = mino_index * height;
            for iy in 0..height {
                if mino.deleted_rows.test_at(iy) {
                    using_rows_each_y[head_index + iy] = mino.using_rows;
                }
            }
        }

        let mut filled = [0; 100];
        let mut results = [0; 100];
        self.aggregate_(&predefines, &using_rows_each_y, 0, 0, &mut filled, &mut results)
    }

    fn aggregate_(
        &self,
        predefines: &Vec<PiecePredefines3>,
        using_rows_each_y: &Vec<Lines>,
        index_id: usize,
        depth: usize,
        filled: &mut [u64; 100],
        results: &mut [usize; 100],
    ) -> u64 {
        match self.nodes.indexes[index_id] {
            IndexNode::Jump(next_item_id, item_length) => {
                let mut success = 0u64;

                let height = self.clipped_board.height() as usize;
                let (next_item_id, item_length) = (next_item_id as usize, item_length as usize);
                for index in next_item_id..(next_item_id + item_length) {
                    let item = &self.nodes.items[index];
                    let mino_index_and_lx = item.mino_index() as usize;
                    let mino_index = mino_index_and_lx / WIDTH;
                    let predefine = &predefines[mino_index];

                    let s = predefine.ys.iter().fold(0u64, |prev, y| {
                        prev | filled[depth * height + *y]
                    });

                    // 注目しているミノを置く前の絶対に揃えられないラインが削除されていないといけないか
                    if 0 < (s & predefine.deleted_rows.key) {
                        // 使っている
                        continue;
                    }

                    results[depth] = mino_index_and_lx;

                    match item {
                        ItemNode::ToHi(_) => {
                            let x = results[0..=depth].iter()
                                .map(|&it| {
                                    let mino_index = it / WIDTH;
                                    let lx = it % WIDTH;
                                    let pre = &predefines[mino_index];
                                    let offset = Offset::new(lx as i32, 0);
                                    let board = pre.locations.iter()
                                        .map(|location| { location + offset })
                                        .fold(Board64::blank(), |mut merge, location| {
                                            merge.set_at(location);
                                            merge
                                        });
                                    (pre, lx, board)
                                })
                                .collect_vec();
                            self.ok(&x);
                        }
                        ItemNode::ToIndex(_, next_index_id) => {
                            let next_depth = depth + 1;

                            let ni = next_depth * height;
                            let ci = depth * height;
                            let hi = mino_index * height;

                            // 揃えられないラインを更新
                            // temp = [y]ラインにブロックがあると、使用できないライン一覧が記録されている
                            // ミノXの[y]がdeletedKeyに指定されていると、Xのブロックのあるラインは先に揃えられなくなる
                            for j in 0..height {
                                filled[ni + j] = filled[ci + j] | using_rows_each_y[hi + j].key;
                            }

                            success += self.aggregate_(predefines, using_rows_each_y, *next_index_id, next_depth, filled, results);
                        }
                    }
                }

                success
            }
            IndexNode::Skip(next_index_id) => {
                self.aggregate_(predefines, using_rows_each_y, next_index_id as usize, depth, filled, results)
            }
            IndexNode::ToHi => {
                let x = results[0..depth].iter()
                    .map(|&it| {
                        let mino_index = it / WIDTH;
                        let lx = it % WIDTH;
                        let pre = &predefines[mino_index];
                        let offset = Offset::new(lx as i32, 0);
                        let board = pre.locations.iter()
                            .map(|location| { location + offset })
                            .fold(Board64::blank(), |mut merge, location| {
                                merge.set_at(location);
                                merge
                            });
                        (pre, lx, board)
                    })
                    .collect_vec();
                if self.ok(&x) { 1 } else { 0 }
            }
        }
    }

    fn ok(&self, results: &Vec<(&PiecePredefines3, usize, Board64)>) -> bool {
        let mut hash_set = FxHashSet::<u64>::default();
        hash_set.reserve((1u64 << results.len()) as usize);

        let move_rules = MoveRules::srs(Softdrop);

        // if results.iter().all(|&(mino, lx, mino_board)| {
        //     mino.piece.shape == Shape::O || mino.piece.shape == Shape::L || mino.piece.shape == Shape::J
        // }) {
        //     let v = results.iter().collect_vec();
        //     if v[0].0.piece == Piece::new(Shape::L, Orientation::East)
        //         && v[1].0.piece == Piece::new(Shape::J, Orientation::South)
        //         && v[2].0.piece == Piece::new(Shape::O, Orientation::North)
        //     {
        //         println!("!")
        //     }
        //     let r = self.ok2(results, self.clipped_board.board(), (1u64 << results.len()) - 1, &mut hash_set, &move_rules);
        //     if r {
        //         // results.iter()
        //         //     .for_each(|&(mino, lx, _)| {
        //         //         let it = BlPlacement::new(mino.piece, bl(lx as i32, mino.ys[0] as i32));
        //         //         print!("{it} {},", mino.deleted_rows.key);
        //         //     });
        //         // println!("OK");
        //     } else {
        //         results.iter()
        //             .for_each(|&(mino, lx, _)| {
        //                 let it = BlPlacement::new(mino.piece, bl(lx as i32, mino.ys[0] as i32));
        //                 print!("{it} {},", mino.deleted_rows.key);
        //             });
        //         println!("NG");
        //     }
        // }

        self.ok2(results, self.clipped_board.board(), (1u64 << results.len()) - 1, &mut hash_set, &move_rules)
    }

    fn ok2(&self, results: &Vec<(&PiecePredefines3, usize, Board64)>, board_: Board64, rest: u64, visited: &mut FxHashSet::<u64>, move_rules: &MoveRules<SrsKickTable>) -> bool {
        let mut board2 = board_.clone();
        let deleted_key = board2.clear_lines();

        let mut rest2 = rest;
        while rest2 != 0 {
            let bit = rest2 & (-(rest2 as i64)) as u64;

            // let next_used = used | bit;
            let index = bit.trailing_zeros() as usize;
            let (mino, lx, mino_board) = &results[index];
            let mino = *mino;

            // ミノを置くためのラインがすべて削除されている
            if (mino.deleted_rows.key & deleted_key.key) == mino.deleted_rows.key {
                let original_by = mino.ys[0] as i32;
                let mask = (1u64 << original_by) - 1;
                let deleted_lines = mask & deleted_key.key;

                let by = original_by - deleted_lines.count_ones() as i32;
                let placement = mino.piece.with(bl(*lx as i32, by));

                // Spawn on top of the well to avoid getting stuck.
                let spawn_position = bl(5, board2.well_top() as i32);
                let spawn = mino.piece.shape.with(Orientation::North).with(spawn_position);

                if move_rules.can_reach(placement, board2, spawn) {
                    let next_rest = rest - bit;
                    if next_rest == 0 {
                        return true;
                    }

                    if !visited.insert(next_rest) {
                        rest2 -= bit;
                        continue;
                    }

                    let mut next_field = board_.clone();
                    next_field.merge(mino_board);

                    if self.ok2(results, next_field, next_rest, visited, move_rules) {
                        return true;
                    }
                }
            }

            rest2 -= bit;
        }

        return false;
    }
}

// TODO SequenceやOrderをcollect()したい
// TODO FromIteratorをじっそうする？
// TODO assert! > debug_assert!


#[derive(Copy, Clone, PartialEq, PartialOrd, Hash, Default, Debug)]
struct PiecePredefines2 {
    pub piece: Piece,
    pub ys: Array4<usize>,
    pub locations: Array4<Location>,
}

impl PiecePredefines2 {
    fn to_piece_predefines(&self, height: u32) -> PiecePredefines {
        let min_vertical_index = self.locations
            .iter()
            .filter(|location| { location.x == 0 })
            .map(|location| location.y as usize)
            .min()
            .unwrap();

        let vertical_relative_block = self.locations
            .iter()
            .fold(0u64, |prev, location| {
                let shift = location.x * (height as i32) + location.y - min_vertical_index as i32;
                prev | (1u64 << shift)
            });

        PiecePredefines { piece: self.piece, min_vertical_index, vertical_relative_block }
    }

    fn to_piece_predefines3(&self) -> PiecePredefines3 {
        let deleted_rows = self.ys.iter()
            .skip(1)
            .fold((self.ys[0], 0u64), |(prev_y, merge), y| {
                let a = (1u64 << y) - 1;
                let b = (1u64 << (prev_y + 1)) - 1;
                let i = a ^ b;
                (*y, merge | (i))
            }).1;

        let using_rows = self.ys.iter()
            .fold(0u64, |merge, y| {
                merge | (1u64 << y)
            });

        PiecePredefines3 {
            piece: self.piece,
            ys: self.ys,
            locations: self.locations,
            using_rows: Lines::new(using_rows),
            deleted_rows: Lines::new(deleted_rows),
        }
    }
}

fn make_minos2(piece: Piece, height: usize) -> Vec<PiecePredefines2> {
    let piece_blocks = piece.to_piece_blocks();
    (0..height).combinations(piece_blocks.height as usize)
        .map(|mut ys| {
            ys.sort();
            Array4::try_from(ys).unwrap()
        })
        .map(|ys| {
            let locations = piece_blocks.offsets
                .into_iter()
                .map(|offset| { offset - piece_blocks.bottom_left })
                .map(|offset| { Location::new(offset.dx, ys[offset.dy as usize] as i32) })
                .collect_vec()
                .try_into()
                .unwrap();
            PiecePredefines2 { piece, ys, locations }
        })
        .collect()
}

#[derive(Copy, Clone, PartialEq, PartialOrd, Hash, Default, Debug)]
struct PiecePredefines {
    pub piece: Piece,
    pub min_vertical_index: usize,
    pub vertical_relative_block: u64,
}

#[derive(Clone, PartialEq, PartialOrd, Hash, Default, Debug)]
struct PiecePredefines3 {
    pub piece: Piece,
    pub ys: Array4<usize>,
    pub using_rows: Lines,
    pub deleted_rows: Lines,
    pub locations: Array4<Location>,
}

struct Frontier {
    board: u64,
}

fn make_predefines2(height: usize) -> Vec<(usize, PiecePredefines2)> {
    Piece::all_vec()
        .into_iter()
        .filter(|piece| piece.canonical().is_none())
        .flat_map(|piece| {
            make_minos2(piece, height)
        })
        .enumerate()
        .collect()
}

fn build2(clipped_board: ClippedBoard) -> Aggregator {
    let predefines = make_predefines2(clipped_board.height() as usize);
    let nodes = build(clipped_board, &predefines);
    Aggregator { clipped_board, nodes, predefines }
}

fn build(clipped_board: ClippedBoard, predefines: &Vec<(usize, PiecePredefines2)>) -> Nodes {
    assert!(!predefines.is_empty());

    let predefines = predefines.iter()
        .map(|(index, p2)| (*index, p2.to_piece_predefines(clipped_board.height())))
        .collect_vec();

    let height = clipped_board.height() as usize;

    let mut nodes = Nodes::empty();
    let mut frontiers = Vec::<Frontier>::new();

    frontiers.push(Frontier { board: 0 });

    let mut hash_map = FxHashMap::<u64, usize>::default();

    for lx in 0..WIDTH {
        for y in 0..height {
            if clipped_board.board().is_occupied_at(xy(lx as i32, y as i32)) {
                // TODO あっている？
                for tail in (nodes.index_serial())..(frontiers.len()) {
                    frontiers[tail].board >>= 1; // TODO sliceで置き換えられる?
                }
                continue;
            }

            let minos = predefines.iter()
                .filter(|(_, mino)| mino.min_vertical_index == y)
                .filter(|(_, mino)| lx as u32 + mino.piece.width() <= WIDTH as u32)
                .map(|(mino_index, mino)| (mino_index * WIDTH + lx, mino))
                .collect_vec();

            if minos.is_empty() {
                continue;
            }

            hash_map.clear();

            let board_mask = {
                let board = clipped_board.board();
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
                        if x == WIDTH as i32 {
                            break;
                        }
                    }
                }
                assert_eq!(m & 1, 0);
                m
            };

            // Number of remaining search blocks, including the block at `index`
            let rest: usize = height * (WIDTH - lx - 1) + (height - y);
            let fill_block_mask = if rest <= 3 * height + 1 {
                // All remaining blocks are filled, including the block at `index`
                (1u64 << rest) - 1
            } else {
                u64::MAX
            };

            assert!(nodes.index_serial() < frontiers.len());

            for tail in (nodes.index_serial())..(frontiers.len()) {
                let current_bits = frontiers[tail].board | board_mask; // TODO sliceで置き換えられる?
                if current_bits & 1 == 0 {
                    // No block at `index`
                    let start_item_node_index = nodes.item_serial();
                    let mut item_size = 0usize;

                    for (mino_index, mino) in &minos {
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
                            if let Some(next_index_id) = hash_map.get(&hash_key) {
                                nodes.put(mino_index, *next_index_id);
                            } else {
                                // Found new state

                                // [Future reference] If `head` exceeds the `frontiers` size and rotate index, becomes nextIndexId != head
                                let next_index_id = frontiers.len();
                                hash_map.insert(hash_key, next_index_id);

                                frontiers.push(Frontier { board: hash_key });

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
                    if let Some(next_index_id) = hash_map.get(&hash_key) {
                        assert_eq!(tail, nodes.index_serial());
                        nodes.skip(*next_index_id as u32);
                    } else {
                        // Found new state

                        // [Future reference] If `head` exceeds the `frontiers` size and rotate index, becomes nextIndexId != head
                        let next_index_id = frontiers.len();
                        hash_map.insert(hash_key, next_index_id);

                        frontiers.push(Frontier { board: hash_key });

                        assert_eq!(tail, nodes.index_serial());
                        nodes.skip(next_index_id as u32);
                    }
                }
            }
        }
    }

    for _ in (nodes.index_serial())..(frontiers.len()) {
        nodes.complete2();
    }

    assert_eq!(nodes.index_serial(), frontiers.len());

    nodes
}


/// A collection of errors that occur when making the executor.
#[derive(Error, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum AllPcsExecutorBulkCreationError {
    #[error("Unexpected the count of board spaces.")]
    UnexpectedBoardSpaces,
    #[error("The pattern is too short to take a PC.")]
    ShortPatternDimension,
    #[error("Board height exceeds the upper limit. Up to 20 are supported.")]
    BoardIsTooHigh,
}

/// The executor to find PC possibles.
#[derive(Clone, PartialEq, PartialOrd, Hash, Debug)]
pub struct AllPcsBulkExecutor<'a, T: RotationSystem> {
    move_rules: &'a MoveRules<'a, T>,
    clipped_board: ClippedBoard,
    pattern: &'a Pattern,
    allows_hold: bool,
    has_extra_shapes: bool,
    spawn_position: BlPosition,
}

impl<'a, T: RotationSystem> AllPcsBulkExecutor<'a, T> {
    // TODO desc
    pub fn try_new(
        move_rules: &'a MoveRules<T>,
        clipped_board: ClippedBoard,
        pattern: &'a Pattern,
        allows_hold: bool,
    ) -> Result<Self, AllPcsExecutorBulkCreationError> {
        use AllPcsExecutorBulkCreationError::*;

        if 20 < clipped_board.height() {
            return Err(BoardIsTooHigh);
        }

        if clipped_board.spaces() % 4 != 0 {
            return Err(UnexpectedBoardSpaces);
        }

        let dimension = pattern.dim_shapes() as u32;
        if dimension < clipped_board.spaces() / 4 {
            return Err(ShortPatternDimension);
        }

        debug_assert!(0 < clipped_board.spaces());

        let has_extra_shapes = clipped_board.spaces() / 4 < dimension;

        // Spawn over the top of the well to avoid getting stuck.
        let spawn_position = bl(5, clipped_board.height() as i32 + 4);

        Ok(Self { move_rules, clipped_board, pattern, allows_hold, has_extra_shapes, spawn_position })
    }

    /// TODO desc Start the search for PC possible in bulk.
    pub fn execute(&self) -> u64 {
        let aggregator = build2(self.clipped_board);
        aggregator.aggregate()
    }
}


#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use bitris::Board64;
    use rstest::rstest;

    use crate::all_pcs::bulk_executor::{build, build2, make_predefines2};
    use crate::ClippedBoard;

    #[rstest(height, item, index,
    case(4, 5385, 4168 - 2),
    case(6, 412515, 178069 - 2),
    )]
    fn blank(height: u32, item: usize, index: usize) {
        let clipped_board = ClippedBoard::try_new(Board64::blank(), height).unwrap();
        let predefines = make_predefines2(clipped_board.height() as usize);
        let nodes = build(clipped_board, &predefines);
        assert_eq!(nodes.item_serial(), item);
        assert_eq!(nodes.index_serial(), index);
    }

    #[test]
    fn test() {
        let board = Board64::from_str("
            #######...
            #######...
            #######...
            #######...
        ").unwrap();
        let clipped_board = ClippedBoard::try_new(board, 4).unwrap();
        let predefines = make_predefines2(clipped_board.height() as usize);
        let nodes = build(clipped_board, &predefines);
        assert_eq!(nodes.item_serial(), 379); // TODO
        assert_eq!(nodes.index_serial(), 299); // TODO
    }

    #[test]
    fn test2() {
        let board = Board64::from_str("
            ...#######
            ...#######
            ...#######
            ...#######
        ").unwrap();
        let clipped_board = ClippedBoard::try_new(board, 4).unwrap();
        let predefines = make_predefines2(clipped_board.height() as usize);
        let nodes = build(clipped_board, &predefines);
        assert_eq!(nodes.item_serial(), 379); // TODO
        assert_eq!(nodes.index_serial(), 309); // TODO
    }

    #[test]
    fn test3_() {
        let board = Board64::from_str("
            ...#######
            ...#######
            ...#######
            ...#######
        ").unwrap();
        let height = 4;
        let clipped_board = ClippedBoard::try_new(board, height).unwrap();
        let aggregator = build2(clipped_board);
        aggregator.aggregate();
    }

    #[test]
    fn test2_() {
        let board = Board64::from_str("
            ...######.
            ...######.
            ...######.
            ..########
        ").unwrap();
        let height = 4;
        let clipped_board = ClippedBoard::try_new(board, height).unwrap();
        let aggregator = build2(clipped_board);
        aggregator.aggregate();
    }
}
