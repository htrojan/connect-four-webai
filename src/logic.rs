use wasm_bindgen::prelude::*;
use wasm_bindgen::__rt::std::fmt;
use std::fmt::Formatter;
use hashbrown::HashMap;
use std::hash::{Hash, Hasher};

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
                    FieldType::Computer => 1,
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
                    Some(FieldType::Computer) => {write!(f, "C");}
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
                        Some(FieldType::Computer) => {result.computer[streak] += 1},
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
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum FieldType {
    Computer,
    Player
}

impl FieldType {
    pub fn opposite(&self) -> FieldType {
        match self {
            FieldType::Computer => {FieldType::Player }
            FieldType::Player => {FieldType::Computer }
        }
    }
}

#[wasm_bindgen]
pub struct ABSolver {

}

struct TableEntry {
    score: Option<i32>,
}

#[wasm_bindgen]
#[derive(Copy, Clone, Eq, PartialEq)]
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

        // As the score always is evaluated for a specific player, both players are maximising
        // --> Both alpha and beta are at -infinity.
        let (score, best_move) =  ABSolver::solve_ab(board, depth, player, i32::MIN+2, i32::MAX-2, &mut trans_table);
        return BestMove::new(score.unwrap(), best_move.unwrap());
    }

    /***
    Returns the score in an option and the row for the move
     */
    fn solve_ab(board: &GameBoard, depth: u8, player: FieldType, mut alpha: i32, mut beta: i32, table: &mut HashMap<GameBoard, TableEntry>) -> (Option<i32>, Option<usize>) {
        // Evaluate board
        if depth == 0 {
            let score = table.get(board);

            let score = match score {
                None => {
                    let s = board.evaluate().score(&player);
                    table.insert(*board, TableEntry{score:s});
                    s
                },
                Some(entry) => entry.score
            };

            // let score = board.evaluate().score(&player);
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
            let score: Option<i32> = match new_board {
                None => {
                    // println!("Invalid Board!");
                    continue;}
                Some(ref board) => {
                    let (score, me) = ABSolver::solve_ab(board, depth-1, player.opposite(), -beta, -alpha, table);
                    score
                }
            };

            // if depth == 3 {
            //     println!("..................");
            // }
            // println!("Player: {:?}, Depth: {}, Row: {}, Score: {:?}", player, depth, i, score);
            // if (depth == 2 && i == 1) {
            //     match &new_board {
            //         None => {
            //             print!("Board = None");}
            //         Some(b) => {
            //             // println!("{}", b);
            //         }
            //     }
            //     // println!("This is a stop!");
            // }
            // If score is None, this node is invalid as both players have won at this point.
            // Reject all children and evaluate boards further up
            match score {
                None => {
                    // Evaluate this node
                    // Unwrap is safe as continue is called before if board is None
                    return (new_board.unwrap().evaluate().score(&player), Some(i));
                }
                Some(score) => {
                    // The score was done for the opposite player. Revert this
                    let score = -score;
                    //Alpha beta cutoff
                    if score >= beta {
                        // println!("Cutoff!");
                        return (Some(score), Some(i));
                    }
                    if score > max_score {
                        max_score = score;
                        best_move = i;
                        if score > alpha{
                            alpha = score;
                        }
                    }
                }
            }
        }

        return (Some(max_score), Some(best_move));
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
            return Some(FieldType::Computer);
        } else if self.player[3] > 0 {
            return Some(FieldType::Player);
        } else {
            None
        }
    }

}

impl Evaluation {
    /**
     * Gets score for specified player.
     * The return value is None if both players have a streak of 4, indicating this score
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
            FieldType::Computer => {
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
    use crate::logic::{FieldType, BOARD_HEIGHT, BOARD_WIDTH, GameBoard, ABSolver};

    #[test]
    fn correct_eval() {
        let p = Some(FieldType::Player);
        let c = Some(FieldType::Computer);
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
        let c = Some(FieldType::Computer);
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
        let best_move = ABSolver::solve(&board, 7, FieldType::Computer);
        println!("Score: {}, Move: {}", best_move.score, best_move.move_row);
    }

}