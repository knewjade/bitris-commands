use std::rc::Rc;
use std::str::FromStr;

use criterion::{Criterion, criterion_group, criterion_main};

use bitris_commands::pc_possible;
use bitris_commands::prelude::*;

#[inline(always)]
fn pc_possible(data: &PcPossibleBenchmarkData) {
    let move_rules = MoveRules::srs(data.allow_move);
    let clipped_board = ClippedBoard::try_new(data.board, data.height).unwrap();
    let executor = pc_possible::PcPossibleBulkExecutor::try_new(
        &move_rules, clipped_board, &data.pattern, data.allows_hold,
    ).unwrap();
    let result = executor.execute();
    assert_eq!(result.count_succeed(), data.expected);
}

#[derive(Debug)]
struct PcPossibleBenchmarkData {
    id: String,
    board: Board64,
    height: u32,
    pattern: Rc<Pattern>,
    allow_move: AllowMove,
    allows_hold: bool,
    expected: u64,
}

fn bench_pc_possibles(c: &mut Criterion) {
    use Shape::*;
    use PatternElement::*;

    let benchmarks = vec![
        PcPossibleBenchmarkData {
            id: format!("pco-last3"),
            board: Board64::from_str(
                "
                ####....##
                ####...###
                ####..####
                ####...###
            ").unwrap(),
            height: 4,
            pattern: Rc::from(Pattern::try_from(vec![
                Permutation(ShapeCounter::one_of_each(), 4),
            ]).unwrap()),
            allow_move: AllowMove::Softdrop,
            allows_hold: true,
            expected: 514,
        },
        PcPossibleBenchmarkData {
            id: format!("pco-last4"),
            board: Board64::from_str(
                "
                ##.....###
                ###....###
                ####...###
                ###....###
            ").unwrap(),
            height: 4,
            pattern: Rc::from(Pattern::try_from(vec![
                Permutation(ShapeCounter::one_of_each(), 5),
            ]).unwrap()),
            allow_move: AllowMove::Softdrop,
            allows_hold: true,
            expected: 1672,
        },
        PcPossibleBenchmarkData {
            id: format!("pco-last6"),
            board: Board64::from_str(
                "
                #.......##
                #......###
                #.....####
                #......###
            ").unwrap(),
            height: 4,
            pattern: Rc::from(Pattern::try_from(vec![
                Factorial(ShapeCounter::one_of_each()),
            ]).unwrap()),
            allow_move: AllowMove::Softdrop,
            allows_hold: true,
            expected: 5028,
        },
        PcPossibleBenchmarkData {
            id: format!("1st-cycle-partial"),
            board: Board64::blank(),
            height: 4,
            pattern: Rc::from(Pattern::try_from(vec![
                Fixed(BitShapes::try_from(vec![
                    T, I, O, S, L, J, Z, T, I, O,
                ]).unwrap()),
                Wildcard,
            ]).unwrap()),
            allow_move: AllowMove::Softdrop,
            allows_hold: true,
            expected: 7,
        },
        PcPossibleBenchmarkData {
            id: format!("grace-system"),
            board: Board64::from_str(
                "
                ######....
                ######....
                ######....
                ######....
            ").unwrap(),
            height: 4,
            pattern: Rc::from(Pattern::try_from(vec![
                One(T),
                Permutation(ShapeCounter::one_of_each(), 4),
            ]).unwrap()),
            allow_move: AllowMove::Softdrop,
            allows_hold: true,
            expected: 744,
        },
        PcPossibleBenchmarkData {
            id: format!("2nd-pattern"),
            board: Board64::from_str(
                "
                ..........
                ....####..
                ....######
                ....######
            ").unwrap(),
            height: 4,
            pattern: Rc::from(Pattern::try_from(vec![
                Factorial(ShapeCounter::one_of_each()),
            ]).unwrap()),
            allow_move: AllowMove::Softdrop,
            allows_hold: true,
            expected: 4788,
        },
    ];

    benchmarks.iter().for_each(|benchmark| {
        let id = format!("pc-rates-{}", benchmark.id);
        c.bench_function(id.as_str(), |b| {
            b.iter(|| pc_possible(benchmark));
        });
    });
}

criterion_group!(benches, bench_pc_possibles);
criterion_main!(benches);
