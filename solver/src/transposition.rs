//! Hash methods and HashTable for a transposition table
//!

use std::alloc::{Allocator, Global, Layout};
use std::marker::PhantomData;
use std::ptr::NonNull;

use rand::prelude::*;

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
            let bit_pos = column * 8 + row;

            if player1 & (1 << bit_pos) > 0 {
                hash ^= self.table_p1[bit_pos];
            } else if player2 & (1 << bit_pos) > 0 {
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
        let column_count = leading_zeros / 6;
        let index: usize = leading_zeros - column_count * 2;
        return match player {
            FieldType::Opponent => hash ^ self.table_p2[index],
            FieldType::Player => hash ^ self.table_p1[index],
        };
    }
}

/// An SwissTable/Hashbrown inspired rudimentary hashmap implementation.
/// Most code is taken and heavily broken down and adapted from ['rust-lang/hashbrown']
/// This implementation has a constant size and employs a replacement strategy
/// when full or when above a certain load factor(?).
/// Additionally, only BitBoards can be stored. No generic interface is provided.
/// todo: Replacement strategy description
/// Additional resources:
/// - ['The Swiss Army Knife of Hashmaps']
/// - ['rust-lang/hashbrown']
///
/// ['The Swiss Army Knife of Hashmaps']: https://blog.waffles.space/2018/12/07/deep-dive-into-hashbrown/
/// ['rust-lang/hashbrown']: https://github.com/rust-lang/hashbrown
pub struct TMap<A: Allocator> {
    raw_table: RawTable<A>,
}

impl TMap<Global> {
    fn new(buckets: usize) -> Result<Self, AllocationError> {
        let table = match unsafe { RawTable::new_in(Global, buckets) } {
            Ok(table) => table,
            Err(_) => {
                return Err(AllocationError {});
            }
        };
        Ok(Self { raw_table: table })
    }
}

/// Internally used to store the BitBoard together with additional information used
/// for the replacement strategy.
struct TEntry {
    /// The actual data board to be stored in the hashmap.
    board: BitBoard,
    /// The associated score
    score: i32,
    /// Used to differentiate 'hot' from 'cold' entries and replace the cold ones if memory is needed.
    depth: u8,
}

/// Hides unsafe code from the public TMap interface
struct RawTable<A: Allocator> {
    /// Number of buckets. This is a multiple of 16 (multiple of group size)
    buckets: usize,

    /// Pointer to the control struct holding the array of 8-bit control entries
    ctrl: NonNull<u8>,

    /// Number of elements in the table
    items: usize,

    /// Tells the drop checker that this table owns the BitBoard entries
    marker: PhantomData<BitBoard>,

    alloc: A,
}

struct AllocationError {}

impl<A: Allocator + Clone> RawTable<A> {
    pub unsafe fn new_in(alloc: A, buckets: usize) -> Result<Self, AllocationError> {
        // Make buckets power of two
        let buckets = buckets.next_power_of_two();

        // Calculate layout
        let (layout, ctrl_offset) = TableLayout::calculate_layout(buckets);
        println!("Table layout: {:?}", layout);
        println!("Ctrl offset: {:?}", ctrl_offset);

        let ptr: NonNull<u8> = match do_alloc(&alloc, layout) {
            Ok(block) => block.cast(),
            Err(_) => {
                return Err(AllocationError {});
            }
        };

        Ok(Self {
            buckets,
            ctrl: NonNull::new_unchecked(ptr.as_ptr().add(ctrl_offset)),
            items: 0,
            marker: PhantomData,
            alloc,
        })
    }

    pub fn insert(&self, entry: TEntry, hash: u64) {
        let bucket = h1(hash) % self.buckets;

        let index = self.probe_for(hash);
    }

    /// Probes the control sequence for the specified hash
    fn probe_for(&self, hash: u64) -> usize {
        //todo: can be made faster using bitmasks and power of two bucket size
        let bucket = h1(hash) % self.buckets;
        todo!("Not implemented")
    }
}

/// Returns the h1 hash (for 32 bit platforms)
#[inline]
fn h1(hash: u64) -> usize{
    hash as usize
}

/// Returns the top 7 bits of the has to be saved in the low 7 bits of the control byte
#[inline]
fn h2(hash: u64) -> u8 {
    // Shift the bits so that the top 7 bits are the only bits left
    let bit_shift = 64 - 7;
    let h2 = hash >> bit_shift;
    h2 as u8
}

#[allow(clippy::map_err_ignore)]
pub fn do_alloc<A: Allocator>(alloc: &A, layout: Layout) -> Result<NonNull<u8>, ()> {
    alloc
        .allocate(layout)
        .map(|ptr| ptr.as_non_null_ptr())
        .map_err(|_| ())
}

/// Struct representing a set of bytes from the control structure.
/// This is done without the SIMD feature (not sure if this code would benefit due to
/// latency reasons of the sse instructions)
/// TODO: Test in the future
struct Group {}

impl Group {
    const WIDTH: usize = 8;

    /// Loads Group::WIDTH control bytes from memory. The offset given has to be aligned
    /// to the group size.
    fn load(ptr: *mut u8) {

    }
}

struct TableLayout {}

impl TableLayout {
    /// Returns the layout needed to create the buckets followed by the control
    /// structure.
    /// The offset is given in bytes
    fn calculate_layout(buckets: usize) -> (Layout, usize) {
        assert_eq!(buckets % Group::WIDTH, 0);

        let layout_entries = Layout::new::<TEntry>().repeat(buckets).unwrap();
        let layout_ctrl = Layout::new::<u8>().repeat(buckets).unwrap();
        println!("Entry layout: {:?}", Layout::new::<TEntry>());
        layout_entries.0.extend(layout_ctrl.0).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use crate::transposition::TMap;

    #[test]
    fn test_table_init() {
        let table = TMap::new(10e5 as usize);
    }
}
