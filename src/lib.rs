#![feature(crate_visibility_modifier)]

mod utils;
pub mod logic;
pub mod BitBoard;
mod bit_solver;

use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
// #[cfg(feature = "wee_alloc")]
// #[global_allocator]
// static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet(name: &str) {
    let t = (format!("Hello {}! Connect-four-ai!", name) );
    alert(t.as_str());
}
