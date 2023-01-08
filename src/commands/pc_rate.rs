use std::collections::hash_map::Keys;

use bitris::{Board64, MoveRules, RotationSystem};
use fxhash::FxHashMap;

use crate::{Pattern, ShapeSequence};
use crate::internals::ClippedBoard;

mod pc_rate {
    use bitris::prelude::*;

    use crate::{ForEachVisitor, OrderCursor, Pattern, PopOp, ShapeOrder, ShapeSequence};
    use crate::commands::PcRateResult;
    use crate::internals::{ClippedBoard, FuzzyShape, FuzzyShapeOrder};

    #[inline]
    fn validate(clipped: &ClippedBoard) -> bool {
        let wall = (1 << clipped.height) - 1;
        let mut frees_sum = clipped.height - clipped.board.cols[0].count_ones();

        for x in 1..10 {
            let frees_in_column = clipped.height - clipped.board.cols[x].count_ones();
            if (clipped.board.cols[x - 1] | clipped.board.cols[x]) == wall {
                if frees_sum % 4 != 0 {
                    return false;
                }
                frees_sum = frees_in_column;
            } else {
                frees_sum += frees_in_column;
            }
        }

        debug_assert_eq!(frees_sum % 4, 0);

        true
    }

    struct Visitor<'a> {
        result: &'a mut PcRateResult,
    }

    impl<'a> ForEachVisitor<[FuzzyShape]> for Visitor<'a> {
        #[inline]
        fn visit(&mut self, fuzzy_shapes: &[FuzzyShape]) {
            let fuzzy_shape_order = FuzzyShapeOrder::new(fuzzy_shapes.to_vec());
            fuzzy_shape_order.expand_as_wildcard_walk(self);
        }
    }

    impl<'a> ForEachVisitor<[Shape]> for Visitor<'a> {
        #[inline]
        fn visit(&mut self, shapes: &[Shape]) {
            let order = ShapeSequence::new(shapes.to_vec());
            self.result.add_success(&order);
        }
    }

    #[inline]
    pub(crate) fn success_rates<T>(
        move_rules: &MoveRules<T>,
        clipped_board: ClippedBoard,
        patterns: &Pattern,
        allows_hold: bool,
    ) -> PcRateResult where T: RotationSystem {
        assert_eq!(clipped_board.spaces() % 4, 0, "Unexpected the count of board spaces.");
        assert!(clipped_board.spaces() / 4 <= patterns.dim_shapes() as u32, "The pattern does not have enough blocks for PC.");

        let allows_hold = allows_hold && (clipped_board.spaces() / 4 < patterns.dim_shapes() as u32);

        fn search_pc_order<T>(move_rules: &MoveRules<T>, sequence: ShapeOrder, clipped_board: ClippedBoard, allows_hold: bool) -> Option<ShapeSequence> where T: RotationSystem {
            #[inline]
            fn pop_shape<T>(move_rules: &MoveRules<T>, cursor: OrderCursor, clipped_board: ClippedBoard, buffer: &mut Vec<Shape>, index_: usize, parity: &[i32; 2], allows_hold: bool) -> Option<ShapeSequence> where T: RotationSystem {
                let (popped, next_cursor) = cursor.pop(PopOp::First);
                if let Some(shape) = popped {
                    if let Some(order) = increment(move_rules, shape, clipped_board, next_cursor, buffer, index_, parity, allows_hold) {
                        return Some(order);
                    }
                } else {
                    return None;
                }

                if allows_hold {
                    let (popped, next_cursor) = cursor.pop(PopOp::Second);
                    if let Some(shape) = popped {
                        if let Some(order) = increment(move_rules, shape, clipped_board, next_cursor, buffer, index_, parity, allows_hold) {
                            return Some(order);
                        }
                    }
                }

                None
            }

            #[inline]
            fn validates_by_parity(shapes: &[Shape], index: usize, parity: [i32; 2], allows_hold: bool) -> bool {
                debug_assert!(0 < shapes.len());
                debug_assert!((parity[0] + parity[1]) <= (shapes.len() * 4) as i32);

                if parity[0] < 0 || parity[1] < 0 {
                    return false;
                }

                if parity == [0, 0] {
                    return true;
                }

                let vertical_parity: &[(i32, i32)] = match shapes[index] {
                    Shape::T => &[(2, 2), (1, 3)],
                    Shape::I => &[(2, 2), (0, 4)],
                    Shape::L | Shape::J => &[(1, 3)],
                    Shape::O | Shape::S | Shape::Z => &[(2, 2)],
                };

                for (left, right) in vertical_parity {
                    if validates_by_parity(shapes, index + 1, [parity[0] - left, parity[1] - right], allows_hold) {
                        return true;
                    }

                    if left != right {
                        if validates_by_parity(shapes, index + 1, [parity[0] - right, parity[1] - left], allows_hold) {
                            return true;
                        }
                    }

                    if allows_hold {
                        if validates_by_parity(shapes, index + 1, parity.clone(), false) {
                            return true;
                        }
                    }
                }

                false
            }

            #[inline]
            fn increment<T>(move_rules: &MoveRules<T>, shape: Shape, clipped_board: ClippedBoard, next_cursor: OrderCursor, buffer: &mut Vec<Shape>, index: usize, parity: &[i32; 2], allows_hold: bool) -> Option<ShapeSequence> where T: RotationSystem {
                buffer[index] = shape;
                let index = index + 1;

                const POSITION: BlPosition = bl(5, 20);
                let placement = shape.with(Orientation::North).with(POSITION);
                let moves = move_rules.generate_minimized_moves(clipped_board.board, placement);

                for placement in moves {
                    if clipped_board.height as i32 <= placement.tr_placement().position.ty {
                        continue;
                    }

                    let mut board = clipped_board.board.clone();
                    let lines_cleared = placement.place_on_and_clear_lines(&mut board).unwrap();
                    if board.is_empty() {
                        return Some(ShapeSequence::new(buffer[0..index].to_vec()));
                    }

                    let next_clipped_board = ClippedBoard::new(board, clipped_board.height - lines_cleared.count());

                    let mut parity = parity.clone();
                    for location in placement.locations() {
                        parity[(location.x % 2) as usize] -= 1;
                    }

                    if !validate(&next_clipped_board) {
                        continue;
                    }

                    let shape_order = next_cursor.unused_shapes();
                    let rest_shapes = shape_order.shapes();
                    if !validates_by_parity(rest_shapes, 0, parity, allows_hold) {
                        continue;
                    }

                    if let Some(order) = pop_shape(move_rules, next_cursor, next_clipped_board, buffer, index, &parity, allows_hold) {
                        return Some(order);
                    }
                }

                None
            }

            let mut parity: [i32; 2] = [0; 2];
            for y in 0..clipped_board.height {
                let y = y as i32;
                for x in 0..10 {
                    if clipped_board.board.is_free_at(xy(x, y)) {
                        parity[(x % 2) as usize] += 1;
                    }
                }
            }

            let cursor = sequence.new_cursor();
            let mut buffer = {
                let size = cursor.len_unused();
                let mut vec = Vec::with_capacity(size);
                vec.resize(size, Shape::T);
                vec
            };

            pop_shape(move_rules, cursor, clipped_board, &mut buffer, 0, &parity, allows_hold)
        }

        let orders = patterns.to_sequences();
        let infer_size = patterns.dim_shapes();

        let mut result = PcRateResult::new(&orders);

        for order in orders {
            if result.is_succeed(&order).unwrap() {
                continue;
            }

            let mut visitor = Visitor { result: &mut result };

            let sequence = order.to_order();
            if let Some(order) = search_pc_order(move_rules, sequence, clipped_board, allows_hold) {
                order.infer_input_walk(infer_size, &mut visitor);
            }
        }

        result
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Default)]
pub struct PcRateResult {
    succeed: FxHashMap<ShapeSequence, bool>,
}

impl PcRateResult {
    #[inline]
    pub fn new(orders: &Vec<ShapeSequence>) -> Self {
        let mut succeed = FxHashMap::<ShapeSequence, bool>::default();
        succeed.reserve(orders.len());
        for order in orders {
            succeed.insert(order.clone(), false);
        }
        Self { succeed }
    }

    #[inline]
    fn add_success(&mut self, order: &ShapeSequence) {
        if self.succeed.contains_key(&order) {
            self.succeed.insert(order.clone(), true);
        }
    }

    #[inline]
    pub fn contains(&self, order: &ShapeSequence) -> bool {
        self.succeed.contains_key(order)
    }

    #[inline]
    pub fn is_succeed(&self, order: &ShapeSequence) -> Option<bool> {
        self.succeed.get(order).map(|it| *it)
    }

    #[inline]
    pub fn is_failed(&self, order: &ShapeSequence) -> Option<bool> {
        self.is_succeed(order).map(|it| !it)
    }

    #[inline]
    pub fn orders(&self) -> Keys<'_, ShapeSequence, bool> {
        self.succeed.keys()
    }

    #[inline]
    pub fn count_success(&self) -> usize {
        self.succeed.values().filter(|it| **it).count()
    }
}

#[inline]
pub fn pc_success_rates<T>(
    move_rules: &MoveRules<T>,
    board: Board64,
    height: u32,
    patterns: &Pattern,
    allows_hold: bool,
) -> PcRateResult where T: RotationSystem {
    let clipped_board = ClippedBoard::new(board, height);
    pc_rate::success_rates(move_rules, clipped_board, patterns, allows_hold)
}


#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use bitris::{Board64, Drop, MoveRules, Shape};

    use crate::{Pattern, ShapeCounter, ShapeSequence};
    use crate::commands::{pc_success_rates, PcRateResult};

    #[test]
    fn pc_rate_result() {
        use Shape::*;
        let mut result = PcRateResult::new(&vec![
            ShapeSequence::new(vec!(I, T, O)),
            ShapeSequence::new(vec!(I, T, S)),
            ShapeSequence::new(vec!(I, T, Z)),
        ]);

        result.add_success(&ShapeSequence::new(vec!(I, T, O)));

        assert!(result.contains(&ShapeSequence::new(vec!(I, T, O))));
        assert!(result.contains(&ShapeSequence::new(vec!(I, T, S))));
        assert!(!result.contains(&ShapeSequence::new(vec!(I, T, J))));

        assert_eq!(result.is_succeed(&ShapeSequence::new(vec!(I, T, O))), Some(true));
        assert_eq!(result.is_succeed(&ShapeSequence::new(vec!(I, T, S))), Some(false));
        assert_eq!(result.is_succeed(&ShapeSequence::new(vec!(I, T, J))), None);

        assert_eq!(result.is_failed(&ShapeSequence::new(vec!(I, T, O))), Some(false));
        assert_eq!(result.is_failed(&ShapeSequence::new(vec!(I, T, S))), Some(true));
        assert_eq!(result.is_failed(&ShapeSequence::new(vec!(I, T, J))), None);

        assert_eq!(result.orders().len(), 3);
        assert_eq!(result.count_success(), 1);
    }

    #[test]
    fn success_rates_grace_system() {
        use crate::PatternElement::*;
        let board = Board64::from_str("
            ######....
            ######....
            ######....
            ######....
        ").unwrap();
        let patterns = Pattern::new(vec![
            One(Shape::T),
            Permutation(ShapeCounter::one_of_each(), 4),
        ]);
        const HEIGHT: u32 = 4;
        let move_rules = MoveRules::srs(Drop::Softdrop);
        let result = pc_success_rates(&move_rules, board, HEIGHT, &patterns, true);
        assert_eq!(result.count_success(), 744);
    }

    #[test]
    #[should_panic]
    fn fewer_patterns() {
        use crate::PatternElement::*;
        let board = Board64::from_str("
            ######....
            ######....
        ").unwrap();
        let patterns = Pattern::new(vec![
            One(Shape::O),
        ]);
        const HEIGHT: u32 = 2;
        let move_rules = MoveRules::srs(Drop::Softdrop);
        pc_success_rates(&move_rules, board, HEIGHT, &patterns, true);
    }
}
