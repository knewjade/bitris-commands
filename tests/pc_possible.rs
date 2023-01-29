#[cfg(test)]
mod tests {
    use std::rc::Rc;
    use std::str::FromStr;

    use bitris_commands::pc_possible::*;
    use bitris_commands::prelude::*;

    struct PcPossibleTestingData {
        id: String,
        succeed: u64,
        accepted: u64,
        generator: fn() -> PcPossibleBulkExecutorBinder<SrsKickTable>,
    }

    #[test]
    fn srs() {
        use PatternElement::*;
        use Shape::*;

        let benchmarks = vec![
            PcPossibleTestingData {
                id: format!("1st-ILSZ-hold"),
                generator: || {
                    let mut binder = PcPossibleBulkExecutorBinder::srs();

                    let board = Board64::from_str("
                        #......###
                        #.......##
                        #.....####
                        #......###
                    ").unwrap();
                    let height = 4;
                    binder.clipped_board = ClippedBoard::try_new(board, height).unwrap();

                    binder.pattern = Rc::from(Pattern::try_from(vec![
                        Factorial(ShapeCounter::try_from(vec![
                            L, T, O,
                        ]).unwrap()),
                        Permutation(ShapeCounter::one_of_each(), 4),
                    ]).unwrap());

                    binder.allows_hold = true;

                    binder
                },
                succeed: 5040,
                accepted: 5040,
            },
            PcPossibleTestingData {
                id: format!("1st-ILSZ-no-hold"),
                generator: || {
                    let mut binder = PcPossibleBulkExecutorBinder::srs();

                    let board = Board64::from_str("
                        #......###
                        #.......##
                        #.....####
                        #......###
                    ").unwrap();
                    let height = 4;
                    binder.clipped_board = ClippedBoard::try_new(board, height).unwrap();

                    binder.pattern = Rc::from(Pattern::try_from(vec![
                        Factorial(ShapeCounter::try_from(vec![
                            L, T, O,
                        ]).unwrap()),
                        Permutation(ShapeCounter::one_of_each(), 3),
                    ]).unwrap());

                    binder.allows_hold = false;

                    binder
                },
                succeed: 523,
                accepted: 1260,
            },
            PcPossibleTestingData {
                id: format!("1st-grace-system-hold"),
                generator: || {
                    let mut binder = PcPossibleBulkExecutorBinder::srs();

                    let board = Board64::from_str("
                        ######....
                        ######....
                        ######....
                        ######....
                    ").unwrap();
                    let height = 4;
                    binder.clipped_board = ClippedBoard::try_new(board, height).unwrap();

                    binder.pattern = Rc::from(Pattern::try_from(vec![
                        One(T),
                        Permutation(ShapeCounter::one_of_each(), 4),
                    ]).unwrap());

                    binder
                },
                succeed: 744,
                accepted: 840,
            },
            PcPossibleTestingData {
                id: format!("1st-grace-system-no-hold"),
                generator: || {
                    let mut binder = PcPossibleBulkExecutorBinder::srs();

                    let board = Board64::from_str("
                        ######....
                        ######....
                        ######....
                        ######....
                    ").unwrap();
                    let height = 4;
                    binder.clipped_board = ClippedBoard::try_new(board, height).unwrap();

                    binder.pattern = Rc::from(Pattern::try_from(vec![
                        One(T),
                        Permutation(ShapeCounter::one_of_each(), 3),
                    ]).unwrap());

                    binder.allows_hold = false;

                    binder
                },
                succeed: 67,
                accepted: 210,
            },
            PcPossibleTestingData {
                id: format!("2nd-LSZT"),
                generator: || {
                    let mut binder = PcPossibleBulkExecutorBinder::srs();

                    let board = Board64::from_str("
                        #.........
                        ##...#....
                        ######....
                        ######....
                    ").unwrap();
                    let height = 4;
                    binder.clipped_board = ClippedBoard::try_new(board, height).unwrap();

                    binder
                },
                succeed: 5028,
                accepted: 5040,
            },
            PcPossibleTestingData {
                id: format!("empty"),
                generator: || {
                    let mut binder = PcPossibleBulkExecutorBinder::srs();

                    binder.pattern = Rc::from(Pattern::try_from(vec![
                        Fixed(BitShapes::try_from(vec![
                            S, L, Z, O, S, L, S, J, O, Z,
                        ]).unwrap()),
                        Wildcard, // I or O is not PC-able
                    ]).unwrap());

                    binder.allows_hold = true;

                    binder
                },
                succeed: 5,
                accepted: 7,
            },
            PcPossibleTestingData {
                id: format!("harddrop-only"),
                generator: || {
                    let mut binder = PcPossibleBulkExecutorBinder::srs();

                    let board = Board64::from_str("
                        ######....
                        ######....
                        ######....
                        ######....
                    ").unwrap();
                    let height = 4;
                    binder.clipped_board = ClippedBoard::try_new(board, height).unwrap();

                    binder.pattern = Rc::from(Pattern::try_from(vec![
                        Permutation(ShapeCounter::one_of_each(), 5),
                    ]).unwrap());

                    binder.allow_move = AllowMove::Harddrop;

                    binder
                },
                succeed: 1552,
                accepted: 2520,
            },
            PcPossibleTestingData {
                id: format!("contains-no-extra-piece-hold"),
                generator: || {
                    let mut binder = PcPossibleBulkExecutorBinder::srs();

                    let board = Board64::from_str("
                        ######....
                        ######....
                        ######....
                        ######....
                    ").unwrap();
                    let height = 4;
                    binder.clipped_board = ClippedBoard::try_new(board, height).unwrap();

                    binder.pattern = Rc::from(Pattern::try_from(vec![
                        Permutation(ShapeCounter::one_of_each(), 4),
                    ]).unwrap());

                    binder.allow_move = AllowMove::Harddrop;

                    binder
                },
                succeed: 314,
                accepted: 840,
            },
            PcPossibleTestingData {
                id: format!("contains-no-extra-piece-no-hold"),
                generator: || {
                    let mut binder = PcPossibleBulkExecutorBinder::srs();

                    let board = Board64::from_str("
                        ######....
                        ######....
                        ######....
                        ######....
                    ").unwrap();
                    let height = 4;
                    binder.clipped_board = ClippedBoard::try_new(board, height).unwrap();

                    binder.pattern = Rc::from(Pattern::try_from(vec![
                        Permutation(ShapeCounter::one_of_each(), 4),
                    ]).unwrap());

                    binder.allow_move = AllowMove::Harddrop;
                    binder.allows_hold = false;

                    binder
                },
                succeed: 116,
                accepted: 840,
            },
        ];

        for benchmark in benchmarks {
            println!("id: {}", benchmark.id);

            let binder = (benchmark.generator)();

            let results = binder.try_execute().unwrap();

            assert_eq!(results.count_succeed(), benchmark.succeed);
            assert_eq!(results.count_accepted(), benchmark.accepted);
        }
    }
}
