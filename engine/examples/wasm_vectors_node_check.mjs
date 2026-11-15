// T223: run the shared PK test vectors inside a real wasm32 module (the
// engine compiled for wasm32-unknown-unknown) via node's WebAssembly API.
// Passes only if wasm f64 math reproduces the Python-computed fixture that
// the native tests also assert (spec-003 FR-038, SC-033).
//
// Usage (repo root):
//   cargo build --target wasm32-unknown-unknown -p engine --bin wasm_vectors --no-default-features --offline
//   node engine/examples/wasm_vectors_node_check.mjs [path/to/wasm_vectors.wasm]
import { readFileSync } from "node:fs";

const wasmPath =
  process.argv[2] ??
  "target/wasm32-unknown-unknown/debug/wasm_vectors.wasm";

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
const { memory } = instance.exports;

function readReport(runName, ptrName, lenName) {
  instance.exports[runName]();
  const ptr = Number(instance.exports[ptrName]());
  const len = Number(instance.exports[lenName]());
  return new TextDecoder().decode(new Uint8Array(memory.buffer, ptr, len));
}

// T223: raw curve maths. T250: panel data derivation (panel_view path).
const reports = [
  ["wasm PK vectors", readReport("pk_vectors_run", "pk_vectors_report_ptr", "pk_vectors_report_len")],
  ["wasm panel vectors", readReport("panel_vectors_run", "panel_vectors_report_ptr", "panel_vectors_report_len")],
];

let failed = false;
for (const [label, report] of reports) {
  if (report === "OK") {
    console.log(label + ": OK (all checks match native fixture)");
  } else {
    console.error(label + ": DRIFT vs native fixture:\n" + report);
    failed = true;
  }
}
process.exit(failed ? 1 : 0);
