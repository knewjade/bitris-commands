use std::rc::Rc;
use std::str::FromStr;

use criterion::{Criterion, criterion_group, criterion_main};

use bitris_commands::{all_pcs, pc_possible};
use bitris_commands::prelude::*;

#[inline(always)]
fn all_pcs(data: &AllPcsBenchmarkData) {
    let move_rules = MoveRules::srs(data.allow_move);
    let clipped_board = ClippedBoard::try_new(data.board, data.height).unwrap();
    let executor = all_pcs::AllPcsBulkExecutor::try_new(
        &move_rules, clipped_board, &data.patterns, true,
    ).unwrap();
    let result = executor.execute();
    assert_eq!(result, data.expected);
}

#[derive(Debug)]
struct AllPcsBenchmarkData {
    id: String,
    board: Board64,
    height: u32,
    patterns: Rc<Pattern>,
    allow_move: AllowMove,
    allows_hold: bool,
    expected: u64,
}

fn bench_all_pcs(c: &mut Criterion) {
    use Shape::*;
    use PatternElement::*;

    let benchmarks = vec![
        AllPcsBenchmarkData {
            id: format!("pco-last3"), // TODO
            board: Board64::from_str(
                "
                ####....##
                ####...###
                ####..####
                ####...###
            ").unwrap(),
            height: 4,
            patterns: Rc::from(Pattern::try_from(vec![
                Wildcard,
            ].repeat(3)).unwrap()),
            allow_move: AllowMove::Softdrop,
            allows_hold: true,
            expected: 28,
        },
        AllPcsBenchmarkData {
            id: format!("pco-last4"), // TODO
            board: Board64::from_str(
                "
                ...#######
                ...#######
                ...#######
                ...#######
            ").unwrap(),
            height: 4,
            patterns: Rc::from(Pattern::try_from(vec![
                Wildcard,
            ].repeat(3)).unwrap()),
            allow_move: AllowMove::Softdrop,
            allows_hold: true,
            expected: 79,
        },
        AllPcsBenchmarkData {
            id: format!("pco-last5"), // TODO
            board: Board64::from_str(
                "
                ......####
                ......####
                ......####
                ......####
            ").unwrap(),
            height: 4,
            patterns: Rc::from(Pattern::try_from(vec![
                Wildcard,
            ].repeat(6)).unwrap()),
            allow_move: AllowMove::Softdrop,
            allows_hold: true,
            expected: 16944,
        },
    ];

    benchmarks.iter().for_each(|benchmark| {
        let id = format!("all-pcs-{}", benchmark.id);
        c.bench_function(id.as_str(), |b| {
            b.iter(|| all_pcs(benchmark));
        });
    });
}

criterion_group!(benches, bench_all_pcs);
criterion_main!(benches);
