#![feature(test)]

use criterion::*;
use connect_four::board::BitBoard;
use std::hint::black_box;

pub fn bench_winning_move(crit: &mut Criterion) {
    let board_1 =
        "nnnpnnn
        nnnnnnn
        nnnnnnn
        npnpnnn
        npnnpnn
        nncccpn";

    let bits_1 = BitBoard::from_string(board_1).unwrap();

    let player_move =
        "nnnnnnn
            nnnnnnn
            nnpnnnn
            nnnnnnn
            nnnnnnn
            nnnnnnn";
    let position = BitBoard::from_string(player_move).unwrap().player;

    crit.bench_function("winning_move_01", |b| b.iter( || bits_1.is_winning_move(black_box(position))));
}

pub fn bench_winning_board(crit: &mut Criterion) {
    let board_1 =
        "nnnpnnn
        nnnnpnn
        nnnnpnn
        npnppnn
        npnnpnn
        nncccpn";

    let bits_1 = BitBoard::from_string(board_1).unwrap();
    let bits_1 = black_box(bits_1);

    crit.bench_function("winning_board_01", |b| b.iter( || bits_1.has_won()));

}

pub fn bench_evaluation(crit: &mut Criterion) {
    let board_1 =
        "nnnpnnn
        nnnnpnn
        nnnnpnn
        npnppnn
        npnnpnn
        nncccpn";

    let bits_1 = BitBoard::from_string(board_1).unwrap();
    let bits_1 = black_box(bits_1);

    crit.bench_function("evaluation_01", |b| b.iter(|| bits_1.heuristic()));
}

pub fn bench_heuristics2(crit: &mut Criterion) {
    let board_1 =
        "nnnpnnn
        nnnnpnn
        nnnnpnn
        npnppnn
        npnnpnn
        nncccpn";

    let bits_1 = BitBoard::from_string(board_1).unwrap();
    let bits_1 = black_box(bits_1);

    crit.bench_function("heuristics2_01", |b| b.iter(|| bits_1.heuristic_2()));

}

criterion_group!(bench_bitboard, bench_heuristics2, bench_evaluation, bench_winning_move, bench_winning_board);