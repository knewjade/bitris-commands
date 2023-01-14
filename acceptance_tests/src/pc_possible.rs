#[cfg(test)]
mod tests {
    use std::rc::Rc;
    use std::str::FromStr;

    use bitris_commands::pc_possible::*;
    use bitris_commands::prelude::*;

    #[test]
    fn srs() {
        let mut binder = PcPossibleExecutorBinder::srs(MoveType::Softdrop);

        let board = Board64::from_str("
            #......###
            #.......##
            #.....####
            #......###
        ").unwrap();
        let height = 4;
        binder.clipped_board = ClippedBoard::try_new(board, height).unwrap();

        {
            binder.pattern = Rc::from(Pattern::new(vec![
                PatternElement::Factorial(ShapeCounter::try_from(vec![
                    Shape::L, Shape::T, Shape::O,
                ]).unwrap()),
                PatternElement::Permutation(ShapeCounter::one_of_each(), 4),
            ]));
            binder.allows_hold = true;
            let results = binder.try_bind().unwrap().execute();
            assert_eq!(results.count_succeed(), 5040);
            assert_eq!(results.count_accepted(), 5040);
        }

        {
            binder.pattern = Rc::from(Pattern::new(vec![
                PatternElement::Factorial(ShapeCounter::try_from(vec![
                    Shape::L, Shape::T, Shape::O,
                ]).unwrap()),
                PatternElement::Permutation(ShapeCounter::one_of_each(), 3),
            ]));
            binder.allows_hold = false;
            let results = binder.try_bind().unwrap().execute();
            assert_eq!(results.count_succeed(), 523);
            assert_eq!(results.count_accepted(), 1260);
        }
    }
}
