use std::fmt::Formatter;
use std::hash::{Hash, Hasher};

use hashbrown::HashMap;
use wasm_bindgen::__rt::std::fmt;
use wasm_bindgen::prelude::*;

use crate::BitBoard::FieldType;

// Connect four dimensions
pub const BOARD_WIDTH: usize = 7;
pub const BOARD_HEIGHT: usize = 6;

const SEARCH_ORDER: [usize; 7] = [3,2,4,1,5,0,6];

pub struct Direction {
    x: i32, y: i32
}

#[wasm_bindgen]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct GameBoard {
    fields: [[Option<FieldType>; BOARD_HEIGHT]; BOARD_WIDTH],
}

impl GameBoard {
    /**
    Converts the enum array to a byte array with the following layout:
    - 2 bits per field entry (3 possible states in total)
    - Each row is represented by 16 bits (2 bytes). This is equivalent to
      8 possible columns, but is done for simplicity (instead of the 6 real columns)

    **/
    fn to_bytes(&self) -> [u8; 14] {
        let mut bytes: [u8; BOARD_WIDTH*2] = [0; BOARD_WIDTH*2];

        for x in 0..BOARD_WIDTH {
            // Write first byte
            for y in 0..4 {
                let to_write = GameBoard::entry_to_num(self.fields[x][y]);
                let to_write = to_write << y * 2; // Shift the 2bit pattern to its appropriate position
                bytes[2 * x] = bytes[2 * x] + to_write;
            }
            // Write second byte
            for y in 4..BOARD_HEIGHT {
                let to_write = GameBoard::entry_to_num(self.fields[x][y]);
                let to_write = to_write << (y-4) * 2; // Shift the 2bit pattern to its appropriate position
                bytes[2 * x + 1] = bytes[2 * x + 1] + to_write;
            }
        }
        bytes
    }

    #[inline]
    fn entry_to_num(entry: Option<FieldType>) -> u8 {
        match entry {
            None => {0}
            Some(s) => {
                match s {
                    FieldType::Opponent => 1,
                    FieldType::Player => 2,
                }
            }
        }
    }
}
impl Hash for GameBoard {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write(&self.to_bytes())
    }
}

impl fmt::Display for GameBoard {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for x in 0..BOARD_WIDTH {
            for y in 0..BOARD_HEIGHT {
                match &self.fields[x][y] {
                    None => {write!(f, "N");}
                    Some(FieldType::Opponent) => {write!(f, "C");}
                    Some(FieldType::Player) => {write!(f, "P");}
                }
            }
            write!(f, "\n");
        }
        Ok(())
    }
}


impl Direction {
    const fn up() -> Direction {Direction{ x: 0, y: 1 }}
    const fn down() -> Direction {Direction{ x: 0, y: -1 }}
    const fn left() -> Direction {Direction{ x: -1, y: 0 }}
    const fn right() -> Direction {Direction{ x: 1, y: 0 }}
    const fn up_right() -> Direction {Direction{ x: 1, y: 1 }}
    const fn up_left() -> Direction {Direction{ x: -1, y: 1 }}

    const fn opposite(&self) -> Direction {
        Direction {
            x: -self.x,
            y: -self.y
        }
    }

    const UP: Direction = Direction::up();
    const DOWN: Direction = Direction::down();
    const LEFT: Direction = Direction::left();
    const RIGHT: Direction = Direction::right();
    const UP_LEFT: Direction = Direction::up_left();
    const UP_RIGHT: Direction = Direction::up_right();

    // Returns all directions needed during traversal
    const REAL: [Direction; 4] = [Direction::UP, Direction::RIGHT, Direction::UP_LEFT, Direction::UP_RIGHT];
}

#[wasm_bindgen]
impl GameBoard {

    pub fn empty() -> GameBoard {
        GameBoard {
            fields: [[None; BOARD_HEIGHT]; BOARD_WIDTH]
        }
    }

    pub fn get_x() -> usize {
        BOARD_WIDTH
    }
    pub fn get_y() -> usize {
        BOARD_HEIGHT
    }

    pub fn at(&self, x: usize, y: usize) -> Option<FieldType> {
        self.fields[x][y]
    }

    pub fn set(& mut self, x:usize, y:usize, field: Option<FieldType>) {
        self.fields[x][y] = field;
    }

    pub fn empty_fields(&self) -> u8 {
        let mut empty = 0;

        for x in 0..BOARD_WIDTH {
            for y in 0..BOARD_HEIGHT {
                if self.fields[x][y] == None {
                    empty = empty + 1;
                }
            }
        }
        return empty;
    }

    // Constructs a cloned GameBoard with the new move applied
    pub fn new_with_move(&self, row: usize, player: FieldType) -> Option<GameBoard>{
        // The move has to be a row in the board
        if row >= BOARD_WIDTH {
            return None;
        }

        // Check if there is space
        for i in 0..BOARD_HEIGHT {
            if self.fields[row as usize][i] == None {
                // Construct new GameBoard
                let mut fields :[[Option<FieldType>; BOARD_HEIGHT]; BOARD_WIDTH]
                    = [[None; BOARD_HEIGHT]; BOARD_WIDTH];
                fields.copy_from_slice(&self.fields);
                fields[row as usize][i] = Some(player);
                return Some(GameBoard::new(fields))
            }
        }

        None
    }
    pub fn evaluate(&self) -> Evaluation{
        let mut result = Evaluation::default();

        for x in 0..BOARD_WIDTH {
            for y in 0..BOARD_HEIGHT {
                // Stores the last color to compare the current one to
                let last_color: Option<FieldType> = self.fields[x][y];
                if self.fields[x][y] == None{
                    continue;
                }

                for dir in Direction::REAL.iter() {

                    // Flag that is set to true when at least one way this color-snake can be expanded
                    // To a four-snake is not blocked
                    let mut open = false;

                    // The number of entries in this color-snake
                    let mut streak = 0;

                    // If this field has no color, ignore
                    // Check the opposite direction for the same color. If the same color
                    // is there, this line was already checked before
                    let opposite_dir = dir.opposite();
                    let before_x = opposite_dir.x + x as i32;
                    let before_y = opposite_dir.y + y as i32;

                    if GameBoard::position_in_bounds(before_x, before_y) {
                        if self.fields[before_x as usize][before_y as usize] == last_color {
                            // Already checked
                            streak = 0;
                            continue;
                        } else if self.fields[before_x as usize][before_y as usize] == None{
                            // If one end of the color-snake is open, this is an open snake
                            open = true;
                        }
                    }

                    // Go further into the direction up to a color change
                    // last_color = self.fields[position as usize];
                    let mut new_x = x as i32 + dir.x;
                    let mut new_y = y as i32 + dir.y;

                    while GameBoard::position_in_bounds(new_x, new_y) && self.fields[new_x as usize][new_y as usize] == last_color {
                        streak += 1;
                        new_x += dir.x;
                        new_y += dir.y;
                    }

                    // Check if the end in this direction is open and set open flag if not
                    // already set from the opposite direction
                    if open == false && GameBoard::position_in_bounds(new_x, new_y) && self.fields[new_x as usize][new_y as usize] == None{
                        open = true;
                    }

                    // Its possible to have a streak >= 4 --> Reset to 3 maximum (as 0 to 3 are the
                    // possible values)
                    if streak >= 4 {
                        streak = 3;
                    }

                    // If no end is open, ignore this color-snake
                    // A streak of 4 is a win. So the openness does not matter
                    if open == false  && streak < 3{
                        continue;
                    }
                    match last_color {
                        None => {}
                        Some(FieldType::Opponent) => {result.computer[streak] += 1},
                        Some(FieldType::Player) => {result.player[streak] += 1}
                    }

                }
            }
        }
        result
    }
}

impl GameBoard {

    pub fn new(fields: [[Option<FieldType>; BOARD_HEIGHT]; BOARD_WIDTH]) -> GameBoard {
        GameBoard {
            fields
        }
    }



    const fn position_in_bounds(x: i32, y: i32) -> bool {
        return x >= 0 && y >= 0 && x < BOARD_WIDTH as i32 && y < BOARD_HEIGHT as i32
    }

}

#[wasm_bindgen]
pub struct ABSolver {

}

enum NodeType {
    // Represents an invalid move. Does not have to be evaluated further
    Invalid,
    Upperbound,
    Lowerbound,
    Exact
}
struct TableEntry {
    flag: NodeType,
    value: i32,
    best_move: usize
}

#[wasm_bindgen]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct BestMove {
    pub score: i32,
    pub move_row: usize
}

impl BestMove {
    pub fn new(score: i32, move_row: usize) -> BestMove {
        BestMove {
            score, move_row
        }
    }
}
#[wasm_bindgen]
impl ABSolver {
    pub fn solve(board: &GameBoard, depth: u8, player: FieldType) -> BestMove {
        let free_fields = board.empty_fields();
        let depth = u8::min(free_fields, depth);
        let mut trans_table = HashMap::new();

        // As the Score always is evaluated for a specific player, both players are maximising
        // --> Both alpha and beta are at -infinity.
        let (score, best_move) =  ABSolver::solve_ab(board, depth, player, i32::MIN+2, i32::MAX-2, &mut trans_table);
        return BestMove::new(score.unwrap(), best_move.unwrap());
    }

    /***
    Returns the Score in an option and the row for the move
     */
    fn solve_ab(board: &GameBoard, depth: u8, player: FieldType, mut alpha: i32, mut beta: i32, table: &mut HashMap<GameBoard, TableEntry>) -> (Option<i32>, Option<usize>) {
        let orig_alpha = alpha;
        // Check transposition table for move
        let cache = table.get(board);

        // Check alpha and beta values stored in cache
        match cache {
            Some(entry) => {
                match entry.flag {
                    NodeType::Invalid => {return (None, None)}
                    NodeType::Upperbound => {beta = i32::min(beta, entry.value)}
                    NodeType::Lowerbound => {alpha = i32::max(alpha, entry.value)}
                    NodeType::Exact => {return (Some(entry.value), Some(entry.best_move))}
                }
                if alpha >= beta {
                    return (Some(entry.value), Some(entry.best_move))
                }
            }
            None => {}
        }

        // Evaluate board
        if depth == 0 {
            let score = board.evaluate().score(&player);
            // There is no best move. This is the only board. Return none for the best move
            return (score, None);
        }


        let mut max_score = i32::MIN;
        let mut best_move: usize = 0;
        // Possible moves
        for i in &SEARCH_ORDER {
            let i = *i;
            let new_board = board.new_with_move(i, player);


            // A return of None marks an invalid board
            let score = match new_board {
                None => { continue;}
                Some(ref board) => {
                    let (s, _ ) = ABSolver::solve_ab(board, depth-1, player.opposite(), -beta, -alpha, table);
                    s
                }
            };

            // If Score is None, this node is invalid as both players have won at this point.
            // Reject all children and evaluate boards further up
            match score {
                None => {
                    // Evaluate this node. Insert the invalid as invalid into the transposition table.
                    // Is this necessary? Maybe leave invalid nodes out of the table ...?
                    // They could take away space that can be used in a better way
                    table.insert(new_board.unwrap(), TableEntry {
                        flag: NodeType::Invalid,
                        value: 0,
                        best_move
                    });
                    // Unwrap is safe as continue is called before if board is None
                    return (new_board.unwrap().evaluate().score(&player), Some(i));
                }
                Some(score) => {
                    let score = -score;
                    // Update max_score and best_move if this move is better
                    if score > max_score {
                        max_score = score;
                        best_move = i;
                    }
                    alpha = i32::max(alpha, score);

                    if alpha >= beta {
                        break
                    }
                }
            }
        }

        let mut node_type = NodeType::Invalid;
        // Fill transposition table with values of this node
        if max_score <= orig_alpha{
            node_type = NodeType::Upperbound;
        } else if max_score >= beta {
            node_type = NodeType::Lowerbound;
        } else {
            node_type = NodeType::Exact;
        }

        // Store or update entry
        let entry = TableEntry {
            flag: node_type,
            value: max_score,
            best_move
        };
        table.insert(*board, entry);


        return (Some(max_score), Some(best_move));
    }

    fn store_lower(table: &mut HashMap<GameBoard, TableEntry>, board: &GameBoard, lower: i32) {

    }
    fn store_upper(table: &mut HashMap<GameBoard, TableEntry>, board: &GameBoard, upper: i32) {

    }
    fn store_exact(table: &mut HashMap<GameBoard, TableEntry>, board: &GameBoard, exact: i32) {

    }

    pub fn solve_mtdf(board: &GameBoard, depth: u8, player: FieldType) -> BestMove {
        let mut optimal_move = Some(0);
        let free_fields = board.empty_fields();
        let depth = u8::min(free_fields, depth);

        // Get initial guess by searching the tree with a wide window in a short depth
        let mut guess = ABSolver::solve(board, u8::min(depth, 3), player);

        // Only every second iteration will be searched.
        let iterations: u8 = (depth - 3)/2;

        for d in 0..iterations{
            guess = ABSolver::solve_mtdf_guessed(board, depth, player, guess.score);
        }
        guess
    }

    pub fn solve_mtdf_guessed(board: &GameBoard, depth: u8, player: FieldType, initial_guess: i32) -> BestMove {
        let mut guess = initial_guess;
        let mut optimal_move = Some(0);
        let free_fields = board.empty_fields();
        let depth = u8::min(free_fields, depth);

        let mut trans_table = HashMap::new();
        let mut upperbound = i32::MAX - 2;
        let mut lowerbound = i32::MIN + 2;

        loop {
            let beta = if guess == lowerbound {
                guess + 1
            } else {
                guess
            };
            let (score, row) = ABSolver::solve_ab(board, depth, player, beta-1, beta, &mut trans_table);
            // (guess, optimal_move) = ABSolver::solve_ab(board, depth, player, beta-1, beta, &mut trans_table);
            optimal_move = row;
            guess = score.unwrap();

            if guess < beta {
                upperbound = guess;
            } else {
                lowerbound = guess;
            }

            if lowerbound >= upperbound {
                break;
            }
        }
        BestMove {
            score: guess,
            move_row: optimal_move.unwrap()
        }
    }
}

#[wasm_bindgen]
#[derive(Debug, Eq, PartialEq)]
pub struct Evaluation {
    computer: [i32;4],
    player: [i32;4],
}

#[wasm_bindgen]
impl Evaluation {
    pub fn player_win(&self) -> Option<FieldType> {
        if self.computer[3] > 0 {
            return Some(FieldType::Opponent);
        } else if self.player[3] > 0 {
            return Some(FieldType::Player);
        } else {
            None
        }
    }

}

impl Evaluation {
    /**
     * Gets Score for specified player.
     * The return value is None if both players have a streak of 4, indicating this Score
     * is invalid.
    **/
    pub fn score(&self, player: &FieldType) -> Option<i32> {
        if self.computer[3] > 0 && self.player[3] > 0 {
            // println!("Score for both players!");
            return None;
        }

        let mut player_score = (
            self.computer[0] + 6*self.computer[1] + 11*self.computer[2] + 100000 * self.computer[3]
                - (self.player[0] + 6*self.player[1] + 11*self.player[2] + 100000 * self.player[3])
        );
        match player {
            FieldType::Player => {
                player_score = -player_score
            }
            FieldType::Opponent => {
            }
            _ => {panic!("Score called for FieldType::None!")}
        };
        Some(player_score)
    }
}

impl Evaluation {
    pub fn default() -> Evaluation {
        Evaluation {
            computer: [0;4],
            player: [0;4],
        }
    }
}

#[cfg(test)]
mod test{
    use hashbrown::HashMap;

    use crate::BitBoard::FieldType;
    use crate::logic::{ABSolver, BestMove, BOARD_HEIGHT, BOARD_WIDTH, GameBoard};

    #[test]
    fn correct_eval() {
        let p = Some(FieldType::Player);
        let c = Some(FieldType::Opponent);
        let n = None::<FieldType>;
        let fields: [[Option<FieldType>; BOARD_HEIGHT]; BOARD_WIDTH]
            = [
            [n,n,n,n,n,n,],
            [c,c,c,c,n,n,],
            [p,n,n,n,n,n,],
            [p,n,n,n,n,n,],
            [c,n,n,n,n,n,],
            [n,n,n,n,n,n,],
            [n,n,n,n,n,n,],
        ];
        let board = GameBoard::new(fields);
        let result = board.evaluate();
        println!("{:?}", result);
    }

    #[test]
    fn correct_solve() {
        let p = Some(FieldType::Player);
        let c = Some(FieldType::Opponent);
        let n = None::<FieldType>;
        let fields: [[Option<FieldType>; BOARD_HEIGHT]; BOARD_WIDTH]
            = [
            [n,n,n,n,n,n,],
            [c,c,c,n,n,n,],
            [p,n,n,n,n,n,],
            [p,n,n,n,n,n,],
            [c,n,n,n,n,n,],
            [n,n,n,n,n,n,],
            [n,n,n,n,n,n,],
        ];
        let board = GameBoard::new(fields);
        let best_move = ABSolver::solve(&board, 7, FieldType::Opponent);
        println!("Score: {}, Move: {}", best_move.score, best_move.move_row);
    }

    #[test]
    fn correct_solve_mtdf() {
        let p = Some(FieldType::Player);
        let c = Some(FieldType::Opponent);
        let n = None::<FieldType>;
        let fields: [[Option<FieldType>; BOARD_HEIGHT]; BOARD_WIDTH]
            = [
            [n,n,n,n,n,n,],
            [n,n,n,n,n,n,],
            [n,n,n,n,n,n,],
            [p,n,n,n,n,n,],
            [n,n,n,n,n,n,],
            [n,n,n,n,n,n,],
            [n,n,n,n,n,n,],
        ];
        let board = GameBoard::new(fields);

        // let mut trans_table = HashMap::new();

        // As the Score always is evaluated for a specific player, both players are maximising
        // --> Both alpha and beta are at -infinity.
        // let (score, best_move) =  ABSolver::solve_ab(&board, 9, FieldType::Computer, 0, 1, &mut trans_table);
        let best_move= ABSolver::solve_mtdf_guessed(&board, 11, FieldType::Opponent, 1);
        // let best_move = BestMove::new(score.unwrap(), best_move.unwrap());

        // let best_move = ABSolver::solve_mtdf(&board, 11, FieldType::Computer);
        println!("Score: {}, Move: {}", best_move.score, best_move.move_row);
        let best_move_traditional = ABSolver::solve(&board, 11, FieldType::Opponent);
        println!("Score: {}, RealMove: {}", best_move_traditional.score, best_move_traditional.move_row);
        assert_eq!(best_move, best_move_traditional)
    }

}