use std::fmt;

use bitris::prelude::*;
use fxhash::FxHashSet;
use thiserror::Error;

use crate::{ClippedBoard, Pattern, ShapeOrder, ShapeSequence};
use crate::internals::FuzzyShapeOrder;
use crate::pc_possible::{Buffer, ExecuteInstruction, PcResults, VerticalParity};
use crate::pc_possible::executor_bulk::ExecuteInstruction::Continue;

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
#[derive(Error, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum PcPossibleExecutorBulkCreationError {
    #[error("Unexpected the count of board spaces.")]
    UnexpectedBoardSpaces,
    #[error("The pattern is too short to take a PC.")]
    ShortPatternDimension,
    #[error("Board height exceeds the upper limit. Up to 56 are supported.")]
    BoardIsTooHigh,
}

/// A collection of errors that occur when making the executor.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default, Debug)]
pub enum PcPossibleAlgorithm {
    #[default] AllPcs,
    Simulation,
}

impl fmt::Display for PcPossibleAlgorithm {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PcPossibleAlgorithm::AllPcs => write!(f, "All PCs"),
            PcPossibleAlgorithm::Simulation => write!(f, "Simulation"),
        }
    }
}

/// The executor to find PC possibles.
#[derive(Clone, PartialEq, PartialOrd, Hash, Debug)]
pub struct PcPossibleBulkExecutor<'a, T: RotationSystem> {
    move_rules: &'a MoveRules<'a, T>,
    clipped_board: ClippedBoard,
    pattern: &'a Pattern,
    allows_hold: bool,
    has_extra_shapes: bool,
    spawn_position: BlPosition,
    algorithm: PcPossibleAlgorithm,
}

impl<'a, T: RotationSystem> PcPossibleBulkExecutor<'a, T> {
    /// Make PcPossibleBulkExecutor.
    ///
    /// Returns `Err()` if the setting is incorrect or restricted.
    /// See `PcPossibleBulkExecutorCreationError` for error cases.
    /// ```
    /// use std::str::FromStr;
    /// use bitris::prelude::*;
    /// use bitris_commands::prelude::*;
    /// use bitris_commands::pc_possible::*;
    ///
    /// let move_rules = MoveRules::srs(AllowMove::Softdrop);
    ///
    /// let board = Board64::from_str("
    ///     XXX.....XX
    ///     XXX....XXX
    ///     XXX...XXXX
    ///     XXX....XXX
    /// ").expect("Failed to create a board");
    /// let height = 4;
    /// let clipped_board = ClippedBoard::try_new(board, height).expect("Failed to clip");
    ///
    /// // Defines available shape sequences. Below represents 840 sequences to take out four from all shapes.
    /// let pattern = Pattern::try_from(vec![
    ///     PatternElement::One(Shape::I),
    ///     PatternElement::Permutation(ShapeCounter::one_of_each(), 4),
    /// ]).expect("Failed to create a pattern");
    ///
    /// let allows_hold = true;
    ///
    /// let executor = PcPossibleBulkExecutor::try_new(&move_rules, clipped_board, &pattern, allows_hold, PcPossibleAlgorithm::AllPcs)
    ///     .expect("Failed to create an executor");
    ///
    /// let results = executor.execute();
    /// assert_eq!(results.count_succeed(), 711);
    /// assert_eq!(results.count_failed(), 129);
    /// assert_eq!(results.count_accepted(), 840);
    /// ```
    pub fn try_new(
        move_rules: &'a MoveRules<'a, T>,
        clipped_board: ClippedBoard,
        pattern: &'a Pattern,
        allows_hold: bool,
        algorithm: PcPossibleAlgorithm,
    ) -> Result<Self, PcPossibleExecutorBulkCreationError> {
        use PcPossibleExecutorBulkCreationError::*;

        if 56 < clipped_board.height() {
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

        Ok(Self {
            move_rules,
            clipped_board,
            pattern,
            allows_hold,
            has_extra_shapes,
            spawn_position,
            algorithm,
        })
    }

    /// Start the search for PC possible in bulk.
    pub fn execute(&self) -> PcResults {
        match self.algorithm {
            PcPossibleAlgorithm::Simulation => self.execute_with_early_stopping(move |_| Continue),
            PcPossibleAlgorithm::AllPcs => self.execute_with_early_stopping(move |_| Continue),
        }
    }

    /// Start the search for PC possible in bulk with early stopping.
    /// If the clojure returns `ExecuteInstruction::Stop`, it stops.
    /// ```
    /// use std::str::FromStr;
    /// use bitris::prelude::*;
    /// use bitris_commands::prelude::*;
    /// use bitris_commands::pc_possible::*;
    ///
    /// let move_rules = MoveRules::srs(AllowMove::Softdrop);
    ///
    /// let board = Board64::from_str("
    ///     ......####
    ///     .....#####
    ///     ..#..#####
    ///     .#....####
    /// ").expect("Failed to create a board");
    /// let height = 4;
    /// let clipped_board = ClippedBoard::try_new(board, height).expect("Failed to clip");
    ///
    /// let pattern = Pattern::try_from(vec![
    ///     PatternElement::Permutation(ShapeCounter::one_of_each(), 5),
    /// ]).expect("Failed to create a pattern");
    ///
    /// let allows_hold = false;
    ///
    /// let executor = PcPossibleBulkExecutor::try_new(&move_rules, clipped_board, &pattern, allows_hold, PcPossibleAlgorithm::AllPcs)
    ///     .expect("Failed to create an executor");
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
                    sequence_pc.infer_input_walk(infer_size, &mut |fuzzy_shapes| {
                        let fuzzy_shape_order = FuzzyShapeOrder::new(fuzzy_shapes.to_vec());
                        fuzzy_shape_order.expand_as_wildcard_walk(&mut |shapes| {
                            let order = ShapeSequence::new(shapes.to_vec());
                            results.accept_if_present(&order, true);
                        });
                    });
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

    /// This function is dedicated to a single sequence because .
    /// The interface is not directly exposed since it's a shortcut to improve speed.
    pub(crate) fn execute_single(&self) -> bool {
        let sequences = self.pattern.to_sequences();
        assert_eq!(sequences.len(), 1, "This function is dedicated to a single sequence.");
        let order = sequences.first().unwrap().to_shape_order();

        let mut visited_states = FxHashSet::<SearchingState>::default();
        self.search_pc_order(self.clipped_board, order, &mut visited_states).is_some()
    }

    fn search_pc_order(
        &self,
        current_clipped_board: ClippedBoard,
        order: ShapeOrder,
        visited_states: &mut FxHashSet<SearchingState>,
    ) -> Option<ShapeSequence> {
        let cursor = order.new_cursor();
        let mut buffer = Buffer::with_resized(cursor.len_remaining());
        let parity = VerticalParity::new(current_clipped_board);

        self.pop_shape(cursor, current_clipped_board, visited_states, &mut buffer, &parity)
    }

    fn pop_shape(
        &self,
        cursor: OrderCursor<Shape>,
        clipped_board: ClippedBoard,
        visited_states: &mut FxHashSet<SearchingState>,
        buffer: &mut Buffer,
        parity: &VerticalParity,
    ) -> Option<ShapeSequence> {
        let (popped, next_cursor) = cursor.pop(PopOp::First);
        if let Some(&shape) = popped {
            if let Some(order) = self.increment(shape, clipped_board, next_cursor, visited_states, buffer, parity) {
                return Some(order);
            }
        } else {
            return None;
        }

        if self.allows_hold {
            let (popped, next_cursor) = cursor.pop(PopOp::Second);
            if let Some(&shape) = popped {
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
        next_cursor: OrderCursor<Shape>,
        visited_states: &mut FxHashSet<SearchingState>,
        buffer: &mut Buffer,
        parity: &VerticalParity,
    ) -> Option<ShapeSequence> {
        buffer.increment(shape);

        let placement = shape.with(Orientation::North).with(self.spawn_position);
        let moves = self.move_rules.generate_minimized_moves(clipped_board.board(), placement);

        for placement in moves {
            if clipped_board.height() as i32 <= placement.to_tr_placement().position.ty {
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
                first: next_cursor.peek_first().map(|&shape| shape),
            }) {
                continue;
            }

            let next_clipped_board = ClippedBoard::new_unsafe(board, height);
            if !validate_board(&next_clipped_board) {
                continue;
            }

            let remaining_shapes: Vec<Shape> = next_cursor.iter_remaining().map(|&shape| shape).collect();
            let next_parity = parity.place(placement);
            // The flag is off if the hold is enabled but does not have an extra piece (because parity is not affected by the shape order)
            if !next_parity.validates(remaining_shapes.as_slice(), 0, self.allows_hold && self.has_extra_shapes) {
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

    use bitris::prelude::*;

    use crate::{BitShapes, ClippedBoard, Pattern, PatternElement, ShapeCounter, ShapeSequence};
    use crate::pc_possible::{PcPossibleAlgorithm, PcPossibleBulkExecutor, PcPossibleExecutorBulkCreationError};

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
        let pattern = Pattern::try_from(vec![
            Permutation(ShapeCounter::one_of_each(), 3),
        ]).unwrap();
        let move_rules = MoveRules::srs(AllowMove::Softdrop);

        let executor = PcPossibleBulkExecutor::try_new(
            &move_rules, clipped_board, &pattern, true, PcPossibleAlgorithm::AllPcs,
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
    fn execute_single() {
        use PatternElement::*;
        use Shape::*;

        let board = Board64::from_str("
            ####....##
            #####..###
            #####..###
        ").unwrap();
        let clipped_board = ClippedBoard::try_new(board, 3).unwrap();
        let move_rules = MoveRules::srs(AllowMove::Softdrop);

        {
            let single_pattern = Pattern::try_from(vec![
                Fixed(BitShapes::try_from(vec![J, O, I]).unwrap()),
            ]).unwrap();
            let executor = PcPossibleBulkExecutor::try_new(
                &move_rules, clipped_board, &single_pattern, true, PcPossibleAlgorithm::AllPcs,
            ).unwrap();
            assert!(executor.execute_single());
        }
        {
            let single_pattern = Pattern::try_from(vec![
                Fixed(BitShapes::try_from(vec![J, T, I]).unwrap()),
            ]).unwrap();
            let executor = PcPossibleBulkExecutor::try_new(
                &move_rules, clipped_board, &single_pattern, true, PcPossibleAlgorithm::AllPcs,
            ).unwrap();
            assert!(!executor.execute_single());
        }
    }

    #[test]
    fn error_unexpected_board_spaces() {
        use crate::PatternElement::*;
        let board = Board64::from_str("
            ######...#
            ######..##
        ").unwrap();
        let clipped_board = ClippedBoard::try_new(board, 2).unwrap();
        let pattern = Pattern::try_from(vec![
            One(Shape::O),
            One(Shape::O),
        ]).unwrap();
        let move_rules = MoveRules::srs(AllowMove::Softdrop);
        assert_eq!(
            PcPossibleBulkExecutor::try_new(&move_rules, clipped_board, &pattern, true, PcPossibleAlgorithm::AllPcs).unwrap_err(),
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
        let pattern = Pattern::try_from(vec![
            One(Shape::O),
        ]).unwrap();
        let move_rules = MoveRules::srs(AllowMove::Softdrop);
        assert_eq!(
            PcPossibleBulkExecutor::try_new(&move_rules, clipped_board, &pattern, true, PcPossibleAlgorithm::AllPcs).unwrap_err(),
            PcPossibleExecutorBulkCreationError::ShortPatternDimension,
        );
    }

    #[test]
    fn maximum_height() {
        let height: u32 = 56;
        let mut board = Board64::blank();
        for y in 0..height as i32 {
            for x in 0..9 {
                board.set_at(xy(x, y));
            }
        }

        let clipped_board = ClippedBoard::try_new(board, height).unwrap();
        let pattern = Pattern::try_from(vec![
            PatternElement::One(Shape::I),
        ].repeat((height / 4) as usize)).unwrap();
        let move_rules = MoveRules::srs(AllowMove::Softdrop);

        let executor = PcPossibleBulkExecutor::try_new(
            &move_rules, clipped_board, &pattern, true, PcPossibleAlgorithm::AllPcs,
        ).unwrap();
        let result = executor.execute();
        assert_eq!(result.count_succeed(), 1);
    }

    #[test]
    fn error_exceeds_the_height_limit() {
        let height: u32 = 57;
        let mut board = Board64::blank();
        for y in 0..56 as i32 {
            for x in 0..9 {
                board.set_at(xy(x, y));
            }
        }
        for x in 0..6 {
            board.set_at(xy(x, 56));
        }

        let clipped_board = ClippedBoard::try_new(board, height).unwrap();
        let pattern = Pattern::try_from(vec![
            PatternElement::One(Shape::I),
        ].repeat((height / 4) as usize + 2)).unwrap();
        let move_rules = MoveRules::srs(AllowMove::Softdrop);
        assert_eq!(
            PcPossibleBulkExecutor::try_new(&move_rules, clipped_board, &pattern, true, PcPossibleAlgorithm::AllPcs).unwrap_err(),
            PcPossibleExecutorBulkCreationError::BoardIsTooHigh,
        );
    }
}
