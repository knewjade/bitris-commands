use std::rc::Rc;
use std::str::FromStr;

use criterion::{Criterion, criterion_group, criterion_main};

use bitris_commands::pc_possible;
use bitris_commands::prelude::*;

#[inline(always)]
fn pc_possible(data: &PcPossibleBenchmarkData) {
    let move_rules = MoveRules::srs(MoveType::Softdrop);
    let clipped_board = ClippedBoard::try_new(data.board, data.height).unwrap();
    let executor = pc_possible::PcPossibleExecutor::try_new(
        &move_rules, clipped_board, &data.patterns, true,
    ).unwrap();
    let result = executor.execute();
    assert_eq!(result.count_succeed(), data.expected);
}

#[derive(Debug)]
struct PcPossibleBenchmarkData {
    id: String,
    board: Board64,
    height: u32,
    patterns: Rc<Pattern>,
    expected: usize,
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
            patterns: Rc::from(Pattern::new(vec![
                Permutation(ShapeCounter::one_of_each(), 4),
            ])),
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
            patterns: Rc::from(Pattern::new(vec![
                Permutation(ShapeCounter::one_of_each(), 5),
            ])),
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
            patterns: Rc::from(Pattern::new(vec![
                Factorial(ShapeCounter::one_of_each()),
            ])),
            expected: 5028,
        },
        PcPossibleBenchmarkData {
            id: format!("1st-cycle-partial"),
            board: Board64::blank(),
            height: 4,
            patterns: Rc::from(Pattern::new(vec![
                Fixed(BitShapes::try_from(vec![
                    T, I, O, S, L, J, Z, T, I, O,
                ]).unwrap()),
                Wildcard,
            ])),
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
            patterns: Rc::from(Pattern::new(vec![
                One(T),
                Permutation(ShapeCounter::one_of_each(), 4),
            ])),
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
            patterns: Rc::from(Pattern::new(vec![
                Factorial(ShapeCounter::one_of_each()),
            ])),
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
