mod utils;
pub mod board;
mod engine;
mod transposition;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern {
    fn alert(s: &str);
}