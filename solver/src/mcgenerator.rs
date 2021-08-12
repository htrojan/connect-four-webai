use crate::board::{BitBoard, MoveSetIterator};
use rand::prelude::*;
use rand_pcg;
use rand::SeedableRng;
use rand_pcg::Pcg64;
use crate::engine::{solve, SolverType};
use std::alloc::Global;

pub struct RandomGenerator {
    rng: Pcg64
}

impl RandomGenerator {
    pub fn new(seed: u64) -> RandomGenerator{
        RandomGenerator {
            rng: Pcg64::seed_from_u64(seed)
        }
    }

    /// Generates boards up to the max_depth given by randomly choosing
    /// one of the possible moves every turn. The resulting board is guaranteed
    /// to be valid with no player having won.
    /// If a situation is encountered during the 'virtual random play' where
    /// one player has the ability to win before reaching max_depth, the current board
    /// where no player has won is returned
    pub fn generate_board_random(&mut self, max_depth: u8) -> BitBoard {
        let mut board = BitBoard::empty();

        for i in 0..max_depth {

            // The other player has effectively won
            if board.forced_moves() > 1 {
                return board;
            }
            // The player now only has one option
            else if board.forced_moves() == 1 {
                board = board.play_field(board.forced_moves());
            }

            let possible_moves = board.all_possible_moves();
            let mut iterator = MoveSetIterator::from_moves(possible_moves);

            let move_index = self.rng.gen_range(0..(possible_moves.count_ones()-1));
            // Unwrap should be safe as the correct bounds are enforced in the range of
            // move_index
            let selected_move = iterator.nth(move_index as usize).unwrap();
            board = board.play_field(selected_move);
        }

        return board;
    }
}

pub struct BoardEvaluation {
    column_scores: [Option<i32>; 7]
}

impl BoardEvaluation {
    /// Uses the board evaluation scores to compute a softmax function for
    /// each of the components.
    /// If a move is invalid due to a already full column, the softmax value
    /// of this column-move is fixed at 0.
    /// As the strong solver emits scores at 100 + some value to indicate
    /// a guaranteed win/loss, the scores are divided by the scale_factor
    /// before calculating the exponential functions of the values to insure e^score
    /// is still in the range of a 32-bit floating point value.
    pub fn to_approx_softmax(&self, scale_factor: f32) -> Vec<f32, Global> {
        let total: f32 = self.column_scores
            .iter().filter(|x| x.is_some())
            .map(|x| (x.unwrap() as f32 / scale_factor).exp())
            .sum();

        let softmax = self.column_scores.iter()
            .map(|x| {
               match x  {
                   None => {0.}
                   Some(value) => {(*value as f32 / scale_factor).exp() / total}
               }
            }).collect();
        softmax
    }
}

/// Evaluates the board by calculating the score for each possible
/// move and returning their values.
/// The score calculation is done using the c4solver strong engine
/// up to the specified depth
pub fn evaluate_board(board: BitBoard, depth: u8) -> BoardEvaluation {
    let possible_moves = board.all_possible_moves();
    let moveset = MoveSetIterator::from_moves(possible_moves);
    let mut scores: [Option<i32>; 7] = [None; 7];

    for i in 0..7 {
        let mov = moveset.move_at_column(i);
        if mov > 0 {
            let new_board = board.play_field(mov);
            let score = solve(&new_board, depth, SolverType::Strong);
            scores[i as usize] = Some(score.score);
        }
    }

    BoardEvaluation {
        column_scores: scores
    }
}

#[cfg(test)]
mod tests {
    use crate::mcgenerator::{RandomGenerator, evaluate_board};

    #[test]
    fn test_generation() {
        let mut gen = RandomGenerator::new(42);
        for i in 0..200 {
            let generated = gen.generate_board_random(18);
            let eval = evaluate_board(generated, 7);
            println!("Number of stones: {}", generated.occupied.count_ones());
            println!("Eval: {:?}", eval.to_approx_softmax(50.));
        }
    }

}
