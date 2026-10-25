import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

void test("the actual Rust VSIX configuration validator keeps typed defaults and refusal controls", (t) => {
  const target = fs.mkdtempSync(path.join(os.tmpdir(), "vize-vsix-validation-"));
  t.after(() => fs.rmSync(target, { force: true, recursive: true }));
  const result = spawnSync(
    "cargo",
    [
      "test",
      "--manifest-path",
      path.join(root, "tools/support/Cargo.toml"),
      "vsix_configuration::tests",
      "--",
      "--nocapture",
    ],
    {
      cwd: root,
      encoding: "utf8",
      timeout: 120_000,
      env: { ...process.env, CARGO_TARGET_DIR: target },
    },
  );
  assert.equal(result.status, 0, result.stderr + result.stdout);
  assert.match(result.stdout, /6 passed; 0 failed/);
});
