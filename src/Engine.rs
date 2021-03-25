use crate::BitBoard::BitBoard;
use wasm_bindgen::prelude::*;
use crate::logic::BestMove;

const SEARCH_ORDER: [u64; 7] = [3,2,4,1,5,0,6];

#[wasm_bindgen]
#[derive(Eq, PartialEq, Debug)]
pub enum SolverType {
    Strong, Weak
}

#[wasm_bindgen]
#[derive(Default, Eq, PartialEq, Debug)]
pub struct SolveResult {
    pub score: i32,
    pub mov: u64,
    pub nodes_searched: u64,
    // Win was found in x rounds
    pub end_in: i32,
}

#[wasm_bindgen]
impl SolveResult {
    pub fn new(score: i32, mov: u64) -> SolveResult {
        SolveResult {
            score, mov,
            nodes_searched: 0,
            end_in: 0
        }
    }

}

#[wasm_bindgen]
pub fn solve(start: &BitBoard, depth: u8, solver: SolverType) -> SolveResult {
    let start = start.clone();
    let depth = u8::min(depth, 42 - start.number_of_stones() as u8);
    let mut nodes_searched: u64 = 0;

    let (score, mov) = match solver {
        SolverType::Strong => { solve_strong(start, depth, i32::MIN+2, i32::MAX-2, &mut nodes_searched)}
        SolverType::Weak => { solve_weak(start, depth, i32::MIN+2, i32::MAX-2, &mut nodes_searched)}
    };

    let end_in = match solver {
        SolverType::Strong => {0}
        SolverType::Weak => {
            if score == 0 {
                0
            } else {
                depth as i32 - score.abs() + 1
            }
        }
    };
    SolveResult {
        score,
        mov,
        nodes_searched,
        end_in
    }
    // return SolveResult::new(0,0)
}

/// Solves the board using a strong solver BitBoard::is_winning_board()
/// return score, best_move
pub fn solve_strong(start: BitBoard, depth: u8, mut alpha: i32, mut beta: i32, num_nodes: &mut u64) -> (i32, u64) {
    if start.has_lost() {
        // 100 as a high value to differentiate a guaranteed win from the heuristic
        return (-100 - depth as i32, 0);
    }
    *num_nodes += 1;

    // No conclusion found --> draw
    if depth == 0 {
        let score = start.heuristic_2();
        return (score, 0);
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
        let (score, _) =  solve_strong(new_board, depth-1, -beta, -alpha, num_nodes);
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
/// Solves the board using a weak solver BitBoard::is_winning_board()
/// return score, best_move
pub fn solve_weak(start: BitBoard, depth: u8, mut alpha: i32, mut beta: i32, num_nodes: &mut u64) -> (i32, u64) {
    if start.has_lost() {
        return (-1 - depth as i32, 0);
    }
    *num_nodes += 1;

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
        let (score, _) =  solve_weak(new_board, depth-1, -beta, -alpha, num_nodes);
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
    use crate::Engine::{solve_weak, solve};
    use crate::Engine::SolverType::{Weak, Strong};

    #[test]
    fn test_solve_easy() {
        let board_easy =
            "nnnnnnn
            nnnnnnn
            nnnnnnn
            nnnnnnn
            nnnnnnn
            nnncnnn";
        let bits = BitBoard::from_string(board_easy).unwrap();

        let result = solve(&bits, 3, Weak);

        let best_move_easy =
            "nnnnnnn
            nnnnnnn
            nnnnnnn
            nnnnnnn
            nnnpnnn
            nnnnnnn";
        let best_move_easy = BitBoard::from_string(best_move_easy).unwrap().player;

        println!("{}", result.score);
        assert_eq!(best_move_easy, result.mov)
    }

    // #[test]
    fn test_solve_strong() {
        let board_easy =
            "nnnnnnn
            nnnnnnn
            nnnnnnn
            nnppcnn
            npcccpn
            npcccpn";
        let bits = BitBoard::from_string(board_easy).unwrap();

        let result = solve(&bits, 1, Strong);

        let best_move_easy =
            "nnnnnnn
            nnnnnnn
            nnnnnpn
            nnnnnnn
            nnnnnnn
            nnnnnnn";
        let best_move_easy = BitBoard::from_string(best_move_easy).unwrap().player;

        println!("Score: {}", result.score);
        println!("Move: {}", result.mov);
    }

}