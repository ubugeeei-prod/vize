/**
 * Builds the Vitrine WebAssembly bindings once and emits them for the web.
 *
 * Usage: node tools/support/github/build-vitrine-wasm.ts <out-dir> [out-dir...]
 *
 * Cargo inherits the job environment. When the job sets no `RUSTFLAGS`, Cargo
 * receives the `wasm_js` getrandom backend flag; an inherited `RUSTFLAGS` keeps
 * precedence, as it did for the previous MoonBit command. `wasm-bindgen` from
 * `PATH` then writes each output directory in order and stops at the first
 * failure.
 */
import {
  addMissingVariables,
  commandArguments,
  isEntrypoint,
  printLine,
  runCommand,
  runEntrypoint,
} from "../command-runtime.ts";
import { wasmRustflags } from "../tasks/build-vitrine-wasm.ts";

const usage = "Usage: node tools/support/github/build-vitrine-wasm.ts <out-dir> [out-dir...]";

export function main(outDirs: readonly string[]): number {
  if (outDirs.length === 0) {
    printLine(usage);
    return 1;
  }
  const cargoExit = runCommand(
    "cargo",
    [
      "build",
      "--release",
      "-p",
      "vize_vitrine",
      "--no-default-features",
      "--features",
      "wasm",
      "--target",
      "wasm32-unknown-unknown",
    ],
    { env: addMissingVariables({ RUSTFLAGS: wasmRustflags() }) },
  );
  if (cargoExit !== 0) return cargoExit;
  for (const outDir of outDirs) {
    const bindgenExit = runCommand("wasm-bindgen", [
      "target/wasm32-unknown-unknown/release/vize_vitrine.wasm",
      "--out-dir",
      outDir,
      "--target",
      "web",
    ]);
    if (bindgenExit !== 0) return bindgenExit;
  }
  return 0;
}

if (isEntrypoint(import.meta.url)) runEntrypoint(() => main(commandArguments()));
