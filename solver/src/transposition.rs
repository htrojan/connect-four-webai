//! Hash methods and HashTable for a transposition table
//!

use rand::prelude::*;
use std::collections::VecDeque;

use crate::board::{BitBoard, FieldType};

/// Implementation of the ['Zobrist'] hash function
///
/// ['Zobrist']: https://www.chessprogramming.org/Zobrist_Hashing
pub struct ZobristHasher {
    table_p1: [u64; 42],
    table_p2: [u64; 42],
}

impl ZobristHasher {
    pub fn new() -> ZobristHasher {
        let mut table_p1 = [0; 42];
        let mut table_p2 = [0; 42];

        let mut rnd = StdRng::seed_from_u64(42);

        for i in 0..42 {
            table_p1[i] = rnd.next_u64();
            table_p2[i] = rnd.next_u64();
        }

        ZobristHasher { table_p1, table_p2 }
    }

    pub fn hash_board(&self, board: &BitBoard) {
        let mut hash = 0;

        for field in board.field_iter() {
            if board.is_occupied_by_player_at_field(&field) {
                hash ^= self.table_p1[field.get_index() as usize];
            } else if board.is_occupied_at_field(&field) {
                // Occupied by player2
                hash ^= self.table_p2[field.get_index() as usize];
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
        let column_count = leading_zeros / 6;
        let index: usize = leading_zeros - column_count * 2;
        return match player {
            FieldType::Opponent => hash ^ self.table_p2[index],
            FieldType::Player => hash ^ self.table_p1[index],
        };
    }
}

/// A transposition table for the solver
/// The table is implemented as a hash map with linear probing.
/// It stores the hashes in a vector and the corresponding scores in another vector, where
/// the indices of both vectors correspond to each other.
/// Therefore, if a hash is found in the hash vector, the corresponding score can be found
/// in the score vector at the same index.
struct TranspositionTable<T: Sized + Default + Clone> {
    ///Reserved size in number of entries. Does not correspond to the
    /// Number of stored entries, as the load factor will be kept below max_load_factor
    max_size: usize,

    /// Maximum load factor. The load factor is actual entries / size.
    /// Kept low to decrease chance of hash collisions
    max_load_factor: f32,

    /// Current number of entries in the table
    num_entries: usize,

    /// Maximum depth currently stored in the table
    max_encountered_depth: u8,

    /// Physical entries
    entries: Vec<T>,

    /// Vector of hashes. Used to resolve hash collisions
    hashes: Vec<Option<u64>>,

    /// Stores depth information about a position. 0 means, it is unitialized
    depths: Vec<u8>,
}

impl<T: Sized + Default + Clone> TranspositionTable<T> {
    pub fn new(max_size: usize, max_load_factor: f32) -> TranspositionTable<T> {
        TranspositionTable {
            max_size,
            max_load_factor,
            num_entries: 0,
            entries: vec![T::default(); max_size],
            hashes: vec![None; max_size],
            depths: vec![0; max_size],
            max_encountered_depth: 0,
        }
    }

    pub fn get_load_factor(&self) -> f32 {
        self.num_entries as f32 / self.max_size as f32
    }

    /// Removes x entries from the table
    fn replacement_strategy() {
        todo!("Implement replacement strategy for old entries");
    }

    /// Inserts a new entry into the table
    /// If the table is full, the oldest entry is overwritten
    pub fn insert(&mut self, hash: u64, score: T, depth: u8) {
        if self.get_load_factor() > self.max_load_factor {
            Self::replacement_strategy();
        }

        // Search for the correct position
        let mut pos = hash % self.max_size as u64;

        // If entry collides, search next free entry
        while (self.hashes[pos as usize].is_some()) {
            // Prevent overflow. todo: Maybe introduce a size check in loop and remove modulo
            pos = (pos + 1) % self.max_size as u64;
        }

        self.entries[pos as usize] = score;
        self.hashes[pos as usize] = Some(hash);
        self.depths[pos as usize] = depth;
        self.num_entries += 1;
    }

    /// Returns the score for the given hash if it exists in the table
    pub fn get_mut(&mut self, hash: u64) -> Option<&mut T> {
        let bucket = (hash % self.max_size as u64);
        let mut pos = bucket;

        while let Some(current_hash) = self.hashes[pos as usize] {
            if current_hash == hash {
                return Some(&mut self.entries[pos as usize]);
            }

            // Detect a bucket change. The entry was not in this bucket
            if (current_hash % self.max_size as u64) != bucket {
                return None;
            }
            pos = (pos + 1) % self.max_size as u64;
        }

        None
    }

    /// Returns the load factor of the table
    pub fn load_factor(&self) -> f32 {
        self.num_entries as f32 / self.max_size as f32
    }

    pub fn is_full(&self) -> bool {
        self.load_factor() >= self.max_load_factor
    }

    pub fn is_empty(&self) -> bool {
        self.num_entries == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_insert() {
        let mut map = TranspositionTable::new(10, 0.8);
        map.insert(1, 1, 0);
        map.insert(11, 2, 0);
        assert_eq!(map.get_mut(1), Some(&mut 1));
        assert_eq!(map.get_mut(11), Some(&mut 2));
    }
}
