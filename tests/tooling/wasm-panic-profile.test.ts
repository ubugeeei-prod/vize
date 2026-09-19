import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { runMoonScript } from "./_helpers/moonbit.ts";
import { writeFakeCommand } from "./support/fake-command.ts";

for (const [script, args, status] of [
  ["build_vize_wasm_package", [], 1],
  ["build_vitrine_wasm", ["web", "out"], 17],
] as const) {
  test(`${script} keeps browser WASM on abort while native releases unwind`, () => {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-wasm-panic-"));
    const bin = path.join(root, "bin");
    const record = path.join(root, "cargo.json");
    const bindgen = path.join(root, ".cache/wasm-pack/.wasm-bindgen-cargo-install-0.2.121/bin");
    try {
      fs.mkdirSync(bin, { recursive: true });
      fs.mkdirSync(bindgen, { recursive: true });
      writeFakeCommand(bindgen, "wasm-bindgen", "console.log('wasm-bindgen 0.2.121');");
      // Exercise hosts without Apple's SDK discovery command, including macOS
      // installations where the command exists but cannot locate an SDK.
      writeFakeCommand(bin, "xcrun", "process.exit(1);");
      writeFakeCommand(
        bin,
        "cargo",
        `require('node:fs').writeFileSync(${JSON.stringify(record)}, JSON.stringify({
          panic: process.env.CARGO_PROFILE_RELEASE_PANIC,
          flags: process.env.RUSTFLAGS,
          args: process.argv.slice(2),
        })); process.exit(17);`,
      );
      const result = runMoonScript(script, [...args], {
        cwd: root,
        env: {
          PATH: `${bin}${path.delimiter}${process.env.PATH ?? ""}`,
          CARGO_PROFILE_RELEASE_PANIC: "unwind",
          RUSTFLAGS: "--cfg vize_profile_probe",
        },
      });
      assert.equal(result.status, status, `${result.stdout}\n${result.stderr}`);
      const recorded = JSON.parse(fs.readFileSync(record, "utf8"));
      assert.equal(recorded.panic, "abort");
      assert.equal(recorded.flags, '--cfg vize_profile_probe --cfg getrandom_backend="wasm_js"');
      assert.deepEqual(recorded.args.slice(-2), ["--target", "wasm32-unknown-unknown"]);
    } finally {
      fs.rmSync(root, { recursive: true, force: true });
    }
  });
}
