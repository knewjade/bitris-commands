#[cfg(test)]
mod tests {
    use std::rc::Rc;
    use std::str::FromStr;

    use bitris_commands::pc_possible::*;
    use bitris_commands::prelude::*;

    struct PcPossibleTestingData {
        id: String,
        succeed: usize,
        accepted: usize,
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
                    let mut binder = PcPossibleBulkExecutorBinder::srs(MoveType::Softdrop);

                    let board = Board64::from_str("
                        #......###
                        #.......##
                        #.....####
                        #......###
                    ").unwrap();
                    let height = 4;
                    binder.clipped_board = ClippedBoard::try_new(board, height).unwrap();

                    binder.pattern = Rc::from(Pattern::new(vec![
                        Factorial(ShapeCounter::try_from(vec![
                            L, T, O,
                        ]).unwrap()),
                        Permutation(ShapeCounter::one_of_each(), 4),
                    ]));

                    binder.allows_hold = true;

                    binder
                },
                succeed: 5040,
                accepted: 5040,
            },
            PcPossibleTestingData {
                id: format!("1st-ILSZ-no-hold"),
                generator: || {
                    let mut binder = PcPossibleBulkExecutorBinder::srs(MoveType::Softdrop);

                    let board = Board64::from_str("
                        #......###
                        #.......##
                        #.....####
                        #......###
                    ").unwrap();
                    let height = 4;
                    binder.clipped_board = ClippedBoard::try_new(board, height).unwrap();

                    binder.pattern = Rc::from(Pattern::new(vec![
                        Factorial(ShapeCounter::try_from(vec![
                            L, T, O,
                        ]).unwrap()),
                        Permutation(ShapeCounter::one_of_each(), 3),
                    ]));

                    binder.allows_hold = false;

                    binder
                },
                succeed: 523,
                accepted: 1260,
            },
            PcPossibleTestingData {
                id: format!("1st-grace-system-hold"),
                generator: || {
                    let mut binder = PcPossibleBulkExecutorBinder::srs(MoveType::Softdrop);

                    let board = Board64::from_str("
                        ######....
                        ######....
                        ######....
                        ######....
                    ").unwrap();
                    let height = 4;
                    binder.clipped_board = ClippedBoard::try_new(board, height).unwrap();

                    binder.pattern = Rc::from(Pattern::new(vec![
                        One(T),
                        Permutation(ShapeCounter::one_of_each(), 4),
                    ]));

                    binder
                },
                succeed: 744,
                accepted: 840,
            },
            PcPossibleTestingData {
                id: format!("1st-grace-system-no-hold"),
                generator: || {
                    let mut binder = PcPossibleBulkExecutorBinder::srs(MoveType::Softdrop);

                    let board = Board64::from_str("
                        ######....
                        ######....
                        ######....
                        ######....
                    ").unwrap();
                    let height = 4;
                    binder.clipped_board = ClippedBoard::try_new(board, height).unwrap();

                    binder.pattern = Rc::from(Pattern::new(vec![
                        One(T),
                        Permutation(ShapeCounter::one_of_each(), 3),
                    ]));

                    binder
                },
                succeed: 67,
                accepted: 210,
            },
            PcPossibleTestingData {
                id: format!("2nd-LSZT"),
                generator: || {
                    let mut binder = PcPossibleBulkExecutorBinder::srs(MoveType::Softdrop);

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
                    let mut binder = PcPossibleBulkExecutorBinder::srs(MoveType::Softdrop);

                    binder.pattern = Rc::from(Pattern::new(vec![
                        Fixed(BitShapes::try_from(vec![
                            S, L, Z, O, S, L, S, J, O, Z,
                        ]).unwrap()),
                        Wildcard, // I or O is not PC-able
                    ]));

                    binder.allows_hold = true;

                    binder
                },
                succeed: 5,
                accepted: 7,
            },
        ];

        for benchmark in benchmarks {
            println!("id: {}", benchmark.id);

            let binder = (benchmark.generator)();

            let results = binder.try_bind().unwrap().execute();

            assert_eq!(results.count_succeed(), benchmark.succeed);
            assert_eq!(results.count_accepted(), benchmark.accepted);
        }
    }
}
