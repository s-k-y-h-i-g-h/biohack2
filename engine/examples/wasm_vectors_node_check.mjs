// T223: run the shared PK test vectors inside a real wasm32 module (the
// engine compiled for wasm32-unknown-unknown) via node's WebAssembly API.
// Passes only if wasm f64 math reproduces the Python-computed fixture that
// the native tests also assert (spec-003 FR-038, SC-033).
//
// Usage (repo root):
//   cargo build --target wasm32-unknown-unknown -p engine --example wasm_vectors --offline
//   node engine/examples/wasm_vectors_node_check.mjs [path/to/wasm_vectors.wasm]
import { readFileSync } from "node:fs";

const wasmPath =
  process.argv[2] ??
  "target/wasm32-unknown-unknown/debug/examples/wasm_vectors.wasm";

const bytes = readFileSync(wasmPath);

// The wasm keeps four wasm-bindgen glue imports (throw/describe/externref
// table) even though the vector path never calls them; provide inert stubs
// that surface a loud error if anything ever does invoke one.
const externrefs = [];
const imports = {
  __wbindgen_placeholder__: {
    __wbindgen_describe: () => {
      throw new Error("unexpected __wbindgen_describe call");
    },
    __wbg___wbindgen_throw_bb96b2010945f0bc: (ptr, len) => {
      const mem = new Uint8Array(instance.exports.memory.buffer, ptr, len);
      throw new Error("wasm-bindgen throw: " + new TextDecoder().decode(mem));
    },
  },
  __wbindgen_externref_xform__: {
    __wbindgen_externref_table_set_null: (idx) => {
      externrefs[idx] = null;
    },
    __wbindgen_externref_table_grow: (delta) => {
      const old = externrefs.length;
      externrefs.length = old + delta;
      return old;
    },
  },
};
const { instance } = await WebAssembly.instantiate(bytes, imports);
const { memory, pk_vectors_run, pk_vectors_report_ptr, pk_vectors_report_len } =
  instance.exports;

pk_vectors_run();
const ptr = Number(pk_vectors_report_ptr());
const len = Number(pk_vectors_report_len());
const report = new TextDecoder().decode(new Uint8Array(memory.buffer, ptr, len));

if (report === "OK") {
  console.log("wasm PK vectors: OK (all checks match native fixture)");
  process.exit(0);
} else {
  console.error("wasm PK vectors: DRIFT vs native fixture:\n" + report);
  process.exit(1);
}
