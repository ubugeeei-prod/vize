import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs, { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import path from "node:path";
import { tmpdir } from "node:os";
import { test } from "node:test";

import { repoRoot } from "./_helpers/tool-commands.ts";
import { readRepoFile } from "./support/github-workflows.ts";

const commandPath = "tools/support/github/clean-node-binaries.ts";

function runCleanNodeBinaries(args: readonly string[]) {
  return spawnSync(process.execPath, [path.join(repoRoot, commandPath), ...args], {
    cwd: repoRoot,
    encoding: "utf8",
  });
}

test("github/clean-node-binaries removes only top-level .node files", () => {
  const tempDir = mkdtempSync(path.join(tmpdir(), "node-clean-node-"));
  const firstDir = path.join(tempDir, "first");
  const secondDir = path.join(tempDir, "second");
  const nestedDir = path.join(firstDir, "nested");

  try {
    fs.mkdirSync(nestedDir, { recursive: true });
    fs.mkdirSync(secondDir, { recursive: true });
    writeFileSync(path.join(firstDir, "native.node"), "native");
    writeFileSync(path.join(firstDir, "keep.txt"), "keep");
    writeFileSync(path.join(nestedDir, "nested.node"), "nested");
    writeFileSync(path.join(secondDir, "addon.node"), "addon");

    const result = runCleanNodeBinaries([firstDir, secondDir]);

    assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
    assert.equal(fs.existsSync(path.join(firstDir, "native.node")), false);
    assert.equal(fs.existsSync(path.join(secondDir, "addon.node")), false);
    assert.equal(fs.existsSync(path.join(firstDir, "keep.txt")), true);
    assert.equal(fs.existsSync(path.join(nestedDir, "nested.node")), true);
  } finally {
    rmSync(tempDir, { recursive: true, force: true });
  }
});

test("release recipes clean native binaries with Node type stripping", () => {
  const workflow = readRepoFile(".github", "workflows", "release.yml");
  assert.match(
    readRepoFile(".github", "actions", "build-release-native", "action.yml"),
    /run: node tools\/support\/github\/clean-node-binaries\.ts npm\/native npm\/fresco-native$/m,
  );
  assert.match(
    readRepoFile(".github", "actions", "release-npm-artifacts", "action.yml"),
    /node tools\/support\/github\/clean-node-binaries\.ts npm\/fresco-native$/m,
  );
  assert.doesNotMatch(workflow, /tools\/moon\/cmd\/github\/clean_node_binaries/);
});
