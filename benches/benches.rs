mod bitboard;
mod gameboard;

use criterion::*;
use bitboard::*;
use gameboard::*;

criterion_main!(bench_bitboard, bench_gameboard);
