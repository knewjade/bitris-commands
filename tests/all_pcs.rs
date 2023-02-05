#[cfg(test)]
mod tests {
    use std::rc::Rc;
    use std::str::FromStr;
    use std::time::Instant;

    use bitris_commands::all_pcs::{AllPcsFromCountersExecutorBinder, AllPcsFromOrderExecutorBinder, AllPcsFromPatternExecutorBinder};
    use bitris_commands::prelude::*;

    #[test]
    fn from_order() {
        use Shape::*;

        struct TestingData {
            id: String,
            clipped_board: ClippedBoard,
            // (allow move, allows hold, result)
            expected: Vec<(AllowMove, bool, usize)>,
            generator: fn() -> AllPcsFromOrderExecutorBinder<SrsKickTable>,
        }

        let testings = vec![
            TestingData {
                id: format!("4-pieces-just"),
                clipped_board: ClippedBoard::try_new(Board64::from_str("
                    ######....
                    ######....
                    ######....
                    ######....
                ").unwrap(), 4).unwrap(),
                generator: || {
                    let mut binder = AllPcsFromOrderExecutorBinder::srs();

                    binder.shape_order = Rc::new(vec![S, I, T, J].into());

                    binder
                },
                expected: vec![
                    (AllowMove::Softdrop, true, 4),
                    (AllowMove::Harddrop, true, 3),
                    (AllowMove::Softdrop, false, 1),
                    (AllowMove::Harddrop, false, 1),
                ],
            },
            TestingData {
                id: format!("4-pieces-no-solutions"),
                clipped_board: ClippedBoard::try_new(Board64::from_str("
                    ######....
                    ######....
                    ######....
                    ######....
                ").unwrap(), 4).unwrap(),
                generator: || {
                    let mut binder = AllPcsFromOrderExecutorBinder::srs();

                    binder.shape_order = Rc::new(vec![S, T, J, O].into());

                    binder
                },
                expected: vec![
                    (AllowMove::Softdrop, true, 0),
                    (AllowMove::Harddrop, true, 0),
                    (AllowMove::Softdrop, false, 0),
                    (AllowMove::Harddrop, false, 0),
                ],
            },
            TestingData {
                id: format!("empty-extra"),
                clipped_board: ClippedBoard::try_new(Board64::blank(), 4).unwrap(),
                generator: || {
                    let mut binder = AllPcsFromOrderExecutorBinder::srs();

                    binder.shape_order = Rc::new(vec![T, L, J, I, O, S, Z, L, J, T, O].into());

                    binder
                },
                expected: vec![
                    (AllowMove::Softdrop, true, 8272),
                ],
            },
        ];

        for testing in testings {
            println!("id: {}", testing.id);

            let mut binder = (testing.generator)();
            binder.clipped_board = testing.clipped_board;

            for (allow_move, allows_hold, count) in testing.expected {
                binder.allow_move = allow_move;
                binder.allows_hold = allows_hold;

                let start = Instant::now();
                let solutions = binder.try_execute().unwrap();
                let end = start.elapsed();
                println!("  {}: {} μs", allow_move, end.as_micros());

                assert_eq!(solutions.len(), count);
            }
        }
    }

    #[test]
    fn from_counters() {
        use Shape::*;

        struct TestingData {
            id: String,
            clipped_board: ClippedBoard,
            // (allow move, result)
            expected: Vec<(AllowMove, usize)>,
            generator: fn() -> AllPcsFromCountersExecutorBinder<SrsKickTable>,
        }

        let testings = vec![
            TestingData {
                id: format!("2nd"),
                clipped_board: ClippedBoard::try_new(Board64::from_str("
                    ..........
                    ...#......
                    #######...
                    ########..
                ").unwrap(), 4).unwrap(),
                generator: || {
                    let mut binder = AllPcsFromCountersExecutorBinder::srs();

                    binder.shape_counters = Rc::new(vec![
                        ShapeCounter::one_of_each(),
                    ]);

                    binder
                },
                expected: vec![
                    (AllowMove::Softdrop, 13),
                    (AllowMove::Harddrop, 11),
                ],
            },
            TestingData {
                id: format!("3rd"),
                clipped_board: ClippedBoard::try_new(Board64::from_str("
                    ###....###
                    ###.....##
                    ###.....##
                    ###......#
                ").unwrap(), 4).unwrap(),
                generator: || {
                    let mut binder = AllPcsFromCountersExecutorBinder::srs();

                    binder.shape_counters = Rc::new(vec![
                        ShapeCounter::one_of_each(),
                    ]);

                    binder
                },
                expected: vec![
                    (AllowMove::Softdrop, 70),
                    (AllowMove::Harddrop, 0),
                ],
            },
            TestingData {
                id: format!("same-shapes"),
                clipped_board: ClippedBoard::try_new(Board64::from_str("
                    ####......
                    ####......
                    ####......
                    ####......
                ").unwrap(), 4).unwrap(),
                generator: || {
                    let mut binder = AllPcsFromCountersExecutorBinder::srs();

                    binder.shape_counters = Rc::new(vec![
                        ShapeCounter::single_shape(T, 6),
                        ShapeCounter::single_shape(L, 6),
                        ShapeCounter::single_shape(O, 6),
                    ]);

                    binder
                },
                expected: vec![
                    (AllowMove::Softdrop, 2 + 6 + 1),
                    (AllowMove::Harddrop, 2 + 6 + 1),
                ],
            },
        ];

        for testing in testings {
            println!("id: {}", testing.id);

            let mut binder = (testing.generator)();
            binder.clipped_board = testing.clipped_board;

            for (allow_move, count) in testing.expected {
                binder.allow_move = allow_move;

                let start = Instant::now();
                let solutions = binder.try_execute().unwrap();
                let end = start.elapsed();
                println!("  {}: {} μs", allow_move, end.as_micros());

                assert_eq!(solutions.len(), count);
            }
        }
    }

    #[test]
    fn from_pattern() {
        use PatternElement::*;
        use Shape::*;

        struct TestingData {
            id: String,
            clipped_board: ClippedBoard,
            // (allow move, allows hold, result)
            expected: Vec<(AllowMove, bool, usize)>,
            generator: fn() -> AllPcsFromPatternExecutorBinder<SrsKickTable>,
        }

        let testings = vec![
            TestingData {
                id: format!("2nd-extra"),
                clipped_board: ClippedBoard::try_new(Board64::from_str("
                    ..........
                    #..#......
                    #####...##
                    #####...##
                ").unwrap(), 4).unwrap(),
                generator: || {
                    let mut binder = AllPcsFromPatternExecutorBinder::srs();

                    binder.pattern = Rc::from(Pattern::try_from(vec![
                        One(T),
                        Wildcard,
                        Fixed(vec![I, J, O, Z, S].try_into().unwrap()),
                    ]).unwrap());

                    binder
                },
                expected: vec![
                    (AllowMove::Softdrop, true, 7),
                    (AllowMove::Harddrop, true, 5),
                ],
            },
        ];

        for testing in testings {
            println!("id: {}", testing.id);

            let mut binder = (testing.generator)();
            binder.clipped_board = testing.clipped_board;

            for (allow_move, allows_hold, count) in testing.expected {
                binder.allow_move = allow_move;
                binder.allows_hold = allows_hold;

                let start = Instant::now();
                let solutions = binder.try_execute().unwrap();
                let end = start.elapsed();
                println!("  {}: {} μs", allow_move, end.as_micros());

                assert_eq!(solutions.len(), count);
            }
        }
    }
}
