import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

test("expression regression corpus retains the complete issue 7808 input after regeneration", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-expression-seeds-"));
  try {
    fs.writeFileSync(path.join(root, "Cargo.toml"), "[workspace]\n");
    fs.writeFileSync(path.join(root, "pnpm-workspace.yaml"), "packages: []\n");
    const relative = "tests/fuzz/regressions/js_ts_expression/issue-7808.input";
    const bytes = fs.readFileSync(path.join(repoRoot, relative));
    assert.equal(bytes.length, 22_933);
    assert.equal(
      createHash("sha256").update(bytes).digest("hex"),
      "8670c91bace6df50473b3ad261b306446fb59b345d8d644a9771aa5aa4a0e5cb",
    );
    fs.mkdirSync(path.dirname(path.join(root, relative)), { recursive: true });
    fs.writeFileSync(path.join(root, relative), bytes);
    const corpus = path.join(root, "tests/fuzz/corpus/js_ts_expression");
    const digest = createHash("sha1").update(bytes).digest("hex").slice(0, 16);
    for (let pass = 0; pass < 2; pass++) {
      fs.mkdirSync(corpus, { recursive: true });
      fs.writeFileSync(path.join(corpus, "stale-seed"), "discard me");
      const result = spawnSync(
        "rust-script",
        [path.join(repoRoot, "tools/commands/ci/fuzz/seed_corpus.rs")],
        { encoding: "utf8", env: { ...process.env, VIZE_REPO_ROOT: root } },
      );
      assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);
      assert.match(result.stdout, /1 JS\/TS expression entries/);
      assert.deepEqual(fs.readdirSync(corpus), [digest]);
      assert.deepEqual(fs.readFileSync(path.join(corpus, digest)), bytes);
    }
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});
