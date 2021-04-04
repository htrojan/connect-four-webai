//! Hash methods and HashTable for a transposition table
//!

use rand::prelude::*;
use crate::board::{BitBoard, FieldType};

/// Implementation of the ['Zobrist'] hash function
///
/// ['Zobrist']: https://www.chessprogramming.org/Zobrist_Hashing
pub struct ZobristHasher {
    table_p1: [u64; 42],
    table_p2: [u64; 42]
}

impl ZobristHasher {
    pub fn new() -> ZobristHasher {
        let mut table_p1 = [0; 42];
        let mut table_p2 = [0; 42];

        let mut rnd = rand::rngs::StdRng::seed_from_u64(42);

        for i in 0..42 {
            table_p1[i] = rnd.next_u64();
            table_p2[i] = rnd.next_u64();
        }

        ZobristHasher { table_p1, table_p2 }
    }

    pub fn hash_board(&self, board: BitBoard) {
        let player1 = board.player;
        let player2 = board.occupied - board.player;

        let mut hash = 0;

        for i in 0..42 {
            // Convert index to actual bit position
            let row = i % 6;
            let column = i / 6;
            let bit_pos = column*8 + row;

            if player1 & (1<<bit_pos) > 0 {
                hash ^= self.table_p1[bit_pos];
            }else if player2 & (1<<bit_pos) > 0 {
                hash ^= self.table_p2[bit_pos];
            }

        }
    }

    /// Updates the given zobrist hash and applies the move
    /// by xoring the hash with the corresponding random value corresponding
    /// to that position and player
    #[inline]
    pub fn update_hash(&self, hash: u64, mov: u64, player: FieldType) -> u64 {
        let leading_zeros = mov.leading_zeros() as usize;
        //Todo: Is it faster to use arrays of length 8*7=56 in the zobrist tables
        // to circumvent the padding correction in the update_hash() function?
        // Con: 16 u64s of unnecessary space used
        // Pro: Hot codepath has one division, one multiplication, one subtraction less

        // Subtract the padding of the u64 board representation
        // (two additional bits per column)
        let column_count = leading_zeros/6;
        let index: usize = leading_zeros - column_count*2;
        return match player {
            FieldType::Opponent => {hash ^ self.table_p2[index]}
            FieldType::Player => {hash ^ self.table_p1[index]}
        }
    }
}

pub struct HashMap {
   num_buckets: u32;
}