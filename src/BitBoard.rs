use crate::logic;
use crate::logic::{BOARD_WIDTH, BOARD_HEIGHT};

/// Encoding: Column-first-order, 8-bit per column.
/// That way 2 bits per column (they are 6 fields high) are left unused.
#[derive(Default)]
pub struct BitBoard {
    pub player: u64,
    pub occupied: u64,
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

    #[inline]
    pub fn number_of_stones(&self) -> u32 {
        self.occupied.count_ones()
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

}

#[cfg(test)]
mod tests {
    use crate::BitBoard::BitBoard;
    use crate::logic::GameBoard;

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