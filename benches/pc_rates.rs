use std::str::FromStr;

use criterion::{Criterion, criterion_group, criterion_main};

use bitris_commands::prelude::*;
use bitris_commands::commands;

#[inline(always)]
fn pc_success_rates(data: &MovesBenchmarkData) {
    let move_rules = MoveRules::srs(Drop::Softdrop);
    let result = commands::pc_success_rates(&move_rules, data.board, data.height, &data.patterns, true);
    assert_eq!(result.count_success(), data.expected);
}

#[derive(Debug)]
struct MovesBenchmarkData {
    id: String,
    board: Board64,
    patterns: Pattern,
    expected: usize,
    height: u32,
}

fn bench_pc_rates(c: &mut Criterion) {
    use Shape::*;
    use PatternElement::*;

    let benchmarks = vec![
        MovesBenchmarkData {
            id: format!("pco-last3"),
            board: Board64::from_str(
                "
                ####....##
                ####...###
                ####..####
                ####...###
            ").unwrap(),
            height: 4,
            patterns: Pattern::new(vec![
                Permutation(ShapeCounter::one_of_each(), 4),
            ]),
            expected: 514,
        },
        MovesBenchmarkData {
            id: format!("pco-last4"),
            board: Board64::from_str(
                "
                ##.....###
                ###....###
                ####...###
                ###....###
            ").unwrap(),
            height: 4,
            patterns: Pattern::new(vec![
                Permutation(ShapeCounter::one_of_each(), 5),
            ]),
            expected: 1672,
        },
        MovesBenchmarkData {
            id: format!("pco-last6"),
            board: Board64::from_str(
                "
                #.......##
                #......###
                #.....####
                #......###
            ").unwrap(),
            height: 4,
            patterns: Pattern::new(vec![
                Permutation(ShapeCounter::one_of_each(), 7),
            ]),
            expected: 5028,
        },
        MovesBenchmarkData {
            id: format!("1st-cycle-partial"),
            board: Board64::blank(),
            height: 4,
            patterns: Pattern::new(vec![
                Fixed(BitShapes::try_from(vec![
                    T, I, O, S, L, J, Z, T, I, O,
                ]).unwrap()),
                Wildcard,
            ]),
            expected: 7,
        },
        MovesBenchmarkData {
            id: format!("grace-system"),
            board: Board64::from_str(
                "
                ######....
                ######....
                ######....
                ######....
            ").unwrap(),
            height: 4,
            patterns: Pattern::new(vec![
                One(Shape::T),
                Permutation(ShapeCounter::one_of_each(), 4),
            ]),
            expected: 744,
        },
    ];

    benchmarks.iter().for_each(|benchmark| {
        let id = format!("pc-rates-{}", benchmark.id);
        c.bench_function(id.as_str(), |b| {
            b.iter(|| pc_success_rates(benchmark));
        });
    });
}

criterion_group!(benches, bench_pc_rates);
criterion_main!(benches);
