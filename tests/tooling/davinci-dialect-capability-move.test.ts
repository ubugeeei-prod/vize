import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const script = path.join(repoRoot, "tools/support/levels/move-dialect-capabilities.ts");
const paths = [
  ["crates/vize_armature/src/legacy.rs", "davinci/vize_l1/src/dialect/vue.rs"],
  ["davinci/vize_l1_to_l2/src/lower/caps.rs", "davinci/vize_l1/src/dialect/template.rs"],
] as const;

function fixture(): string {
  return fs.mkdtempSync(path.join(os.tmpdir(), "vize-dialect-move-"));
}

function write(root: string, file: string, content: string): void {
  const destination = path.join(root, file);
  fs.mkdirSync(path.dirname(destination), { recursive: true });
  fs.writeFileSync(destination, content);
}

function run(root: string, mode: string): ReturnType<typeof spawnSync> {
  return spawnSync(process.execPath, [script, mode, "--root", root], { encoding: "utf8" });
}

test("capability moves retain exact owner bytes and replay idempotently", () => {
  const root = fixture();
  try {
    for (const [old, current] of paths)
      write(root, old, fs.readFileSync(path.join(repoRoot, current), "utf8"));
    const result = run(root, "move");
    assert.equal(result.status, 0, String(result.stderr));
    for (const [old, current] of paths) {
      assert.equal(fs.existsSync(path.join(root, old)), false);
      assert.deepEqual(
        fs.readFileSync(path.join(root, current)),
        fs.readFileSync(path.join(repoRoot, current)),
      );
    }
    assert.equal(run(root, "move").status, 0);
    for (const [old] of paths) write(root, old, "pub use shared::LegacyCaps;\n");
    assert.equal(run(root, "move").status, 0);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

test("conflicting or missing owners fail before mutating any file", () => {
  const root = fixture();
  try {
    write(root, paths[0][0], "pub struct LegacyDialectCapabilities;\n");
    let result = run(root, "move");
    assert.notEqual(result.status, 0);
    assert.equal(fs.existsSync(path.join(root, paths[0][0])), true);
    assert.equal(fs.existsSync(path.join(root, paths[0][1])), false);
    write(root, paths[1][0], "pub struct LegacyCaps;\n");
    write(root, paths[1][1], "pub struct LegacyCaps;\n");
    result = run(root, "move");
    assert.notEqual(result.status, 0);
    assert.equal(fs.existsSync(path.join(root, paths[0][0])), true);
    assert.equal(fs.existsSync(path.join(root, paths[0][1])), false);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

test("capability-owner check rejects missing wiring and accepts the integrated repository", () => {
  const root = fixture();
  try {
    assert.notEqual(run(root, "check").status, 0);
    const result = run(repoRoot, "check");
    assert.equal(result.status, 0, String(result.stderr));
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});
