use bitris::prelude::*;
use fxhash::FxHashSet;
use thiserror::Error;

use crate::{ClippedBoard, ForEachVisitor, OrderCursor, Pattern, PopOp, ShapeOrder, ShapeSequence};
use crate::internals::{FuzzyShape, FuzzyShapeOrder};
use crate::pc_possible::{Buffer, PcResults, VerticalParity};
use crate::pc_possible::bulk_executor::ExecuteInstruction::Continue;

struct Visitor<'a> {
    result: &'a mut PcResults,
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
        self.result.accept_if_present(&order, true);
    }
}


/// Dataset for detecting the same state during PC possible search.
/// The block counts and height on the board can determine the search depth. (Placed pieces will change the block counts.)
/// If the search depth is the same and the head of shapes is the same, they are the same states.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default, Debug)]
struct SearchingState {
    // The board does not include filled rows.
    board: Board64,

    height: u32,

    first: Option<Shape>,
}


#[inline]
fn validate_board(clipped: &ClippedBoard) -> bool {
    let wall = (1 << clipped.height()) - 1;
    let mut frees_sum = clipped.height() - clipped.board_ref().cols[0].count_ones();

    for x in 1..10 {
        let frees_in_column = clipped.height() - clipped.board_ref().cols[x].count_ones();
        if (clipped.board_ref().cols[x - 1] | clipped.board_ref().cols[x]) == wall {
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


/// A collection of errors that occur when making the executor.
#[derive(Error, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum PcPossibleExecutorBulkCreationError {
    #[error("Unexpected the count of board spaces.")]
    UnexpectedBoardSpaces,
    #[error("The pattern is too short to take a PC.")]
    ShortPatternDimension,
    #[error("The pattern does not have shape sequences.")]
    PatternIsEmpty,
}

/// The executor to find PC possibles.
#[derive(Clone, PartialEq, PartialOrd, Hash, Debug)]
pub struct PcPossibleBulkExecutor<'a, T: RotationSystem> {
    move_rules: &'a MoveRules<T>,
    clipped_board: ClippedBoard,
    pattern: &'a Pattern,
    allows_hold: bool,
}

/// A collection of statements that instruct execution to continue/stop.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default, Debug)]
pub enum ExecuteInstruction {
    #[default] Continue,
    Stop,
}

impl<'a, T: RotationSystem> PcPossibleBulkExecutor<'a, T> {
    /// Make PcPossibleBulkExecutor.
    ///
    /// Returns `Err()` if the setting is incorrect.
    /// See `PcPosibleExecutorCreationError` for error patterns.
    /// ```
    /// use std::str::FromStr;
    /// use bitris::{Shape, Board64, MoveRules, MoveType};
    /// use bitris_commands::{ClippedBoard, Pattern, PatternElement, ShapeCounter};
    /// use bitris_commands::pc_possible::PcPossibleBulkExecutor;
    ///
    /// let move_rules = MoveRules::srs(MoveType::Softdrop);
    ///
    /// let board = Board64::from_str("
    ///     XXX.....XX
    ///     XXX....XXX
    ///     XXX...XXXX
    ///     XXX....XXX
    /// ").expect("Failed to create a board.");
    /// let height = 4;
    /// let clipped_board = ClippedBoard::try_new(board, height).expect("Failed to clip.");
    ///
    /// // Defines available shape sequences. Below represents 840 sequences to take out four from all shapes.
    /// let pattern = Pattern::new(vec![
    ///     PatternElement::One(Shape::I),
    ///     PatternElement::Permutation(ShapeCounter::one_of_each(), 4),
    /// ]);
    ///
    /// let allows_hold = true;
    ///
    /// let executor = PcPossibleBulkExecutor::try_new(&move_rules, clipped_board, &pattern, allows_hold)
    ///     .expect("Failed to create an executor.");
    ///
    /// let results = executor.execute();
    /// assert_eq!(results.count_succeed(), 711);
    /// assert_eq!(results.count_failed(), 129);
    /// assert_eq!(results.count_accepted(), 840);
    /// ```
    pub fn try_new(
        move_rules: &'a MoveRules<T>,
        clipped_board: ClippedBoard,
        pattern: &'a Pattern,
        allows_hold: bool,
    ) -> Result<Self, PcPossibleExecutorBulkCreationError> {
        use crate::pc_possible::PcPossibleExecutorBulkCreationError::*;

        if clipped_board.spaces() % 4 != 0 {
            return Err(UnexpectedBoardSpaces);
        }

        if pattern.len_shapes_vec() == 0 {
            return Err(PatternIsEmpty);
        }

        if (pattern.dim_shapes() as u32) < clipped_board.spaces() / 4 {
            return Err(ShortPatternDimension);
        }

        debug_assert!(0 < clipped_board.spaces());

        let allows_hold = allows_hold && (clipped_board.spaces() / 4 < pattern.dim_shapes() as u32);

        Ok(Self { move_rules, clipped_board, pattern, allows_hold })
    }

    /// Start the search for PC possible in bulk.
    /// ```
    /// use std::str::FromStr;
    /// use bitris::{Board64, MoveRules, MoveType};
    /// use bitris_commands::{ClippedBoard, Pattern, PatternElement, ShapeCounter};
    /// use bitris_commands::pc_possible::PcPossibleBulkExecutor;
    ///
    /// let move_rules = MoveRules::srs(MoveType::Softdrop);
    ///
    /// let board = Board64::from_str("
    ///     XXXX....XX
    ///     XXXX...XXX
    ///     XXXX..XXXX
    ///     XXXX...XXX
    /// ").expect("Failed to create a board.");
    /// let height = 4;
    /// let clipped_board = ClippedBoard::try_new(board, height).expect("Failed to clip.");
    ///
    /// let pattern = Pattern::new(vec![
    ///     PatternElement::Permutation(ShapeCounter::one_of_each(), 4),
    /// ]);
    ///
    /// let allows_hold = true;
    ///
    /// let executor = PcPossibleBulkExecutor::try_new(&move_rules, clipped_board, &pattern, allows_hold)
    ///     .expect("Failed to create an executor.");
    ///
    /// let results = executor.execute();
    /// assert_eq!(results.count_succeed(), 514);
    /// assert_eq!(results.count_failed(), 326);
    /// assert_eq!(results.count_accepted(), 840);
    /// ```
    pub fn execute(&self) -> PcResults {
        self.execute_with_early_stopping(move |_| Continue)
    }

    /// Start the search for PC possible in bulk with early stopping.
    /// If the clojure returns `ExecuteInstruction::Stop`, it stops.
    /// ```
    /// use std::str::FromStr;
    /// use bitris::{Board64, MoveRules, MoveType};
    /// use bitris_commands::{ClippedBoard, Pattern, PatternElement, ShapeCounter};
    /// use bitris_commands::pc_possible::{ExecuteInstruction, PcPossibleBulkExecutor};
    ///
    /// let move_rules = MoveRules::srs(MoveType::Softdrop);
    ///
    /// let board = Board64::from_str("
    ///     ......####
    ///     .....#####
    ///     ..#..#####
    ///     .#....####
    /// ").expect("Failed to create a board.");
    /// let height = 4;
    /// let clipped_board = ClippedBoard::try_new(board, height).expect("Failed to clip.");
    ///
    /// let pattern = Pattern::new(vec![
    ///     PatternElement::Permutation(ShapeCounter::one_of_each(), 5),
    /// ]);
    ///
    /// let allows_hold = false;
    ///
    /// let executor = PcPossibleBulkExecutor::try_new(&move_rules, clipped_board, &pattern, allows_hold)
    ///     .expect("Failed to create an executor.");
    ///
    /// // Stops after 10 failures.
    /// let result = executor.execute_with_early_stopping(|results| {
    ///     if results.count_failed() < 10 {
    ///         ExecuteInstruction::Continue
    ///     } else {
    ///         ExecuteInstruction::Stop
    ///     }
    /// });
    /// assert_eq!(result.count_failed(), 10);
    ///
    /// // Unexplored sequences will exist.
    /// assert!(result.count_accepted() < 2520); // under 2520 = 7*6*5*4*3 sequences
    /// assert!(0 < result.count_pending());
    /// ```
    pub fn execute_with_early_stopping(&self, early_stopping: impl Fn(&PcResults) -> ExecuteInstruction) -> PcResults {
        let sequences = self.pattern.to_sequences();
        let infer_size = self.pattern.dim_shapes();

        let mut results = PcResults::new(&sequences);

        let mut visited_states = FxHashSet::<SearchingState>::default();

        for sequence in sequences {
            if let Some(_) = results.get(&sequence) {
                if early_stopping(&results) == ExecuteInstruction::Stop {
                    break;
                }
                continue;
            }

            visited_states.clear();

            let order = sequence.to_shape_order();
            if let Some(sequence_pc) = self.search_pc_order(self.clipped_board, order, &mut visited_states) {
                results.accept_if_present(&sequence, true);

                if self.allows_hold {
                    let mut visitor = Visitor { result: &mut results };
                    sequence_pc.infer_input_walk(infer_size, &mut visitor);
                }
            } else {
                results.accept_if_present(&sequence, false);
            }

            if early_stopping(&results) == ExecuteInstruction::Stop {
                break;
            }
        }

        results
    }

    fn search_pc_order(
        &self,
        current_clipped_board: ClippedBoard,
        order: ShapeOrder,
        visited_states: &mut FxHashSet::<SearchingState>,
    ) -> Option<ShapeSequence> {
        let cursor = order.new_cursor();
        let mut buffer = Buffer::with_resized(cursor.len_unused());
        let parity = VerticalParity::new(current_clipped_board);

        self.pop_shape(cursor, current_clipped_board, visited_states, &mut buffer, &parity)
    }

    fn pop_shape(
        &self,
        cursor: OrderCursor,
        clipped_board: ClippedBoard,
        visited_states: &mut FxHashSet::<SearchingState>,
        buffer: &mut Buffer,
        parity: &VerticalParity,
    ) -> Option<ShapeSequence> {
        let (popped, next_cursor) = cursor.pop(PopOp::First);
        if let Some(shape) = popped {
            if let Some(order) = self.increment(shape, clipped_board, next_cursor, visited_states, buffer, parity) {
                return Some(order);
            }
        } else {
            return None;
        }

        if self.allows_hold {
            let (popped, next_cursor) = cursor.pop(PopOp::Second);
            if let Some(shape) = popped {
                if let Some(order) = self.increment(shape, clipped_board, next_cursor, visited_states, buffer, parity) {
                    return Some(order);
                }
            }
        }

        None
    }

    fn increment(
        &self,
        shape: Shape,
        clipped_board: ClippedBoard,
        next_cursor: OrderCursor,
        visited_states: &mut FxHashSet::<SearchingState>,
        buffer: &mut Buffer,
        parity: &VerticalParity,
    ) -> Option<ShapeSequence> {
        buffer.increment(shape);

        const POSITION: BlPosition = bl(5, 20);
        let placement = shape.with(Orientation::North).with(POSITION);
        let moves = self.move_rules.generate_minimized_moves(clipped_board.board(), placement);

        for placement in moves {
            if clipped_board.height() as i32 <= placement.tr_placement().position.ty {
                continue;
            }

            let mut board = clipped_board.board();
            let lines_cleared = placement.place_on_and_clear_lines(&mut board).unwrap();
            if board.is_empty() {
                return Some(ShapeSequence::new(buffer.as_slice().to_vec()));
            }

            let height = clipped_board.height() - lines_cleared.count();
            if !visited_states.insert(SearchingState {
                board,
                height,
                first: next_cursor.first(),
            }) {
                continue;
            }

            let next_clipped_board = ClippedBoard::new_unsafe(board, height);
            if !validate_board(&next_clipped_board) {
                continue;
            }

            let shape_order = next_cursor.unused_shapes();
            let rest_shapes = shape_order.shapes();
            let next_parity = parity.place(placement);
            if !next_parity.validates(rest_shapes, 0, self.allows_hold) {
                continue;
            }

            if let Some(order) = self.pop_shape(next_cursor, next_clipped_board, visited_states, buffer, &next_parity) {
                return Some(order);
            }
        }

        buffer.decrement();

        None
    }
}


#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use bitris::{Board64, MoveRules, MoveType, Shape};

    use crate::{ClippedBoard, Pattern, PatternElement, ShapeCounter, ShapeSequence};
    use crate::pc_possible::{PcPossibleBulkExecutor, PcPossibleExecutorBulkCreationError};

    #[test]
    fn success_rate_contain_filled_line() {
        use PatternElement::*;
        use Shape::*;
        let board = Board64::from_str("
            ####....##
            #####..###
            ##########
            #####..###
        ").unwrap();
        let clipped_board = ClippedBoard::try_new(board, 4).unwrap();
        let pattern = Pattern::new(vec![
            Permutation(ShapeCounter::one_of_each(), 3),
        ]);
        let move_rules = MoveRules::srs(MoveType::Softdrop);
        let executor = PcPossibleBulkExecutor::try_new(
            &move_rules, clipped_board, &pattern, true,
        ).unwrap();
        let result = executor.execute();
        assert_eq!(result.count_succeed(), 90);
        assert_eq!(result.count_pending(), 0);

        assert_eq!(result.get(&ShapeSequence::new(vec![O, I, T])), Some(true));
        assert_eq!(result.get(&ShapeSequence::new(vec![S, T, Z])), Some(true));
        assert_eq!(result.get(&ShapeSequence::new(vec![T, L, J])), Some(true));
        assert_eq!(result.get(&ShapeSequence::new(vec![S, O, L])), Some(false));
        assert_eq!(result.get(&ShapeSequence::new(vec![O, O, O])), None);
    }

    #[test]
    fn error_unexpected_board_spaces() {
        use crate::PatternElement::*;
        let board = Board64::from_str("
            ######...#
            ######..##
        ").unwrap();
        let clipped_board = ClippedBoard::try_new(board, 2).unwrap();
        let pattern = Pattern::new(vec![
            One(Shape::O),
            One(Shape::O),
        ]);
        let move_rules = MoveRules::srs(MoveType::Softdrop);
        assert_eq!(
            PcPossibleBulkExecutor::try_new(&move_rules, clipped_board, &pattern, true).unwrap_err(),
            PcPossibleExecutorBulkCreationError::UnexpectedBoardSpaces,
        );
    }

    #[test]
    fn error_short_pattern() {
        use crate::PatternElement::*;
        let board = Board64::from_str("
            ######....
            ######....
        ").unwrap();
        let clipped_board = ClippedBoard::try_new(board, 2).unwrap();
        let pattern = Pattern::new(vec![
            One(Shape::O),
        ]);
        let move_rules = MoveRules::srs(MoveType::Softdrop);
        assert_eq!(
            PcPossibleBulkExecutor::try_new(&move_rules, clipped_board, &pattern, true).unwrap_err(),
            PcPossibleExecutorBulkCreationError::ShortPatternDimension,
        );
    }

    #[test]
    fn error_pattern_is_empty() {
        let board = Board64::from_str("
            ######....
            ######....
        ").unwrap();
        let clipped_board = ClippedBoard::try_new(board, 2).unwrap();
        let pattern = Pattern::default();
        let move_rules = MoveRules::srs(MoveType::Softdrop);
        assert_eq!(
            PcPossibleBulkExecutor::try_new(&move_rules, clipped_board, &pattern, true).unwrap_err(),
            PcPossibleExecutorBulkCreationError::PatternIsEmpty,
        );
    }
}
