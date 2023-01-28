use std::str::FromStr;

use criterion::{Criterion, criterion_group, criterion_main};

use bitris_commands::all_pcs;
use bitris_commands::prelude::*;

#[inline(always)]
fn all_pcs(data: &AllPcsFromShapeCounterBenchmarkData) {
    let move_rules = MoveRules::srs(data.allow_move);
    let clipped_board = ClippedBoard::try_new(data.board, data.height).unwrap();
    let executor = all_pcs::AllPcsFromCounterBulkExecutor::try_new(
        move_rules, clipped_board, &data.shape_counters,
    ).unwrap();
    let result = executor.execute();
    assert_eq!(result, data.expected);
}

#[derive(Debug)]
struct AllPcsFromShapeCounterBenchmarkData {
    id: String,
    board: Board64,
    height: u32,
    shape_counters: Vec<ShapeCounter>,
    allow_move: AllowMove,
    expected: u64,
}

fn bench_all_pcs_from_shape_counters(c: &mut Criterion) {
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
        let id = format!("all-pcs-from-shape-counters-{}", benchmark.id);
        c.bench_function(id.as_str(), |b| {
            b.iter(|| all_pcs(benchmark));
        });
    });
}

criterion_group!(benches, bench_all_pcs_from_shape_counters);
criterion_main!(benches);
