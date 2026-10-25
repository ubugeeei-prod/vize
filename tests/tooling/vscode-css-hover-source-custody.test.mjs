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
  verifyPinnedCssHoverProduct,
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
  for (const change of [
    "worktree",
    "staged",
    "untracked",
    "build-input",
    "davinci",
    "vendor-untracked",
    "assets-staged",
    "embedded-schema",
    "auxiliary-cargo",
  ]) {
    const repository = fs.mkdtempSync(path.join(os.tmpdir(), "vize-css-source-custody-"));
    const git = (...args) => execFileSync("git", args, { cwd: repository, encoding: "utf8" });
    try {
      fs.mkdirSync(path.join(repository, "crates"));
      fs.writeFileSync(path.join(repository, "crates/example.rs"), "pub const VALUE: u8 = 1;\n");
      fs.writeFileSync(path.join(repository, "Cargo.toml"), "[workspace]\n");
      for (const name of [
        "davinci/example.rs",
        "vendor/example.rs",
        "assets/example.txt",
        "npm/cli/schemas/vize.config.schema.json",
        "tests/vize_test_runner/Cargo.toml",
      ]) {
        fs.mkdirSync(path.dirname(path.join(repository, name)), { recursive: true });
        fs.writeFileSync(path.join(repository, name), "original\n");
      }
      git("init", "--quiet");
      git("add", ".");
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
      else if (change === "davinci")
        fs.appendFileSync(path.join(repository, "davinci/example.rs"), "dirty\n");
      else if (change === "vendor-untracked")
        fs.writeFileSync(path.join(repository, "vendor/untracked.rs"), "dirty\n");
      else if (change === "assets-staged") {
        fs.appendFileSync(path.join(repository, "assets/example.txt"), "dirty\n");
        git("add", "assets/example.txt");
      } else if (change === "embedded-schema")
        fs.appendFileSync(
          path.join(repository, "npm/cli/schemas/vize.config.schema.json"),
          "dirty\n",
        );
      else if (change === "auxiliary-cargo")
        fs.appendFileSync(path.join(repository, "tests/vize_test_runner/Cargo.toml"), "dirty\n");
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

await test("merged main integration rejects altered postimages, modes, preimages and extra paths", () => {
  const root = fileURLToPath(new URL("../../", import.meta.url));
  const contract = JSON.parse(
    fs.readFileSync(
      new URL("../../editors/vscode/test/css-hover-source-custody.json", import.meta.url),
    ),
  );
  const entries = (revision) =>
    execFileSync("git", ["ls-tree", "-r", "-z", revision, "--", "crates"], {
      cwd: root,
      encoding: "utf8",
      maxBuffer: 32 * 1024 * 1024,
    })
      .split("\0")
      .filter(Boolean)
      .map((line) => {
        const [header, name] = line.split("\t");
        const [mode, type, oid] = header.split(" ");
        return { path: name, mode, type, oid };
      });
  const p1 = composeReviewedCssTrees(entries(contract.historicalBaselineRevision), [
    contract.layers[0],
  ]);
  for (const [preimage, target] of [
    [p1, contract.mergedMain],
    [entries(contract.mergedMain.sourceRevision), contract.successorMain],
  ]) {
    const actual = entries(target.sourceRevision);
    for (const mutation of ["blob", "mode", "preimage", "extra"]) {
      const layer = structuredClone(target.integrationLayer);
      if (mutation === "blob") layer.changes[0].after.oid = "1".repeat(40);
      else if (mutation === "mode") layer.changes[0].after.mode = "100755";
      else if (mutation === "preimage")
        layer.changes[0].before = row(layer.changes[0].path, "2".repeat(40));
      else
        layer.changes.push({
          path: "crates/unreviewed.rs",
          before: null,
          after: row("crates/unreviewed.rs", "3".repeat(40)),
        });
      assert.throws(
        () => assertExactCssTree(actual, composeReviewedCssTrees(preimage, [layer])),
        /reviewed (blob\/mode|preimage)/,
      );
    }
  }
});

await test("historical anchors and the pinned integrated product tree remain qualified", () => {
  const root = fileURLToPath(new URL("../../", import.meta.url));
  const report = verifyPinnedCssHoverProduct(root);
  assert.equal(report.productionChanges.length, 158);
  assert.equal(report.reviewedCssChanges.length, 9);
  assert.equal(report.mergedMain.integrationLayer.changes.length, 34);
  assert.equal(report.successorMain.integrationLayer.changes.length, 122);
  assert.equal(report.historicalBaselineRevision, "2934b35b18393c188b3da5f8e50595299589fb8c");
  assert.equal(report.historicalCapture.sourceRevision, "0cc8615736db2f2ef6b82177aa2e92a7de9f86dc");
  assert.equal(report.historicalCapture.witnesses.length, 5);
  const contract = JSON.parse(
    fs.readFileSync(
      new URL("../../editors/vscode/test/css-hover-source-custody.json", import.meta.url),
    ),
  );
  assert.equal(report.sourceRevision, contract.productTreeRevision);
  assert.equal(report.proofKind, "pinned-product-snapshot");
  assert.deepEqual(
    contract.layers.map((layer) => layer.changes.length),
    [7, 2],
  );
});
