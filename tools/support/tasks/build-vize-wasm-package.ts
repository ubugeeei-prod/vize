/**
 * Builds the WebAssembly package used for npm publication.
 *
 * Usage: node tools/support/tasks/build-vize-wasm-package.ts
 *
 * The release workflow treats `npm/wasm` as a generated artifact. This command
 * builds the Rust crate for `wasm32-unknown-unknown`, runs `wasm-bindgen`, and
 * then restores the checked-in `package.json` from `HEAD` so build tools do not
 * overwrite package metadata that must stay version-controlled.
 */
import fs from "node:fs";

import {
  type CommandEnvironment,
  collectStdout,
  isEntrypoint,
  printLine,
  runCommand,
  runEntrypoint,
} from "../command-runtime.ts";
import { wasmRustflags } from "./build-vitrine-wasm.ts";

/** Returns the complete Cargo environment for the browser WASM release build. */
export function wasmPackageCargoEnvironment(
  env: CommandEnvironment = process.env,
): CommandEnvironment {
  return { ...env, RUSTFLAGS: wasmRustflags(env), CARGO_PROFILE_RELEASE_PANIC: "abort" };
}

export function main(): number {
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
    { env: wasmPackageCargoEnvironment() },
  );
  if (cargoExit !== 0) {
    printLine("Failed to build the WASM crate");
    return 1;
  }

  const bindgenExit = runCommand("wasm-bindgen", [
    "target/wasm32-unknown-unknown/release/vize_vitrine.wasm",
    "--out-dir",
    "npm/wasm",
    "--target",
    "web",
  ]);
  if (bindgenExit !== 0) {
    printLine("Failed to run wasm-bindgen");
    return 1;
  }

  const restore = collectStdout("git", ["show", "HEAD:npm/wasm/package.json"]);
  if (restore.exitCode !== 0) {
    printLine("Failed to restore npm/wasm/package.json from HEAD");
    return 1;
  }
  fs.writeFileSync("npm/wasm/package.json", restore.stdout);
  return 0;
}

if (isEntrypoint(import.meta.url)) runEntrypoint(main);
