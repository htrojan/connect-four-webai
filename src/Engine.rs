use crate::BitBoard::BitBoard;

const SEARCH_ORDER: [u64; 7] = [3,2,4,1,5,0,6];


/// Solves the board using a weak solver BitBoard::is_winning_board()
/// return score, best_move
pub fn solve_weak(start: BitBoard, depth: u8, mut alpha: i32, mut beta: i32, player: bool) -> (i32, u64) {
    if start.has_lost() {
        return (-1 - depth as i32, 0);
    }

    // No conclusion found --> draw
    if depth == 0 {
        return (0, 0);
    }

    let mut max_score = i32::MIN;
    let mut best_move: u64 = 0;

    let possible_moves = start.all_possible_moves();

    for i in &SEARCH_ORDER {
        let i = *i;
        let to_play = BitBoard::move_in_row(possible_moves, i);

        // No valid move
        if to_play == 0 {
            continue;
        }

        let new_board = start.play_field(to_play);
        let (score, _) =  solve_weak(new_board, depth-1, -beta, -alpha, !player);
        let score = -score;

        if score > max_score {
            max_score = score;
            best_move = to_play;
        }
        alpha = i32::max(alpha, score);

        if alpha >= beta {
            // Cutoff!
            break;
        }
    }

    (max_score, best_move)
}

#[cfg(test)]
mod tests {
    use crate::BitBoard::BitBoard;
    use crate::Engine::solve_weak;

    #[test]
    fn test_solve_easy() {
        let board_easy =
            "nnnnnnn
            nnnnnnn
            nnnnnnn
            nnnnnpn
            nnncnpn
            nnnccpn";
        let bits = BitBoard::from_string(board_easy).unwrap();

        let (score, best_move) = solve_weak(bits, 3, i32::MIN+2, i32::MAX-2, true);

        let best_move_easy =
            "nnnnnnn
            nnnnnnn
            nnnnnpn
            nnnnnnn
            nnnnnnn
            nnnnnnn";
        let best_move_easy = BitBoard::from_string(best_move_easy).unwrap().player;

        println!("{}", score);
        assert_eq!(best_move_easy, best_move)
    }

    #[test]
    fn test_solve_advanced() {
        let board_easy =
            "nnnnnnn
            nnnnnnn
            nnnnnnn
            nnnpnnn
            nnncnpn
            nnnccpn";
        let bits = BitBoard::from_string(board_easy).unwrap();

        let (score, best_move) = solve_weak(bits, 15, i32::MIN+2, i32::MAX-2, true);

        let best_move_easy =
            "nnnnnnn
            nnnnnnn
            nnnnnpn
            nnnnnnn
            nnnnnnn
            nnnnnnn";
        let best_move_easy = BitBoard::from_string(best_move_easy).unwrap().player;

        println!("Score: {}", score);
        println!("Move: {}", best_move);
    }

}