use criterion::*;
use connect_four::logic::{ABSolver, GameBoard, FieldType, BOARD_HEIGHT, BOARD_WIDTH};

const p: Option<FieldType> = Some(FieldType::Player);
const c: Option<FieldType> = Some(FieldType::Opponent);
const n: Option<FieldType> = None::<FieldType>;

const fields_endgame01: [[Option<FieldType>; BOARD_HEIGHT]; BOARD_WIDTH]
= [
    [c,p,p,c,n,n,],
    [c,p,p,p,n,n,],
    [p,c,c,p,n,n,],
    [p,c,p,n,n,n,],
    [c,p,c,n,n,n,],
    [c,p,c,n,n,n,],
    [p,c,p,c,p,n,],
];

pub fn bench_evaluate(crit: &mut Criterion) {
    let board_endgame01 = GameBoard::new(black_box(fields_endgame01));
    crit.bench_function("evaluate_endgame01", |b| b.iter(|| board_endgame01.evaluate()));
    // assert!(bits_1.is_winning_move(position));
    // assert!(bits_2.is_winning_move(position));
}

criterion_group!(bench_gameboard, bench_evaluate);