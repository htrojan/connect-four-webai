use criterion::*;
use connect_four::BitBoard::BitBoard;

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

    crit.bench_function("winning_move_1", |b| b.iter( || bits_1.is_winning_move(black_box(position))));
    // assert!(bits_1.is_winning_move(position));
    // assert!(bits_2.is_winning_move(position));
}

criterion_group!(bench_bitboard, bench_winning_move);