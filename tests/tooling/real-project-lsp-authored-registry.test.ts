import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import type { FixtureProject, LspAuthoredOracle } from "./support/real-project-lsp-report.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const registryPath = path.join(root, "tests", "_fixtures", "vue-ecosystem-fixtures.json");

type Registry = {
  lspAuthoredOracleGate: { minimumProjectCount: number; trackingIssue: number };
  projects: FixtureProject[];
};

test("authored LSP feature oracles are explicit and ratcheted", () => {
  const registry = JSON.parse(fs.readFileSync(registryPath, "utf8")) as Registry;
  const configured = registry.projects.filter(
    (project): project is FixtureProject & { lspAuthoredOracle: LspAuthoredOracle } =>
      project.lspAuthoredOracle != null,
  );

  assert.equal(registry.lspAuthoredOracleGate.trackingIssue, 3952);
  assert.ok(registry.lspAuthoredOracleGate.minimumProjectCount > 0);
  assert.ok(configured.length >= registry.lspAuthoredOracleGate.minimumProjectCount);
  assert.deepEqual(
    configured.map((project) => project.id),
    ["misskey"],
  );

  for (const project of configured) {
    const oracle = project.lspAuthoredOracle;
    assert.ok(project.coverage.includes("lsp"));
    assert.notEqual(oracle.templateBinding.file, oracle.componentBoundary.importerFile);
    assert.notEqual(oracle.componentBoundary.importerFile, oracle.componentBoundary.componentFile);
    assert.ok(oracle.templateBinding.hoverContains.length > 0);
    assert.ok(oracle.componentBoundary.completionItems.length > 0);
    assert.ok(
      oracle.componentBoundary.completionItemCount >=
        oracle.componentBoundary.completionItems.length,
    );
    assert.deepEqual(
      oracle.componentBoundary.completionItems.map((item) => item.rank),
      [...oracle.componentBoundary.completionItems.keys()].map((index) => index + 28),
    );
    assert.ok(oracle.componentBoundary.dependencyEdit.completionLabel.length > 0);
  }
});
