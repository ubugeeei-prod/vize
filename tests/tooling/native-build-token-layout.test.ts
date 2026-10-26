import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { randomBytes } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

test("the optimized production getter retains its contiguous invocation token", (t) => {
  const root = fileURLToPath(new URL("../../", import.meta.url));
  const source = fs.readFileSync(
    path.join(root, "crates/vize_vitrine/src/napi/build_token.rs"),
    "utf8",
  );
  const start = source.indexOf("pub fn get_native_build_token()");
  assert.notEqual(start, -1);
  // Compile the actual function body; only the N-API wrapper is outside this
  // small code-generation regression. The dedicated target lane loads N-API.
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "native-token-layout-"));
  t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
  const input = path.join(directory, "getter.rs");
  const image = path.join(directory, "getter-image");
  fs.writeFileSync(input, `#[unsafe(no_mangle)]\n${source.slice(start)}`);
  const token = randomBytes(16).toString("hex");
  const result = spawnSync(
    "rustc",
    [
      "--edition=2024",
      "--crate-type=cdylib",
      "-Copt-level=3",
      "-Clto=fat",
      "-Ccodegen-units=1",
      input,
      "-o",
      image,
    ],
    { env: { ...process.env, VIZE_NATIVE_BUILD_TOKEN: token }, encoding: "utf8", timeout: 30_000 },
  );
  assert.equal(result.error, undefined);
  assert.equal(result.signal, null);
  assert.equal(result.status, 0, result.stdout + result.stderr);
  assert.ok(fs.readFileSync(image).includes(Buffer.from(token)), "whole fresh token is absent");
});
