use std::str::FromStr;

use criterion::{black_box, Criterion, criterion_group, criterion_main};

use bitris_commands::all_pcs;
use bitris_commands::prelude::*;

#[inline(always)]
fn all_pcs_from_counters(data: &AllPcsFromShapeCounterBenchmarkData) {
    let move_rules = MoveRules::srs(data.allow_move);
    let clipped_board = ClippedBoard::try_new(data.board, data.height).unwrap();
    let executor = all_pcs::AllPcsFromCountersExecutor::try_new(
        &move_rules, clipped_board, &data.shape_counters,
    ).unwrap();
    let result = executor.execute();
    assert_eq!(result.len(), data.expected);
}

#[inline(always)]
fn all_pcs_from_pattern(data: &AllPcsFromPatternBenchmarkData) {
    let move_rules = MoveRules::srs(data.allow_move);
    let clipped_board = ClippedBoard::try_new(data.board, data.height).unwrap();
    let executor = all_pcs::AllPcsFromPatternExecutor::try_new(
        &move_rules, clipped_board, &data.pattern, data.allows_hold,
    ).unwrap();
    let result = executor.execute();
    assert_eq!(result.len(), data.expected);
}

#[derive(Debug)]
struct AllPcsFromShapeCounterBenchmarkData {
    id: String,
    board: Board64,
    height: u32,
    shape_counters: Vec<ShapeCounter>,
    allow_move: AllowMove,
    expected: usize,
}

fn bench_all_pcs_from_counters(c: &mut Criterion) {
    let benchmarks = vec![
        AllPcsFromShapeCounterBenchmarkData {
            id: format!("pco-wildcard3"),
            board: Board64::from_str(
                "
                ####....##
                ####...###
                ####..####
                ####...###
            ").unwrap(),
            height: 4,
            shape_counters: vec![
                ShapeCounter::one_of_each() * 3,
            ],
            allow_move: AllowMove::Softdrop,
            expected: 28,
        },
        AllPcsFromShapeCounterBenchmarkData {
            id: format!("wildcard3"),
            board: Board64::from_str(
                "
                ...#######
                ...#######
                ...#######
                ...#######
            ").unwrap(),
            height: 4,
            shape_counters: vec![
                ShapeCounter::one_of_each() * 3,
            ],
            allow_move: AllowMove::Softdrop,
            expected: 79,
        },
        AllPcsFromShapeCounterBenchmarkData {
            id: format!("wildcard6"),
            board: Board64::from_str(
                "
                ......####
                ......####
                ......####
                ......####
            ").unwrap(),
            height: 4,
            shape_counters: vec![
                ShapeCounter::one_of_each() * 6,
            ],
            allow_move: AllowMove::Softdrop,
            expected: 16944,
        },
    ];

    benchmarks.iter().for_each(|benchmark| {
        let id = format!("all-pcs-from-counters-{}", benchmark.id);
        c.bench_function(id.as_str(), |b| {
            b.iter(|| all_pcs_from_counters(black_box(benchmark)));
        });
    });
}

#[derive(Debug)]
struct AllPcsFromPatternBenchmarkData {
    id: String,
    board: Board64,
    height: u32,
    pattern: Pattern,
    allow_move: AllowMove,
    allows_hold: bool,
    expected: usize,
}

fn bench_all_pcs_from_pattern(c: &mut Criterion) {
    use Shape::*;
    let benchmarks = vec![
        AllPcsFromPatternBenchmarkData {
            id: format!("pco-(i,*p4)"),
            board: Board64::from_str(
                "
                ###.....##
                ###....###
                ###...####
                ###....###
            ").unwrap(),
            height: 4,
            pattern: vec![
                PatternElement::One(I),
                PatternElement::Permutation(ShapeCounter::one_of_each(), 4),
            ].try_into().unwrap(),
            allow_move: AllowMove::Softdrop,
            allows_hold: true,
            expected: 63,
        },
        AllPcsFromPatternBenchmarkData {
            id: format!("pco-([TOJ]!,*p4)"),
            board: Board64::from_str(
                "
                ####......
                ###.......
                #####.....
                ####......
            ").unwrap(),
            height: 4,
            pattern: vec![
                PatternElement::Factorial(vec![T, O, J].try_into().unwrap()),
                PatternElement::Permutation(ShapeCounter::one_of_each(), 4),
            ].try_into().unwrap(),
            allow_move: AllowMove::Softdrop,
            allows_hold: true,
            expected: 605,
        },
    ];

    benchmarks.iter().for_each(|benchmark| {
        let id = format!("all-pcs-from-pattern-{}", benchmark.id);
        c.bench_function(id.as_str(), |b| {
            b.iter(|| all_pcs_from_pattern(black_box(benchmark)));
        });
    });
}

criterion_group!(benches, bench_all_pcs_from_counters, bench_all_pcs_from_pattern);
criterion_main!(benches);
