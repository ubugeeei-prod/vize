// The assertion lint scans out-of-line test modules (files reached through a
// `cfg(test)` mod declaration) structurally and holds their pre-existing
// partial assertions in a generated per-file count ratchet.

import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const lintPath = path.join(repoRoot, "tools", "commands", "davinci", "assertion-lint.rs");
const partial = (name: string) =>
  `    #[test]\n    fn ${name}() {\n        assert!("ab".contains("a"));\n    }\n`;

function tree(files: Record<string, string>): string {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-assertion-ratchet-"));
  for (const [file, text] of Object.entries(files)) {
    fs.mkdirSync(path.dirname(path.join(root, file)), { recursive: true });
    fs.writeFileSync(path.join(root, file), text);
  }
  return root;
}

function lint(root: string, ...args: string[]) {
  return spawnSync(
    "rust-script",
    [lintPath, "--root", root, "--ratchet", path.join(root, "ratchet.tsv"), ...args],
    { cwd: repoRoot, encoding: "utf8" },
  );
}

test("sibling, directory and nested out-of-line test modules are scanned", () => {
  const root = tree({
    "src/lib.rs":
      '#[cfg(test)]\n#[path = "parser_tests.rs"]\nmod tests;\n#[cfg(all(test, unix))]\nmod nested;\nmod live;\n',
    "src/parser_tests.rs": `mod more;\n${partial("sibling")}`,
    "src/more.rs": partial("declared_by_a_test_file"),
    "src/nested.rs": partial("cfg_all_test"),
    "src/live.rs": 'pub fn ok() -> bool {\n    "ab".contains("a")\n}\n',
  });
  try {
    const result = lint(root, "--list");
    assert.equal(result.status, 0, result.stderr);
    assert.deepEqual(
      result.stdout
        .trim()
        .split("\n")
        .map((line) => line.split(" ")[0]),
      ["src/more.rs:3", "src/nested.rs:3", "src/parser_tests.rs:4", "assertion-lint:"],
    );
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

test("the generated ratchet holds existing findings and only goes down", () => {
  const root = tree({
    "src/lib.rs": "#[cfg(test)]\nmod tests;\n",
    "src/tests.rs": partial("first") + partial("second"),
  });
  try {
    assert.equal(lint(root).status, 1, "no ratchet yet");
    const written = lint(root, "--write-ratchet");
    assert.equal(written.status, 0, written.stderr);
    assert.match(fs.readFileSync(path.join(root, "ratchet.tsv"), "utf8"), /^src\/tests\.rs\t2$/mu);
    const held = lint(root);
    assert.equal(held.status, 0, held.stdout);
    assert.equal(held.stderr, "");
    assert.match(held.stdout, /2 findings in 1 files held by ratchet/u);

    fs.appendFileSync(path.join(root, "src/tests.rs"), partial("third"));
    assert.equal(lint(root).status, 1, "a new partial assertion exceeds the ratchet");

    fs.writeFileSync(path.join(root, "src/tests.rs"), partial("first"));
    const lowered = lint(root);
    assert.equal(lowered.status, 0, lowered.stdout);
    assert.match(lowered.stderr, /ratchet allows 2 findings in src\/tests\.rs but 1 remain/u);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

test("the committed ratchet matches the tree exactly", () => {
  const result = spawnSync("rust-script", [lintPath], { cwd: repoRoot, encoding: "utf8" });
  assert.equal(result.status, 0, result.stdout);
  assert.equal(result.stderr, "", "regenerate with --write-ratchet after fixing assertions");
});
