use bitris::prelude::*;

use crate::ShapeMatcher;

pub(crate) fn visit_all_dyn<'a, T: ShapeMatcher>(
    initial_board: Board64,
    refs: &Vec<&'a PlacedPieceBlocks>,
    validator: impl Fn(&Board64, BlPlacement) -> SearchResult,
    make_matcher: impl FnOnce() -> T,
) -> Option<PlacedPieceBlocksFlow<'a>> {
    if refs.is_empty() {
        return None;
    }

    assert!(refs.len() < 64, "refs length supports up to 64.");

    struct Builder<'a, 'b> {
        refs: &'a Vec<&'b PlacedPieceBlocks>,
        results: Vec<&'b PlacedPieceBlocks>,
    }

    impl Builder<'_, '_> {
        fn build(
            &mut self,
            board: Board64,
            remaining: u64,
            matcher: impl ShapeMatcher,
            validator: &impl Fn(&Board64, BlPlacement) -> SearchResult,
        ) -> bool {
            let mut candidates = remaining;
            while 0 < candidates {
                let next_candidates = candidates & (candidates - 1);
                let bit = candidates - next_candidates;
                let next_remaining = remaining - bit;

                candidates = next_candidates;

                let placed_piece_blocks = self.refs[bit.trailing_zeros() as usize];
                let shape = placed_piece_blocks.placed_piece.piece.shape;
                let (matched, next_matcher) = matcher.match_shape(shape);
                if !matched {
                    continue;
                }

                if let Some(placement) = placed_piece_blocks.place_according_to(board) {
                    if validator(&board, placement) == SearchResult::Pruned {
                        continue;
                    }

                    self.results.push(placed_piece_blocks);

                    if next_remaining == 0 {
                        return true;
                    }

                    let mut next_board = board;
                    next_board.set_all(&placed_piece_blocks.locations);

                    if self.build(next_board, next_remaining, next_matcher, validator) {
                        return true;
                    }
                    self.results.pop();
                }
            }

            false
        }
    }

    let len = refs.len();
    let mut builder = Builder {
        refs,
        results: Vec::with_capacity(len),
    };

    if builder.build(initial_board, (1u64 << len) - 1, make_matcher(), &validator) {
        Some(PlacedPieceBlocksFlow::new(initial_board, builder.results))
    } else {
        None
    }
}
