#[cfg(test)]
mod tests {
    use std::rc::Rc;
    use std::str::FromStr;
    use std::time::Instant;

    use bitris_commands::pc_possible::*;
    use bitris_commands::prelude::*;

    #[test]
    fn single() {
        use Shape::*;

        struct TestingData {
            id: String,
            clipped_board: ClippedBoard,
            // (shape order, allow move, allows hold, result)
            expected: Vec<(Vec<Shape>, AllowMove, bool, bool)>,
            generator: fn() -> PcPossibleExecutorBinder<SrsKickTable>,
        }

        let testings = vec![
            TestingData {
                id: format!("pco-just"),
                clipped_board: ClippedBoard::try_new(Board64::from_str("
                    ###.....##
                    ###....###
                    ###...####
                    ###....###
                ").unwrap(), 4).unwrap(),
                generator: || PcPossibleExecutorBinder::srs(),
                expected: vec![
                    (vec![I, L, T, J], AllowMove::Softdrop, true, true),
                    (vec![I, L, J, T], AllowMove::Softdrop, false, false),
                    (vec![I, L, T, J], AllowMove::Harddrop, true, true),
                    (vec![I, L, T, J], AllowMove::Harddrop, false, false),
                ],
            },
            TestingData {
                id: format!("4th-extra"),
                clipped_board: ClippedBoard::try_new(Board64::from_str("
                    .....##...
                    ......##..
                    ##...##...
                    ##..######
                ").unwrap(), 4).unwrap(),
                generator: || PcPossibleExecutorBinder::srs(),
                expected: vec![
                    (vec![O, I, S, S, T, Z, L], AllowMove::Softdrop, true, false),
                    (vec![O, I, S, S, L, Z, J], AllowMove::Softdrop, true, true),
                    (vec![O, I, S, S, L, Z, J], AllowMove::Softdrop, false, false),
                    (vec![O, I, S, S, L, Z, J], AllowMove::Harddrop, true, false),
                ],
            },
        ];

        for testing in testings {
            println!("id: {}", testing.id);

            let mut binder = (testing.generator)();
            binder.clipped_board = testing.clipped_board;

            for (shapes, allow_move, allows_hold, succeed) in testing.expected {
                binder.shape_order = Rc::new(ShapeOrder::new(shapes));
                binder.allow_move = allow_move;
                binder.allows_hold = allows_hold;

                let start = Instant::now();
                let result = binder.try_execute().unwrap();
                let end = start.elapsed();
                println!("  {}, hold {}: {} μs", allow_move, allows_hold, end.as_micros());

                assert_eq!(result, succeed);
            }
        }
    }

    #[test]
    fn bulk() {
        use PatternElement::*;
        use Shape::*;

        struct TestingData {
            id: String,
            clipped_board: ClippedBoard,
            accepted: u64,
            // (allow move, allows hold, succeed)
            expected: Vec<(AllowMove, bool, u64)>,
            generator: fn() -> PcPossibleBulkExecutorBinder<SrsKickTable>,
        }

        let testings = vec![
            TestingData {
                id: format!("1st-ILSZ-extra"),
                clipped_board: ClippedBoard::try_new(Board64::from_str("
                    #......###
                    #.......##
                    #.....####
                    #......###
                ").unwrap(), 4).unwrap(),
                generator: || {
                    let mut binder = PcPossibleBulkExecutorBinder::srs();

                    binder.pattern = Rc::from(Pattern::try_from(vec![
                        Factorial(ShapeCounter::try_from(vec![
                            L, T, O,
                        ]).unwrap()),
                        Permutation(ShapeCounter::one_of_each(), 4),
                    ]).unwrap());

                    binder
                },
                accepted: 5040,
                expected: vec![
                    (AllowMove::Softdrop, true, 5040),
                    (AllowMove::Harddrop, true, 1220),
                ],
            },
            TestingData {
                id: format!("1st-ILSZ-just"),
                clipped_board: ClippedBoard::try_new(Board64::from_str("
                    #......###
                    #.......##
                    #.....####
                    #......###
                ").unwrap(), 4).unwrap(),
                generator: || {
                    let mut binder = PcPossibleBulkExecutorBinder::srs();

                    binder.pattern = Rc::from(Pattern::try_from(vec![
                        Factorial(ShapeCounter::try_from(vec![
                            L, T, O,
                        ]).unwrap()),
                        Permutation(ShapeCounter::one_of_each(), 3),
                    ]).unwrap());

                    binder
                },
                accepted: 1260,
                expected: vec![
                    (AllowMove::Softdrop, true, 1188),
                    (AllowMove::Harddrop, true, 240),
                    (AllowMove::Softdrop, false, 523),
                    (AllowMove::Harddrop, false, 18),
                ],
            },
            TestingData {
                id: format!("1st-grace-system-extra"),
                clipped_board: ClippedBoard::try_new(Board64::from_str("
                    ######....
                    ######....
                    ######....
                    ######....
                ").unwrap(), 4).unwrap(),
                generator: || {
                    let mut binder = PcPossibleBulkExecutorBinder::srs();

                    binder.pattern = Rc::from(Pattern::try_from(vec![
                        One(T),
                        Permutation(ShapeCounter::one_of_each(), 4),
                    ]).unwrap());

                    binder
                },
                accepted: 840,
                expected: vec![
                    (AllowMove::Softdrop, true, 744),
                    (AllowMove::Harddrop, true, 634),
                ],
            },
            TestingData {
                id: format!("1st-grace-system-just"),
                clipped_board: ClippedBoard::try_new(Board64::from_str("
                    ######....
                    ######....
                    ######....
                    ######....
                ").unwrap(), 4).unwrap(),
                generator: || {
                    let mut binder = PcPossibleBulkExecutorBinder::srs();

                    binder.pattern = Rc::from(Pattern::try_from(vec![
                        One(T),
                        Permutation(ShapeCounter::one_of_each(), 3),
                    ]).unwrap());

                    binder
                },
                accepted: 210,
                expected: vec![
                    (AllowMove::Softdrop, true, 138),
                    (AllowMove::Harddrop, true, 100),
                    (AllowMove::Softdrop, false, 67),
                    (AllowMove::Harddrop, false, 37),
                ],
            },
            TestingData {
                id: format!("2nd-LSZT-extra"),
                clipped_board: ClippedBoard::try_new(Board64::from_str("
                    #.........
                    ##...#....
                    ######....
                    ######....
                ").unwrap(), 4).unwrap(),
                generator: || {
                    let mut binder = PcPossibleBulkExecutorBinder::srs();

                    binder.pattern = Rc::from(Pattern::try_from(vec![
                        One(T),
                        Permutation(ShapeCounter::one_of_each(), 6),
                    ]).unwrap());

                    binder
                },
                accepted: 5040,
                expected: vec![
                    (AllowMove::Softdrop, true, 4952),
                    (AllowMove::Harddrop, true, 3976),
                ],
            },
            TestingData {
                id: format!("2nd-LSZT-just"),
                clipped_board: ClippedBoard::try_new(Board64::from_str("
                    #.........
                    ##...#....
                    ######....
                    ######....
                ").unwrap(), 4).unwrap(),
                generator: || {
                    let mut binder = PcPossibleBulkExecutorBinder::srs();

                    binder.pattern = Rc::from(Pattern::try_from(vec![
                        One(T),
                        Permutation(ShapeCounter::one_of_each(), 5),
                    ]).unwrap());

                    binder
                },
                accepted: 2520,
                expected: vec![
                    (AllowMove::Softdrop, true, 1812),
                    (AllowMove::Harddrop, true, 1266),
                    (AllowMove::Softdrop, false, 992),
                    (AllowMove::Harddrop, false, 144),
                ],
            },
            TestingData {
                id: format!("empty-extra"),
                clipped_board: ClippedBoard::try_new(Board64::blank(), 4).unwrap(),
                generator: || {
                    let mut binder = PcPossibleBulkExecutorBinder::srs();

                    binder.pattern = Rc::from(Pattern::try_from(vec![
                        Fixed(BitShapes::try_from(vec![
                            S, L, Z, O, S, L, S, J, O, Z,
                        ]).unwrap()),
                        Wildcard, // I or O is not PC-able
                    ]).unwrap());

                    binder
                },
                accepted: 7,
                expected: vec![
                    (AllowMove::Softdrop, true, 5),
                    (AllowMove::Harddrop, true, 3),
                ],
            },
            TestingData {
                id: format!("empty-just"),
                clipped_board: ClippedBoard::try_new(Board64::blank(), 4).unwrap(),
                generator: || {
                    let mut binder = PcPossibleBulkExecutorBinder::srs();

                    binder.pattern = Rc::from(Pattern::try_from(vec![
                        Fixed(BitShapes::try_from(vec![
                            S, L, Z, O, S, L, S, J, O,
                        ]).unwrap()),
                        Wildcard, // I or O is not PC-able
                    ]).unwrap());

                    binder
                },
                accepted: 7,
                expected: vec![
                    (AllowMove::Softdrop, true, 3),
                    (AllowMove::Harddrop, true, 3),
                    (AllowMove::Softdrop, false, 2),
                    (AllowMove::Harddrop, false, 2),
                ],
            },
            TestingData {
                id: format!("4th-L-extra"),
                clipped_board: ClippedBoard::try_new(Board64::from_str("
                    ........##
                    .......###
                    ##....####
                    ##.....###
                ").unwrap(), 4).unwrap(),
                generator: || {
                    let mut binder = PcPossibleBulkExecutorBinder::srs();

                    binder.pattern = Rc::from(Pattern::try_from(vec![
                        One(L),
                        Permutation(ShapeCounter::one_of_each(), 6),
                    ]).unwrap());

                    binder
                },
                accepted: 5040,
                expected: vec![
                    (AllowMove::Softdrop, true, 5011),
                    (AllowMove::Harddrop, true, 3055),
                ],
            },
            TestingData {
                id: format!("4th-L-just"),
                clipped_board: ClippedBoard::try_new(Board64::from_str("
                    ........##
                    .......###
                    ##....####
                    ##.....###
                ").unwrap(), 4).unwrap(),
                generator: || {
                    let mut binder = PcPossibleBulkExecutorBinder::srs();

                    binder.pattern = Rc::from(Pattern::try_from(vec![
                        One(L),
                        Permutation(ShapeCounter::one_of_each(), 5),
                    ]).unwrap());

                    binder
                },
                accepted: 2520,
                expected: vec![
                    (AllowMove::Softdrop, true, 2002),
                    (AllowMove::Harddrop, true, 789),
                    (AllowMove::Softdrop, false, 1030),
                    (AllowMove::Harddrop, false, 177),
                ],
            },
            TestingData {
                id: format!("height-8-extra"),
                clipped_board: ClippedBoard::try_new(Board64::from_str("
                    ..........
                    ####....##
                    ###....###
                    ###....###
                    ###....###
                    ###....###
                    ###....###
                    ###....###
                    ##....####
                ").unwrap(), 8).unwrap(),
                generator: || {
                    let mut binder = PcPossibleBulkExecutorBinder::srs();

                    binder.pattern = Rc::from(Pattern::try_from(vec![
                        Permutation(ShapeCounter::one_of_each(), 2),
                        Factorial(ShapeCounter::one_of_each()),
                    ]).unwrap());

                    binder
                },
                accepted: 211680,
                expected: vec![
                    (AllowMove::Softdrop, true, 210152),
                    (AllowMove::Harddrop, true, 9458),
                ],
            },
        ];

        for testing in testings {
            println!("id: {}", testing.id);

            let mut binder = (testing.generator)();
            binder.clipped_board = testing.clipped_board;

            for (allow_move, allows_hold, succeed) in testing.expected {
                binder.allow_move = allow_move;
                binder.allows_hold = allows_hold;

                print!("  {}, hold {}: ", allow_move, allows_hold);

                {
                    let algorithm = PcPossibleAlgorithm::AllPcs;
                    binder.algorithm = algorithm;

                    let start = Instant::now();
                    let results = binder.try_execute().unwrap();
                    let end = start.elapsed();

                    print!("[{}] {} ms  ", algorithm, end.as_millis());

                    assert_eq!(results.count_succeed(), succeed);
                    assert_eq!(results.count_accepted(), testing.accepted);
                }
                // {
                //     let algorithm = PcPossibleAlgorithm::Simulation;
                //     binder.algorithm = algorithm;
                //
                //     let start = Instant::now();
                //     let results = binder.try_execute().unwrap();
                //     let end = start.elapsed();
                //
                //     print!("[{}] {} ms", algorithm, end.as_millis());
                //
                //     assert_eq!(results.count_succeed(), succeed);
                //     assert_eq!(results.count_accepted(), testing.accepted);
                // }

                println!();
            }
        }
    }
}
