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
  projects: Array<FixtureProject & { lspIncrementalBudget?: unknown }>;
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
  assert.ok(
    configured.some((project) => project.id === "vue-vben-admin"),
    "vue-vben-admin must keep an authored LSP feature oracle",
  );
  assert.ok(
    configured.some((project) => project.id === "misskey"),
    "misskey must keep an authored LSP feature oracle",
  );

  const budgetOwnerIds = registry.projects
    .filter((project) => project.lspIncrementalBudget != null)
    .map((project) => project.id);
  assert.deepEqual(
    budgetOwnerIds,
    ["vue-vben-admin", "misskey"],
    "LSP incremental-budget owners must stay explicit so authored-oracle coverage can ratchet",
  );
  const authoredProjectIds = new Set(configured.map((project) => project.id));
  for (const id of budgetOwnerIds) {
    assert.ok(
      authoredProjectIds.has(id),
      `${id} has an LSP incremental budget but no authored LSP feature oracle`,
    );
  }

  for (const project of configured) {
    const oracle = project.lspAuthoredOracle;
    assert.ok(project.coverage.includes("lsp"));
    assert.notEqual(oracle.templateBinding.file, oracle.componentBoundary.importerFile);
    assert.notEqual(oracle.componentBoundary.importerFile, oracle.componentBoundary.componentFile);
    assert.ok(oracle.templateBinding.hoverContains.length > 0);
    assert.ok(oracle.componentBoundary.completionItems.length > 0);
    assert.equal(
      oracle.componentBoundary.completionItems.length,
      oracle.componentBoundary.completionItemCount,
      `${project.id} must pin every completion label and rank`,
    );
    const rankBase = oracle.componentBoundary.completionItems[0]?.rank ?? -1;
    assert.equal(rankBase, 0, `${project.id} completion ranks must start at zero`);
    assert.deepEqual(
      oracle.componentBoundary.completionItems.map((item) => item.rank),
      [...oracle.componentBoundary.completionItems.keys()].map((index) => index + rankBase),
    );
    assert.ok(oracle.componentBoundary.dependencyEdit.completionLabel.length > 0);

    const lifecycle = oracle.fileLifecycle;
    assert.ok(lifecycle, `${project.id} must declare an authored file lifecycle oracle`);
    assert.notEqual(lifecycle.copiedFile, lifecycle.renamedFile);
    assert.notEqual(lifecycle.originalImportSpecifier, lifecycle.copiedImportSpecifier);
    assert.notEqual(lifecycle.copiedImportSpecifier, lifecycle.renamedImportSpecifier);
    assert.equal(
      typeof (lifecycle.requireDeletedImportDiagnostic ?? true),
      "boolean",
      `${project.id} deleted import diagnostic requirement must be a boolean`,
    );
    assert.match(lifecycle.markerSymbol, /^__vize[A-Za-z0-9_]+__$/);
    assert.ok(lifecycle.markerInsertionAnchor.length > 0);

    const fixtureDir = path.resolve(root, project.fixturePath);
    if (fs.existsSync(fixtureDir)) {
      assert.equal(fs.existsSync(path.resolve(fixtureDir, lifecycle.copiedFile)), false);
      assert.equal(fs.existsSync(path.resolve(fixtureDir, lifecycle.renamedFile)), false);
    }
  }
});
