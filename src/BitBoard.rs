use crate::logic;
use crate::logic::{BOARD_WIDTH, BOARD_HEIGHT, FieldType};
use wasm_bindgen::prelude::*;

/// Encoding: Column-first-order, 8-bit per column.
/// That way 2 bits per column (they are 6 fields high) are left unused.
#[wasm_bindgen]
#[derive(Default, Eq, PartialEq, Debug, Copy, Clone)]
pub struct BitBoard {
    pub player: u64,
    pub occupied: u64,
}



#[wasm_bindgen]
impl BitBoard {

    #[inline]
    pub fn number_of_stones(&self) -> u32 {
        self.occupied.count_ones()
    }
    pub fn empty() -> BitBoard {
        BitBoard::default()
    }
    /// Plays a stone at the given position without swapping players.
    /// if player is true, the current player will be played.
    /// If false the opposing player will be played
    pub fn set_at_coordinate(&mut self, x: u8, y: u8, field: Option<FieldType>) {
        let to_play = BitBoard::coordinate_to_field(x,y);
        self.set_at(to_play, field);
    }

    pub fn set_at(&mut self, to_play: u64, field: Option<FieldType>) {
        match field {
            None => {
                // Set field to zero for both players
                self.player & (!to_play);
                self.occupied & (!to_play);
            }

            Some(FieldType::Player) => {
                self.player |= to_play;
                self.occupied |= to_play;
            }

            Some(FieldType::Opponent) => {
                self.occupied |= to_play;
            }
        }

    }

    pub fn get_at(&self, x: u8, y: u8) -> Option<FieldType> {
        let field = BitBoard::coordinate_to_field(x,y);
        if (self.player & field) > 0 {
            return Some(FieldType::Player)
        } else if (self.occupied & field) > 0 {
            return Some(FieldType::Opponent)
        }

        None
    }

    #[inline]
    fn coordinate_to_field(x: u8, y: u8) -> u64 {
        let pos = x*8 + y;
        1<<pos
    }

    /// Switches the active player represented by the "player" bits
    pub fn switch_players(&mut self) {
        let occupied = self.occupied;
        let player = self.player;

        self.player = occupied - player;
    }

    /// Returns whether the current player has won
    pub fn has_won(&self) -> bool {
        BitBoard::is_winning_board(self.player)
    }

    /// Returns whether the current player has lost
    pub fn has_lost(&self) -> bool {
        BitBoard::is_winning_board(self.occupied - self.player)
    }

    pub fn is_move_valid(&self, column: u8) -> bool {
        let all_valid = self.all_possible_moves();
        BitBoard::move_in_row(all_valid, column as u64) > 0
    }

    pub fn play_column(&self, column: u8, player: FieldType) -> Option<BitBoard> {
        let all_valid = self.all_possible_moves();
        let mov = BitBoard::move_in_row(all_valid, column as u64);

        if mov > 0 {
            let board = match player {
                FieldType::Opponent => {
                    BitBoard {
                        player: self.player,
                        occupied: self.occupied | mov
                    }
                }
                FieldType::Player => {
                    BitBoard {
                        player: self.player | mov,
                        occupied: self.occupied | mov
                    }
                }
            };
            return Some(board);
        }
        return None;
    }

}
/// Stores the stones of one player in "player".
/// All occupied fields are marked in "occupied".
/// -> player2 = !player & occupied
impl BitBoard {
    pub const FIRST_COLUMN: u64 = 1 + (1<<1) + (1<<2) + (1<<3) + (1<<4) + (1<<5) + (1<<6) + (1<<7);
    pub const BOTTOM_LINE: u64 = 1 + (1<<8) + (1<<16) + (1<<24) + (1<<32) + (1<<40) + (1<<48) + (1<<56);
    pub const TOP_LINE: u64 = (1<<7) + (1<<15) + (1<<23) + (1<<31) + (1<<39) + (1<<47) + (1<<55) + (1<<63);
    /// Exclude The top two rows (the 2 extra bits per column that store no information)
    /// and the last (unnecessary) column with no information
    pub const PLAYABLE_FIELDS: u64 = u64::MAX
        & !(BitBoard::TOP_LINE | (BitBoard::TOP_LINE>>1))
        & !(BitBoard::FIRST_COLUMN<<56);


    /// Sets the corresponding field and changes the player
    pub fn play_field(&self, field: u64) -> BitBoard {
        BitBoard {
            player: (self.occupied - self.player),
            occupied: self.occupied | field
        }
    }

    /// Constructs a board from a string representing the board.
    /// p is the playing player, o the opponent
    /// n is nothing - an empty field.
    pub fn from_string(input: &str) -> Result<BitBoard, String> {
        // Remove all whitespace
        let input: String = input.split_whitespace().collect();

        if input.chars().count() != BOARD_WIDTH * BOARD_HEIGHT {
            return Err(String::from(format!("The board string has to be of length 42, but has length {}", input.chars().count())))
        }
        let mut board = BitBoard::default();

        // The first element of the string is the position in the up-left corner,
        // Represented by the 6th byte
        let mut bit: u64 = 1<<BOARD_HEIGHT-1;

        // Counts how many chars have been added to the current column
        let mut column_count = 0;

        for c in input.chars() {
            // println!("Iteration");
            if column_count > BOARD_HEIGHT {
                // Shift to the next position at the left
                bit = bit >> (BOARD_WIDTH * 8 + 1);
                column_count = 0;
            }

            if c == 'c' {
                board.occupied = board.occupied | bit;

            } else if c == 'p' {
                // println!("Player at {}", bit);
                board.occupied = board.occupied | bit;
                board.player = board.player | bit;
            } else if c == 'n' {
                // Do nothing
            } else {
                return Err(String::from(format!("Invalid character \'{}\' encountered", c)))
            }
            column_count += 1;
            bit <<= 8;
        }
        Ok(board)
    }


    /// Returns if this move would win the game by looking for adjacent rows of three
    pub fn is_winning_move(&self, field: u64) -> bool {
        let v_mask: u64 = 0b111;
        let h_mask: u64 = 1 + (1<<8) + (1<<16) + (1<<24);
        let d1_mask: u64 = 1 + (1<<9) + (1<<18) + (1<<27); // 9 bits between
        let d2_mask: u64 = (1<<3) + (1<<10) + (1<<17) + (1<<24); // 7 bits between
        let board = self.player + field;

        let position_bit = u64::trailing_zeros(field) + 1;

        // Check vertical. Only down is possible
        let mask = v_mask << position_bit - 4;
        let mut winning = (board & mask) == mask;

        // Check horizontal
        let mask = h_mask << (position_bit - 1); // Left field of mask is now aligned
        for i in 0..4 {
            let new_mask = mask >> (i*8);
            winning |= (board & new_mask) == new_mask;
        }

        // Check first diagonal (up-right)
        let mask = d1_mask << (position_bit - 1);
        for i in 0..4 {
            let new_mask = mask >> (i*9);
            winning |= (board & new_mask) == new_mask;
        }

        // Check second diagonal (up-left)
        let mask = d2_mask << (position_bit - 4);
        for i in 0..4 {
            let new_mask = mask >> (i*7);
            winning |= (board & new_mask) == new_mask;
        }

        return winning;
    }



    #[inline]
    fn is_winning_board(player_position: u64) -> bool {
        let board = player_position;

        // Check vertical
        let v = (board<<2) & board;
        let mut winning= ((v<<1) & v) > 0;

        // Check horizontal
        let h = (board<<16) & board;
        winning |= ((h<<8) & h) > 0;

        // Check diagonal up_right
        let d1 = (board<<18) & board;
        winning |= ((d1<<9) & d1) > 0;

        // Check diagonal up_left
        let d2 = (board<<14) & board;
        winning |= ((d2<<7) & d2) > 0;

        winning
    }



    /// Returns a board representing all possible locations
    /// for a new move
    #[inline]
    pub fn all_possible_moves(&self) -> u64 {
        let x = BitBoard::PLAYABLE_FIELDS;
        // Sets the topmost line so that unplayed columns are taken into account
        // for the possible moves
        let occupied = self.occupied | BitBoard::TOP_LINE;

        // Shifts everything one field up and compares it to the state before.
        // The top positions of each row have been moved up.
        // Therefore all positions showing a difference are positions of the top fields
        // that have bee moved.
        let possible = occupied ^ ((occupied<<1) + 1);
        possible & BitBoard::PLAYABLE_FIELDS
    }

    /// Returns the next possible move in the column out of all possible moves.
    /// Returns 0 if no move is possible
    #[inline]
    pub fn move_in_row(possible_moves: u64, column: u64) -> u64 {
        let column_mask = BitBoard::FIRST_COLUMN << (8*column);
        column_mask & possible_moves
    }

    /// Calculates the heuristic score of the board
    pub fn heuristic(&self) -> i32 {
        let player = self.player;
        let occupied = self.occupied;
        let opponent = occupied - player;

        BitBoard::score_new(player, occupied) - BitBoard::score_new(opponent, occupied)
    }

    /// Returns the chains of three and the
    #[inline]
    fn chain_helper(player: u64, occupied_closed: u64, closed_mask: u64, offset: u64) -> (u64, u64, u64) {
        let chains_three = (player << offset) & (player >> offset) & player;
        let closed_r = (chains_three << 2*offset) & occupied_closed;
        let closed_l = (chains_three >> 2*offset) & occupied_closed;
        let closed = (closed_l << 4*offset) & closed_r;
        let closed_border = (closed_mask & chains_three) & (closed_r >> 2*offset);
        (chains_three, closed, closed_border)
    }

    pub fn score_new(player: u64, occupied: u64) -> i32{
        let occupied_closed = occupied | !BitBoard::PLAYABLE_FIELDS;

        // Vertical
        let closed_mask: u64 = 0x2;
        let (chains_three, closed, closed_border) = BitBoard::chain_helper(player, occupied_closed, closed_mask, 1);
        let mut chains = chains_three.count_ones() as i32 - closed.count_ones() as i32 - closed_border.count_ones() as i32;

        // Horizontal
        let closed_mask: u64 = 0x3F00;
        let (chains_three, closed, closed_border) = BitBoard::chain_helper(player, occupied_closed, closed_mask, 8);
        chains += chains_three.count_ones() as i32 - closed.count_ones() as i32 - closed_border.count_ones() as i32;

        // Diagonal up-right
        let closed_mask: u64 = 0x21E00;
        let (chains_three, closed, closed_border) = BitBoard::chain_helper(player, occupied_closed, closed_mask, 9);
        chains += chains_three.count_ones() as i32 - closed.count_ones() as i32 - closed_border.count_ones() as i32;

        // Diagonal down-right
        let closed_mask: u64 = 0x1E00;
        let (chains_three, closed, closed_border) = BitBoard::chain_helper(player, occupied_closed, closed_mask, 7);
        chains += chains_three.count_ones() as i32 - closed.count_ones() as i32 - closed_border.count_ones() as i32;

        chains

    }

}

#[cfg(test)]
mod tests {
    use crate::BitBoard::BitBoard;
    use crate::logic::GameBoard;

    #[test]
    fn test_new_score_down_right() {
        let board_1 =
            "pnnpnnn
             npnnpnn
             nnpnnpn
             npnnpnn
             nnpnnpn
             nnnpnnp";
        let board_2 =
            "pnnnpnn
             npnnnpn
             nnpcnnp
             pnncpnn
             npnnnpn
             nnpnnnp";
        let board_1 = BitBoard::from_string(board_1).unwrap();
        let player = board_1.player;
        let occupied = board_1.occupied;
        assert_eq!(4, BitBoard::score_new(player, occupied));
        let board_2 = BitBoard::from_string(board_2).unwrap();
        let player = board_2.player;
        let occupied = board_2.occupied;
        assert_eq!(0, BitBoard::score_new(player, occupied));
    }
    #[test]
    fn test_new_score_vertical() {
        let board_1 =
            "nnnnnnn
            nnnnnnn
            nnnnnnn
            pnnpnnn
            pnnpnnn
            pnnpnnn";
        let board_2 =
            "nnnnnnn
            nnnnnnn
            cnncnnn
            pnnpnnn
            pnnpnnn
            pnnpnnn";
        let board_1 = BitBoard::from_string(board_1).unwrap();
        let player = board_1.player;
        let occupied = board_1.occupied;
        assert_eq!(2, BitBoard::score_new(player, occupied));
        let board_2 = BitBoard::from_string(board_2).unwrap();
        let player = board_2.player;
        let occupied = board_2.occupied;
        assert_eq!(0, BitBoard::score_new(player, occupied));

    }
    #[test]
    fn test_new_score_horizontal() {
        let board_1 =
            "pppnppp
            nnnnnnn
            nnnnnnn
            pppnppp
            nnnnnnn
            pppnppp";
        let board_2 =
            "pppcppp
            nnnnnnn
            nnnnnnn
            pppcppp
            nnnnnnn
            pppcppp";
        let board_1 = BitBoard::from_string(board_1).unwrap();
        let player = board_1.player;
        let occupied = board_1.occupied;
        assert_eq!(6, BitBoard::score_new(player, occupied));
        let board_2 = BitBoard::from_string(board_2).unwrap();
        let player = board_2.player;
        let occupied = board_2.occupied;
        assert_eq!(0, BitBoard::score_new(player, occupied));
    }

    #[test]
    fn test_new_score_up_right() {
        let board_1 =
            "nnnnnnp
             nnnnnpn
             nnnnpnn
             nnpnnnn
             npnnnnn
             pnnnnnn";
        let board_2 =
            "nnpnnnp
             npnnnpn
             pnncpnn
             nnpcnnp
             npnnnpn
             pnnnpnn";
        let board_1 = BitBoard::from_string(board_1).unwrap();
        let player = board_1.player;
        let occupied = board_1.occupied;
        assert_eq!(2, BitBoard::score_new(player, occupied));
        let board_2 = BitBoard::from_string(board_2).unwrap();
        let player = board_2.player;
        let occupied = board_2.occupied;
        assert_eq!(0, BitBoard::score_new(player, occupied));
    }

    #[test]
    fn test_play_move() {
        let board =
            "nnnnnnn
            nnnnnnn
            nnnnnnn
            nnnnnnn
            nnnpcnn
            nnnpcnn";
        let board = BitBoard::from_string(board).unwrap();

        let to_play =
            "nnnnnnn
            nnnnnnn
            nnnnnnn
            nnnpnnn
            nnnnnnn
            nnnnnnn";
        let to_play = BitBoard::from_string(to_play).unwrap().player;

        let expected =
            "nnnnnnn
            nnnnnnn
            nnnnnnn
            nnncnnn
            nnncpnn
            nnncpnn";
        let expected = BitBoard::from_string(expected).unwrap();

        assert_eq!(board.play_field(to_play), expected);
    }

    #[test]
    fn test_board_from_str() {
        let board_1 =
            "nnnnnnn
            nnnnnnn
            nnnnnnn
            nnnnnnn
            nnnnnnn
            nnnnnnn";

        let board_2 =
            "nnnnnnn
            nnnnnnn
            nnnnnnn
            nnnnnnn
            npnnnnn
            nnnnnnn";

        let bits = BitBoard::from_string(board_1).unwrap();
        assert_eq!(bits.player, 0);
        assert_eq!(bits.occupied, 0);

        let bits = BitBoard::from_string(board_2).unwrap();
        assert_eq!(bits.player, 1<<9);
        assert_eq!(bits.occupied, 1<<9);
    }

    #[test]
    fn test_winning_move_vertical() {
        let board_1 =
            "nnnnnnn
            nnnnnnn
            nnnnnnn
            npnnnnn
            npnnnnn
            npnnnnn";
        let board_2 =
            "nnnnnnn
            nnnnnnn
            nnnnnnn
            npnnnnn
            ncnnnnn
            ncnnnnn";
        let bits_1 = BitBoard::from_string(board_1).unwrap();
        let bits_2 = BitBoard::from_string(board_2).unwrap();

        let player_move =
            "nnnnnnn
            nnnnnnn
            npnnnnn
            nnnnnnn
            nnnnnnn
            nnnnnnn";
        let position = BitBoard::from_string(player_move).unwrap().player;

        assert!(bits_1.is_winning_move(position));
        assert!(!bits_2.is_winning_move(position));
    }

    #[test]
    fn test_winning_board_vertical() {
        let board_1 =
            "nnnnnnn
            nnnnnnn
            npnnnnn
            npnnnnn
            npnnnnn
            npnnnnn";
        let board_2 =
            "nnnnnnn
            nnnnnnn
            npnnnnn
            npnnnnn
            ncnnnnn
            ncnnnnn";
        let bits_1 = BitBoard::from_string(board_1).unwrap();
        let bits_2 = BitBoard::from_string(board_2).unwrap();

        assert!(bits_1.has_won());
        assert!(!bits_2.has_won());
    }

    #[test]
    fn test_winning_move_horizontal() {
        let board_1 =
            "nnnnnnn
            nnnnnnn
            nnnnnnn
            nnnnnnn
            nnnnnnn
            pppnnnn";
        let board_2 =
            "nnnnnnn
            nnnnnnn
            nnnnnnn
            nnnnnnn
            nnnnnnn
            nnnnppp";
        let board_3 =
            "nnnnnnn
            nnnnnnn
            nnnnnnn
            nnnnnnn
            nnnnnnn
            nnnnppn";
        let bits_1 = BitBoard::from_string(board_1).unwrap();
        let bits_2 = BitBoard::from_string(board_2).unwrap();
        let bits_3 = BitBoard::from_string(board_3).unwrap();

        let player_move =
            "nnnnnnn
            nnnnnnn
            nnnnnnn
            nnnnnnn
            nnnnnnn
            nnnpnnn";
        let position = BitBoard::from_string(player_move).unwrap().player;

        assert!(bits_1.is_winning_move(position));
        assert!(bits_2.is_winning_move(position));
        assert!(!bits_3.is_winning_move(position));
    }

    #[test]
    fn test_winning_board_horizontal() {
        let board_1 =
            "nnnnnnn
            nnnnnnn
            nnnnnnn
            nnnnnnn
            nnnnnnn
            ppppnnn";
        let board_2 =
            "nnnnnnn
            nnnnnnn
            nnnnnnn
            nnnnnnn
            nnnnnnn
            nnnpppp";
        let board_3 =
            "nnnnnnn
            nnnnnnn
            nnnnnnn
            nnnnnnn
            nnnnnnn
            nnnpppn";
        let bits_1 = BitBoard::from_string(board_1).unwrap();
        let bits_2 = BitBoard::from_string(board_2).unwrap();
        let bits_3 = BitBoard::from_string(board_3).unwrap();


        assert!(bits_1.has_won());
        assert!(bits_2.has_won());
        assert!(!bits_3.has_won());
    }

    #[test]
    fn test_winning_move_up_right() {
        let board_1 =
            "nnnnnnn
            nnnnnnn
            nnnnnnn
            nnnpnnn
            nnpnnnn
            npnnnnn";
        let board_2 =
            "nnnnnnn
            nnnnnpn
            nnnnnnn
            nnnpnnn
            nnpnnnn
            nnnnnnn";
        let bits_1 = BitBoard::from_string(board_1).unwrap();
        let bits_2 = BitBoard::from_string(board_2).unwrap();

        let player_move =
            "nnnnnnn
            nnnnnnn
            nnnnpnn
            nnnnnnn
            nnnnnnn
            nnnnnnn";
        let position = BitBoard::from_string(player_move).unwrap().player;

        assert!(bits_1.is_winning_move(position));
        assert!(bits_2.is_winning_move(position));
    }

    #[test]
    fn test_winning_board_up_right() {
        let board_1 =
            "nnnnnnn
            nnnnnnn
            nnnnpnn
            nnnpnnn
            nnpnnnn
            npnnnnn";
        let board_2 =
            "nnnnnnp
            nnnnnpn
            nnnnpnn
            nnnpnnn
            nnpnnnn
            nnnnnnn";
        let bits_1 = BitBoard::from_string(board_1).unwrap();
        let bits_2 = BitBoard::from_string(board_2).unwrap();


        assert!(bits_1.has_won());
        assert!(bits_2.has_won());
    }

    #[test]
    fn test_winning_move_up_left() {
        let board_1 =
            "nnnnnnn
            nnnnnnn
            nnnnnnn
            nnnpnnn
            nnnnpnn
            nnnnnpn";
        let board_2 =
            "nnnnnnn
            npnnnpn
            nnnnnnn
            nnnpnnn
            nnnnpnn
            nnnnnnn";
        let bits_1 = BitBoard::from_string(board_1).unwrap();
        let bits_2 = BitBoard::from_string(board_2).unwrap();

        let player_move =
            "nnnnnnn
            nnnnnnn
            nnpnnnn
            nnnnnnn
            nnnnnnn
            nnnnnnn";
        let position = BitBoard::from_string(player_move).unwrap().player;

        assert!(bits_1.is_winning_move(position));
        assert!(bits_2.is_winning_move(position));
    }

    #[test]
    fn test_winning_board_up_left() {
        let board_1 =
            "nnnnnnn
            nnnnnnn
            nnpnnnn
            nnnpnnn
            nnnnpnn
            nnnnnpn";
        let board_2 =
            "pnnnnnn
            npnnnpn
            nnpnnnn
            nnnpnnn
            nnnnnnn
            nnnnnnn";
        let bits_1 = BitBoard::from_string(board_1).unwrap();
        let bits_2 = BitBoard::from_string(board_2).unwrap();


        assert!(bits_1.has_won());
        assert!(bits_2.has_won());
    }

    #[test]
    fn test_all_possible_moves() {
        let board_2 =
            "nnnnnnn
            nnnnnnn
            nnnnnnn
            nnnpnnn
            nnnppnn
            nnnpppn";
        let expected_2 =
            "nnnnnnn
            nnnnnnn
            nnnpnnn
            nnnnpnn
            nnnnnpn
            pppnnnp";

        let board_1 = BitBoard::default();
        let board_2 = BitBoard::from_string(board_2).unwrap();

        let possible = board_1.all_possible_moves();
        assert_eq!(possible, BitBoard::BOTTOM_LINE & BitBoard::PLAYABLE_FIELDS);

        let possible = board_2.all_possible_moves();
        let expected = BitBoard::from_string(expected_2).unwrap().player;
        assert_eq!(possible, expected);
    }
}