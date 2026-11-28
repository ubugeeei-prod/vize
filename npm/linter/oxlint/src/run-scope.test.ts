import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { ONE_SHOT_OXLINT_RUN, isOneShotOxlintRun, packageResolution } from "./run-scope.ts";

const node = process.execPath;
const oxlintBin = path.join("project", "node_modules", "oxlint", "bin", "oxlint");

await test("only an oxlint CLI process that serves no language server is one run", () => {
  assert.equal(isOneShotOxlintRun([node, oxlintBin, "-c", ".oxlintrc.json", "."]), true);
  assert.equal(isOneShotOxlintRun([node, path.join("project", ".bin", "oxlint"), "."]), true);
  assert.equal(isOneShotOxlintRun([node, oxlintBin, "--lsp"]), false);
  // Hosts that may lint the same files again after an edit.
  assert.equal(isOneShotOxlintRun([node, path.join("project", "node_modules", "vp")]), false);
  assert.equal(isOneShotOxlintRun([node, path.join("project", "lint-files.ts"), "--fix"]), false);
  assert.equal(isOneShotOxlintRun([node, path.join("project", "oxlint-vize")]), false);
  assert.equal(isOneShotOxlintRun([node]), false);
  // This test runner is such a host.
  assert.equal(ONE_SHOT_OXLINT_RUN, false);
});

await test("a long-lived host finds a package installed after its first lookup", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "oxlint-vize-run-scope-"));
  try {
    const component = path.join(root, "src", "App.vue");
    fs.mkdirSync(path.dirname(component), { recursive: true });
    const fresh = packageResolution("vize-run-scope-probe", false);
    const remembered = packageResolution("vize-run-scope-probe", true);
    assert.equal(fresh(component), false);
    assert.equal(remembered(component), false);

    const probe = path.join(root, "node_modules", "vize-run-scope-probe");
    fs.mkdirSync(probe, { recursive: true });
    fs.writeFileSync(path.join(probe, "package.json"), '{"name":"vize-run-scope-probe"}');
    fs.writeFileSync(path.join(probe, "index.js"), "module.exports = {};\n");

    assert.equal(fresh(component), true);
    // One run's files hold still, so it keeps its first answer for the directory.
    assert.equal(remembered(component), false);
    assert.equal(packageResolution("vize-run-scope-probe", true)(component), true);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});
