use leptos::*;
use wasm_bindgen::prelude::*;
use js_sys::Array;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub fn js_log(s: &str) {
    log(s);
}

#[wasm_bindgen]
pub fn random_uuid() -> String {
    // Generate a simple UUID-like string in JS
    "xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx".replace('x', |_| -> String {
        format!("{:x}", fastrand::u32(0..15))
    }).replace('y', |_| -> String {
        format!("{:x}", (fastrand::u32(0..3) + 8))
    })
}
