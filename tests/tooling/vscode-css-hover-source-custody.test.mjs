import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  assertExactCssTree,
  assertCleanCssSourceWorkingTree,
  composeReviewedCssTrees,
  verifyCssHoverSourceCustody,
} from "../../editors/vscode/test/css-hover-source-custody.mjs";

const row = (path, oid, mode = "100644") => ({ path, mode, type: "blob", oid });
const original = row("crates/example.rs", "1".repeat(40));
const first = row(original.path, "2".repeat(40));
const second = row(original.path, "3".repeat(40));
const layers = [
  { changes: [{ path: original.path, before: original, after: first }] },
  { changes: [{ path: original.path, before: first, after: second }] },
];

await test("reviewed layers compose exact ordered preimages and final blob/mode pairs", () => {
  const expected = composeReviewedCssTrees([original], layers);
  assert.deepEqual(expected, [second]);
  assertExactCssTree([second], expected);
  assert.throws(
    () => composeReviewedCssTrees([original], [...layers].reverse()),
    /reviewed preimage/,
  );
});

await test("new paths, allowed-path tampering and mode changes cannot pass source custody", () => {
  const expected = [second];
  for (const actual of [
    [first],
    [row(second.path, second.oid, "100755")],
    [second, row("crates/unreviewed.rs", "4".repeat(40))],
    [],
  ])
    assert.throws(() => assertExactCssTree(actual, expected), /reviewed blob\/mode/);
});

await test("dirty and staged approved sources, untracked source and dirty build inputs fail closed", () => {
  for (const change of ["worktree", "staged", "untracked", "build-input"]) {
    const repository = fs.mkdtempSync(path.join(os.tmpdir(), "vize-css-source-custody-"));
    const git = (...args) => execFileSync("git", args, { cwd: repository, encoding: "utf8" });
    try {
      fs.mkdirSync(path.join(repository, "crates"));
      fs.writeFileSync(path.join(repository, "crates/example.rs"), "pub const VALUE: u8 = 1;\n");
      fs.writeFileSync(path.join(repository, "Cargo.toml"), "[workspace]\n");
      git("init", "--quiet");
      git("add", "crates", "Cargo.toml");
      git(
        "-c",
        "user.name=Vize fixture",
        "-c",
        "user.email=fixture@example.invalid",
        "-c",
        "commit.gpgsign=false",
        "-c",
        "core.hooksPath=/dev/null",
        "commit",
        "--quiet",
        "-m",
        "test: pin source fixture",
      );
      fs.mkdirSync(path.join(repository, "output"));
      fs.writeFileSync(path.join(repository, "output/evidence.json"), "{}\n");
      assertCleanCssSourceWorkingTree(repository);
      if (change === "untracked")
        fs.writeFileSync(path.join(repository, "crates/extra.rs"), "pub const EXTRA: u8 = 2;\n");
      else if (change === "build-input")
        fs.appendFileSync(path.join(repository, "Cargo.toml"), "members = []\n");
      else {
        fs.writeFileSync(path.join(repository, "crates/example.rs"), "pub const VALUE: u8 = 2;\n");
        if (change === "staged") git("add", "crates/example.rs");
      }
      assert.throws(
        () => assertCleanCssSourceWorkingTree(repository),
        /tracked, staged or untracked source bytes/,
      );
    } finally {
      fs.rmSync(repository, { recursive: true, force: true });
    }
  }
});

await test("historical anchors and the actual integrated source tree remain qualified", () => {
  const root = fileURLToPath(new URL("../../", import.meta.url));
  const revision = execFileSync("git", ["rev-parse", "HEAD"], {
    cwd: root,
    encoding: "utf8",
  }).trim();
  const report = verifyCssHoverSourceCustody(root, revision);
  assert.equal(report.productionChanges.length, 9);
  assert.equal(report.historicalBaselineRevision, "2934b35b18393c188b3da5f8e50595299589fb8c");
  assert.equal(report.historicalCapture.sourceRevision, "0cc8615736db2f2ef6b82177aa2e92a7de9f86dc");
  assert.equal(report.historicalCapture.witnesses.length, 5);
  const contract = JSON.parse(
    fs.readFileSync(
      new URL("../../editors/vscode/test/css-hover-source-custody.json", import.meta.url),
    ),
  );
  assert.deepEqual(
    contract.layers.map((layer) => layer.changes.length),
    [7, 2],
  );
});
