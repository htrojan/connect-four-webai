#![feature(alloc_layout_extra )]
#![feature(allocator_api)]
#![feature(slice_ptr_get)]
#![feature(const_fn)]

mod utils;
pub mod board;
pub mod engine;
mod transposition;

use wasm_bindgen::prelude::*;