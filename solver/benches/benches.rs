mod bitboard;
mod engine;

use criterion::*;
use bitboard::*;
use engine::*;

criterion_main!(bench_weak_solver);
