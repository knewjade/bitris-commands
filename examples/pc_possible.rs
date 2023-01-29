use std::ops::Not;
use std::rc::Rc;
use std::slice::Iter;
use std::str::FromStr;

use bitris_commands::pc_possible::*;
use bitris_commands::prelude::*;

// Finds a PC with SRS.
fn srs() {
    // Makes a binder with SRS. The rotation system changes require the re-making of the binder because of the type system.
    // Default values are already set in Binder. Please check the documentation of `PcPossibleExecutorBinder::default()` for details.
    let mut binder = PcPossibleExecutorBinder::srs();

    // Set softdrop to allow move.
    binder.allow_move = AllowMove::Softdrop;

    // Sets a board and goal for 4 lines PC.
    let board = Board64::from_str("
            ###.....##
            ###....###
            ###...####
            ###....###
        ").expect("Failed to create a board");
    let height = 4;
    binder.clipped_board = ClippedBoard::try_new(board, height).expect("Failed to clip");

    // Sets sequences in which you want the PC to be checked.
    use Shape::*;
    binder.shape_order = Rc::from(ShapeOrder::new(vec![
        I, T, O, L, J,
    ]));

    // Whether or not to allow hold.
    binder.allows_hold = true;

    // Finds a PC. If it contains an invalid configuration, an error is returned.
    let succeed = binder.try_execute().expect("Failed to execute");
    assert!(succeed); // PC possible

    // The binder is reusable.
    binder.shape_order = Rc::from(ShapeOrder::new(vec![
        S, S, S, S,
    ]));
    let succeed = binder.try_execute().expect("Failed to execute");
    assert!(succeed.not()); // PC impossible
}

// Finds PCs in bulk with SRS.
fn bulk_with_srs() {
    // You can also process them all together efficiently with `PcPossibleBulkExecutorBinder`.
    let mut binder = PcPossibleBulkExecutorBinder::srs();

    // Sets sequences in which you want the PC to be checked.
    // The following represents 'I****'.
    let pattern = Pattern::try_from(vec![
        PatternElement::One(Shape::I),
        PatternElement::Permutation(ShapeCounter::one_of_each(), 4),
    ]).expect("Failed to create a pattern");
    binder.pattern = Rc::from(pattern);

    // The others are the same.
    let board = Board64::from_str("
            ###.....##
            ###....###
            ###...####
            ###....###
        ").expect("Failed to create a board");
    let height = 4;
    binder.clipped_board = ClippedBoard::try_new(board, height).expect("Failed to clip");
    binder.allow_move = AllowMove::Softdrop;
    binder.allows_hold = true;

    // Finds PCs. If it contains an invalid configuration, an error is returned.
    let results = binder.try_execute().expect("Failed to execute");

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
fn bulk_using_early_stopping() {
    // Makes a binder.
    let mut binder = PcPossibleBulkExecutorBinder::srs();

    // The following represents the use of all shapes one at a time.
    let pattern = Pattern::try_from(vec![
        PatternElement::Factorial(ShapeCounter::one_of_each()),
    ]).expect("Failed to create a pattern");
    binder.pattern = Rc::from(pattern);

    let board = Board64::from_str("
            ..........
            ....####..
            ....######
            ....######
        ").expect("Failed to create a board");
    let height = 4;
    binder.clipped_board = ClippedBoard::try_new(board, height).expect("Failed to clip");
    binder.allows_hold = false;

    // Executes and stops after 10 failures.
    let result = binder.try_execute_with_early_stopping(|results| {
        if results.count_failed() < 10 {
            ExecuteInstruction::Continue
        } else {
            ExecuteInstruction::Stop
        }
    }).expect("Failed to execute");
    assert_eq!(result.count_failed(), 10);

    // Unexplored sequences will exist.
    assert!(result.count_accepted() < 5040); // Terminated before All sequences are accepted.
    assert!(0 < result.count_pending()); // There are still sequences to be explored.
}

// Use with customized kicks
fn customized_kick() {
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

    // Makes a binder with customized kicks.
    let rotation_system = Rc::from(MyKickTable);
    let mut binder = PcPossibleBulkExecutorBinder::default(rotation_system);

    // The others are the same.
    let board = Board64::from_str("
            ###.....##
            ###....###
            ###...####
            ###....###
        ").expect("Failed to create a board");
    let height = 4;
    binder.clipped_board = ClippedBoard::try_new(board, height).expect("Failed to clip");
    binder.pattern = Rc::from(Pattern::try_from(vec![
        PatternElement::One(Shape::I),
        PatternElement::Permutation(ShapeCounter::one_of_each(), 4),
    ]).expect("Failed to create a pattern"));

    let results = binder.try_execute().expect("Failed to execute");
    assert_eq!(results.count_succeed(), 485);
    assert_eq!(results.count_accepted(), 840);
}

// Execute more efficiently.
fn use_the_executor_directly() {
    // You can also make an executor directly.
    // However, you will need to understand the more internal structures and the lifetime.

    let move_rules = MoveRules::new(&SrsKickTable, AllowMove::Softdrop);

    let board = Board64::from_str("
            ###.....##
            ###....###
            ###...####
            ###....###
        ").expect("Failed to create a board");
    let height = 4;
    let clipped_board = ClippedBoard::try_new(board, height).expect("Failed to clip");

    let pattern = Pattern::try_from(vec![
        PatternElement::One(Shape::I),
        PatternElement::Permutation(ShapeCounter::one_of_each(), 4),
    ]).expect("Failed to create a pattern");

    let allows_hold = true;

    let executor = PcPossibleBulkExecutor::try_new(
        move_rules,
        clipped_board,
        &pattern,
        allows_hold,
    ).expect("Failed to make an executor");

    let results = executor.execute();
    assert_eq!(results.count_succeed(), 711);
    assert_eq!(results.count_accepted(), 840);
}

fn main() {
    srs();
    bulk_with_srs();
    bulk_using_early_stopping();
    customized_kick();
    use_the_executor_directly();
}
