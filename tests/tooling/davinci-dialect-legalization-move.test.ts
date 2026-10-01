import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const sourceRoot = "davinci/vize_l1_to_l2/src";
const script = path.join(repoRoot, "tools/support/levels/move-dialect-legalization.ts");
const helpers = [
  "filter.rs",
  "filter/compound.rs",
  "ids.rs",
  "on.rs",
  "slot.rs",
  "sync.rs",
  "tree.rs",
];
const moves: Array<readonly [string, string]> = [
  ["pass/legacy.rs", "dialect/legalize.rs"],
  ...helpers.map((file) => [`pass/legacy/${file}`, `dialect/legalize/${file}`] as const),
];

function write(root: string, file: string, content: string): void {
  const destination = path.join(root, sourceRoot, file);
  fs.mkdirSync(path.dirname(destination), { recursive: true });
  fs.writeFileSync(destination, content);
}

function fixture(): string {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-dialect-legalization-"));
  for (const [old, current] of moves) {
    write(root, old, fs.readFileSync(path.join(repoRoot, sourceRoot, current), "utf8"));
  }
  return root;
}

function run(root: string, mode: string): ReturnType<typeof spawnSync> {
  return spawnSync(process.execPath, [script, mode, "--root", root], { encoding: "utf8" });
}

test("legalization moves preserve every owner byte and replay after wiring", () => {
  const root = fixture();
  try {
    const result = run(root, "move");
    assert.equal(result.status, 0, String(result.stderr));
    for (const [old, current] of moves) {
      assert.equal(fs.existsSync(path.join(root, sourceRoot, old)), false);
      assert.deepEqual(
        fs.readFileSync(path.join(root, sourceRoot, current)),
        fs.readFileSync(path.join(repoRoot, sourceRoot, current)),
      );
    }
    assert.equal(run(root, "move").status, 0);
    write(root, "pass/legacy.rs", "pub use crate::dialect::legalize::*;\n");
    assert.equal(run(root, "move").status, 0);
    assert.equal(run(repoRoot, "check").status, 0);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

test("unreviewed helpers and missing inputs are rejected before mutation", () => {
  const root = fixture();
  try {
    write(root, "pass/legacy/unknown.rs", "fn extra() {}\n");
    assert.notEqual(run(root, "move").status, 0);
    assert.equal(fs.existsSync(path.join(root, sourceRoot, "pass/legacy.rs")), true);
    fs.rmSync(path.join(root, sourceRoot, "pass/legacy/unknown.rs"));
    fs.rmSync(path.join(root, sourceRoot, "pass/legacy/tree.rs"));
    assert.notEqual(run(root, "move").status, 0);
    assert.equal(fs.existsSync(path.join(root, sourceRoot, "pass/legacy.rs")), true);
    assert.equal(fs.existsSync(path.join(root, sourceRoot, "dialect/legalize.rs")), false);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});
