//! T223 runner shim: exposes spec-003's shared PK test-vector check as a
//! bare wasm32 export with no wasm-bindgen glue, so `node` can execute the
//! *same* fixture the native tests consume via the plain WebAssembly API —
//! no browser or chromedriver needed (FR-038 native↔wasm32 parity).
//!
//! Built with:
//!   cargo build --target wasm32-unknown-unknown --example wasm_vectors --offline
//! Executed with (from the repo root):
//!   node engine/examples/wasm_vectors_node_check.mjs

use engine::pharmacokinetics::check_vectors;

const FIXTURE: &str = include_str!("../tests/fixtures/pk_vectors.json");

static mut REPORT_PTR: usize = 0;
static mut REPORT_LEN: usize = 0;

/// Runs the vector check and leaks the report into linear memory.
/// Exported symbol names are the wasm ABI; the `static mut` pattern is the
/// minimal way to hand bytes back without any glue crate.
#[unsafe(no_mangle)]
pub extern "C" fn pk_vectors_run() {
    let report = match check_vectors(FIXTURE) {
        f if f.is_empty() => "OK".to_string(),
        f => f.join("\n"),
    };
    let bytes = report.into_bytes();
    unsafe {
        REPORT_PTR = bytes.as_ptr() as usize;
        REPORT_LEN = bytes.len();
        std::mem::forget(bytes);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn pk_vectors_report_ptr() -> usize {
    unsafe { REPORT_PTR }
}

#[unsafe(no_mangle)]
pub extern "C" fn pk_vectors_report_len() -> usize {
    unsafe { REPORT_LEN }
}

fn main() {
    // Bin entry exists only to satisfy cargo; the node runner calls the
    // exports directly.
    pk_vectors_run();
}
