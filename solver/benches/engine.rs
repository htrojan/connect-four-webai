use criterion::*;
use c4solver::board::BitBoard;
use std::hint::black_box;
use c4solver::engine::*;

const EARLY_01: &str =
           "nnnnnnn
            nnnnnnn
            nnnnnnn
            nnnnnnn
            nnnnnnn
            nnncnnn";
const EARLY_02: &str =
            "nnnnnnn
            nnnnnnn
            nnnnnnn
            nnncnnn
            nnnppnn
            nnnccnn";
const MID_01: &str =
           "nnnpnnn
            nnnccnn
            nnnppnn
            nnnccnn
            nnnppnn
            cnnccnn";
const MID_02: &str =
           "nnnpnnn
            nnncpnn
            nnnpcnn
            nnncpnn
            npnpcnn
            ncncpcn";

pub fn bench_weak_early(crit: &mut Criterion) {
    let board_01 = black_box(BitBoard::from_string(EARLY_01).unwrap());
    let board_02 = black_box(BitBoard::from_string(EARLY_02).unwrap());
    let mut num_nodes: u64 = 0;

    crit.bench_function("weak_early_01", |b| b.iter(|| solve_weak(board_01, 11, i32::MIN+2,
    i32::MAX-2, &mut num_nodes)));
    crit.bench_function("weak_early_02", |b| b.iter(|| solve_weak(board_02, 11, i32::MIN+2,
                                                                  i32::MAX-2, &mut num_nodes)));
}
pub fn bench_weak_mid(crit: &mut Criterion) {
    let board_01 = black_box(BitBoard::from_string(MID_01).unwrap());
    let board_02 = black_box(BitBoard::from_string(MID_02).unwrap());
    let mut num_nodes: u64 = 0;

    crit.bench_function("weak_mid_01", |b| b.iter(|| solve_weak(board_01, 11, i32::MIN+2,
                                                                  i32::MAX-2, &mut num_nodes)));
    crit.bench_function("weak_mid_02", |b| b.iter(|| solve_weak(board_02, 11, i32::MIN+2,
                                                                  i32::MAX-2, &mut num_nodes)));
}

criterion_group!(bench_weak_solver, bench_weak_early, bench_weak_mid);
