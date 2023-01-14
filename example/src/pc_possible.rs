#[cfg(test)]
mod tests {
    use std::rc::Rc;
    use std::slice::Iter;
    use std::str::FromStr;

    use bitris_commands::pc_possible::*;
    use bitris_commands::prelude::*;

    // Finds PCs in bulk with SRS.
    #[test]
    fn bulk_with_srs() {
        // Makes a binder with SRS & Softdrop.
        // Because of the type system, changes require the re-making of the binder.
        let move_type = MoveType::Softdrop;
        let mut binder = PcPossibleBulkExecutorBinder::srs(move_type);

        // Sets a board and goal for 4 lines PC.
        let board = Board64::from_str("
            ###.....##
            ###....###
            ###...####
            ###....###
        ").expect("Failed to create a board.");
        let height = 4;
        binder.clipped_board = ClippedBoard::try_new(board, height).expect("Failed to clip.");

        // Sets sequences in which you want the PC to be checked.
        // The following represents 'I****'.
        binder.pattern = Rc::from(Pattern::new(vec![
            PatternElement::One(Shape::I),
            PatternElement::Permutation(ShapeCounter::one_of_each(), 4),
        ]));

        // Whether or not to allow hold.
        binder.allows_hold = true;

        // Binds the configuration to the executor.
        // The lifetime of this executor matches the binder.
        let executor = binder.try_bind().expect("Failed to bind");

        // Finds PCs.
        let results = executor.execute();

        // You can take the result out of the return value.
        assert_eq!(results.count_succeed(), 711);
        assert_eq!(results.count_failed(), 129);
        assert_eq!(results.count_accepted(), 840);

        // For example, the count of succeed sequences starting with IS.
        let (mut succeed, mut total) = (0, 0);
        for sequence in results.accepted_shape_sequences() {
            if sequence.shapes().starts_with(&[Shape::I, Shape::S]) {
                total += 1;
                if let Some(true) = results.get(sequence) {
                    succeed += 1;
                }
            }
        }
        assert_eq!(succeed, 98);
        assert_eq!(total, 120);

        // You can also write using `iter()`.
        assert_eq!(
            succeed,
            results.iter()
                .filter(|&(sequence, _)| sequence.shapes().starts_with(&[Shape::I, Shape::S]))
                .filter(|&(_, result)| *result == Some(true))
                .count(),
        );
    }

    // Use early stopping.
    #[test]
    fn bulk_using_early_stopping() {
        // Makes a binder with SRS & Harddrop.
        let move_type = MoveType::Harddrop;
        let mut binder = PcPossibleBulkExecutorBinder::srs(move_type);

        let board = Board64::from_str("
            ..........
            ....####..
            ....######
            ....######
        ").expect("Failed to create a board.");
        let height = 4;
        binder.clipped_board = ClippedBoard::try_new(board, height).expect("Failed to clip.");

        // The following represents the use of all shapes one at a time.
        binder.pattern = Rc::from(Pattern::new(vec![
            PatternElement::Factorial(ShapeCounter::one_of_each()),
        ]));

        binder.allows_hold = false;

        let executor = binder.try_bind().expect("Failed to bind");

        // Stops after 10 failures.
        let result = executor.execute_with_early_stopping(|results| {
            if results.count_failed() < 10 {
                ExecuteInstruction::Continue
            } else {
                ExecuteInstruction::Stop
            }
        });
        assert_eq!(result.count_failed(), 10);

        // Unexplored sequences will exist.
        assert!(result.count_accepted() < 5040); // Terminated before All sequences are accepted.
        assert!(0 < result.count_pending()); // There are still sequences to be explored.
    }

    // Use with customized kicks
    #[test]
    fn customized_kick() {
        // Makes a binder with customized kicks.
        let kick_table = MyKickTable;
        let move_type = MoveType::Softdrop;
        let move_rules = MoveRules::new(Rc::from(kick_table), move_type);
        let mut binder = PcPossibleBulkExecutorBinder::default(move_rules);

        // After that, it's the same as normal.
        let board = Board64::from_str("
            ###.....##
            ###....###
            ###...####
            ###....###
        ").expect("Failed to create a board.");
        let height = 4;
        binder.clipped_board = ClippedBoard::try_new(board, height).expect("Failed to clip.");
        binder.pattern = Rc::from(Pattern::new(vec![
            PatternElement::One(Shape::I),
            PatternElement::Permutation(ShapeCounter::one_of_each(), 4),
        ]));

        let executor = binder.try_bind().expect("Failed to bind.");
        let results = executor.execute();
        assert_eq!(results.count_succeed(), 485);
        assert_eq!(results.count_accepted(), 840);
    }

    struct MyKickTable;

    impl RotationSystem for MyKickTable {
        fn iter_kicks(&self, piece: Piece, _: Rotation) -> Iter<'_, Kick> {
            const KICK: [Kick; 1] = [Kick::new(Offset::new(0, 0))];
            match piece.shape {
                Shape::O => [].iter(), // Cannot rotate
                _ => KICK.iter(), // Rotatable, but no kick
            }
        }

        fn is_moving_in_rotation(&self, shape: Shape) -> bool {
            shape != Shape::O
        }
    }
}
