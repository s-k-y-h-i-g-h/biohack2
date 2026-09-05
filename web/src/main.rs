use wasm_bindgen::prelude::*;

// Re-export the main function from lib.rs
#[wasm_bindgen(start)]
pub fn main() {
    biohack2_web::main();
}
