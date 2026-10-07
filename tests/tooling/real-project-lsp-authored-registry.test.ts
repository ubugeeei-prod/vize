import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import type { FixtureProject, LspAuthoredOracle } from "./support/real-project-lsp-report.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const registryPath = path.join(root, "tests", "_fixtures", "vue-ecosystem-fixtures.json");
const commonCorpus = path.join(
  root,
  "tests/_fixtures/differential/lsp/component-global-attributes-8015",
);

type Registry = {
  lspAuthoredOracleGate: { minimumProjectCount: number; trackingIssue: number };
  projects: Array<FixtureProject & { repository: string; lspIncrementalBudget?: unknown }>;
};

function originalRegistry(): Registry {
  // Complete registry before #8015; this is source authority, not runtime capture.
  const raw = fs.readFileSync(path.join(commonCorpus, "real-project-before.json"));
  assert.equal(
    createHash("sha256").update(raw).digest("hex"),
    "cfce6418b777f0668063c424ae74c74f4b223147046abf3f69e0ff4effb3c8c5",
  );
  return JSON.parse(raw.toString()) as Registry;
}

test("authored LSP feature oracles are explicit and ratcheted", () => {
  const registry = JSON.parse(fs.readFileSync(registryPath, "utf8")) as Registry;
  const configured = registry.projects.filter(
    (project): project is Registry["projects"][number] & { lspAuthoredOracle: LspAuthoredOracle } =>
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
  assert.ok(
    configured.some((project) => project.id === "vue-flow"),
    "vue-flow must keep an authored LSP feature oracle",
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
    if (oracle.templateBinding.file === oracle.componentBoundary.importerFile) {
      assert.equal(
        oracle.templateBinding.sharesComponentImporter,
        true,
        `${project.id} must explicitly mark shared authored LSP files`,
      );
    } else {
      assert.notEqual(oracle.templateBinding.sharesComponentImporter, true);
    }
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

test("event completion goldens remain bound to the six pinned component declarations", () => {
  const registry = JSON.parse(fs.readFileSync(registryPath, "utf8")) as Registry;
  const historical = originalRegistry();
  const corpus = JSON.parse(
    fs.readFileSync(
      path.join(root, "tests/_fixtures/lsp-authored-component-event-contracts.json"),
      "utf8",
    ),
  ) as {
    cases: Array<{
      projectId: string;
      revision: string;
      componentFile: string;
      sourceUrl: string;
      sourceBlob: string;
      originalCompletionCount: number;
      declaration: string;
      source: string;
      eventLabels: string[];
    }>;
  };
  assert.deepEqual(
    corpus.cases.map((entry) => entry.projectId),
    ["vue-vben-admin", "pinia", "varlet", "element-plus", "vue-datepicker", "misskey"],
  );
  for (const entry of corpus.cases) {
    const project = registry.projects.find((candidate) => candidate.id === entry.projectId);
    const originalProject = historical.projects.find(
      (candidate) => candidate.id === entry.projectId,
    );
    assert.ok(project?.lspAuthoredOracle);
    assert.ok(originalProject?.lspAuthoredOracle);
    assert.equal(entry.revision, project.revision);
    assert.equal(entry.componentFile, project.lspAuthoredOracle.componentBoundary.componentFile);
    assert.match(entry.sourceBlob, /^[0-9a-f]{40}$/);
    assert.equal(
      entry.sourceUrl,
      `${project.repository}/blob/${entry.revision}/${entry.componentFile}`,
    );
    assert.equal(entry.source, `<script setup lang="ts">\n${entry.declaration}\n</script>\n`);
    const boundary = originalProject.lspAuthoredOracle.componentBoundary;
    assert.equal(
      boundary.completionItemCount,
      entry.originalCompletionCount + entry.eventLabels.length,
    );
    assert.deepEqual(
      boundary.completionItems.slice(entry.originalCompletionCount),
      entry.eventLabels.map((label, index) => ({
        label,
        rank: entry.originalCompletionCount + index,
      })),
    );
    const current = project.lspAuthoredOracle.componentBoundary;
    const added = current.completionItemCount - boundary.completionItemCount;
    assert.deepEqual(
      current.completionItems.slice(entry.originalCompletionCount + added),
      entry.eventLabels.map((label, index) => ({
        label,
        rank: entry.originalCompletionCount + added + index,
      })),
    );
  }
});

await test("common component attributes change only the complete ranked boundary banks", () => {
  const current = JSON.parse(fs.readFileSync(registryPath, "utf8")) as Registry;
  const expected = originalRegistry();
  const commonRaw = fs.readFileSync(path.join(commonCorpus, "common.expected.json"));
  assert.equal(
    createHash("sha256").update(commonRaw).digest("hex"),
    "0dd210eb41e97a9c80cdbf6ff26af971d8ed06315de73fe0caf262a9e96b6318",
  );
  const common = (JSON.parse(commonRaw.toString()) as { label: string }[]).map(
    (item) => item.label,
  );
  assert.deepEqual(common, [
    "id",
    "class",
    "style",
    "title",
    "role",
    "tabindex",
    "aria-label",
    "ref",
    "key",
  ]);
  const directiveEvents = [
    "v-if",
    "v-else-if",
    "v-else",
    "v-for",
    "v-on",
    "v-bind",
    "v-model",
    "v-slot",
    "v-show",
    "v-pre",
    "v-once",
    "v-memo",
    "v-cloak",
    "v-text",
    "v-html",
    "@",
    ":",
    "#",
    "@click",
    "@input",
    "@change",
    "@submit",
    "@keydown",
    "@keyup",
    "@focus",
    "@blur",
    "@mouseenter",
    "@mouseleave",
  ];
  let banks = 0;
  for (const project of expected.projects) {
    if (!project.lspAuthoredOracle) continue;
    banks++;
    const boundary = project.lspAuthoredOracle.componentBoundary;
    assert.ok(/^[A-Z]/.test(boundary.tagName) || boundary.tagName.includes("-"));
    const original = boundary.completionItems;
    assert.equal(original.length, boundary.completionItemCount);
    assert.deepEqual(
      original.map((item) => item.rank),
      [...original.keys()],
    );
    assert.deepEqual(
      original.slice(0, 28).map((item) => item.label),
      directiveEvents,
    );
    // An event named @title is not a declared PROPERTY named title.
    const declared = new Set(
      original
        .slice(28)
        .filter((item) => !item.label.startsWith("@"))
        .map((item) => item.label),
    );
    const added = common.filter((label) => !declared.has(label));
    boundary.completionItems = [
      ...original.slice(0, 28),
      ...added.map((label) => ({ label, rank: 0 })),
      ...original.slice(28),
    ].map((item, rank) => ({ ...item, rank }));
    boundary.completionItemCount = boundary.completionItems.length;
    assert.equal(
      new Set(boundary.completionItems.map((item) => item.label)).size,
      boundary.completionItemCount,
    );
  }
  assert.equal(banks, 51);
  // Every complete input, revision, dependency edit, config, non-completion
  // oracle, budget and top-level registry field retains its original value.
  assert.deepEqual(current, expected);
});
