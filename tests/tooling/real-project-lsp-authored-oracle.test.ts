import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";

import {
  assertOrderedEvents,
  FakeAuthoredLspSession,
} from "./support/fake-authored-lsp-session.ts";
import { exerciseAuthoredLspOracle } from "./support/real-project-lsp-authored-oracle.ts";
import {
  readOracleDocument,
  responseEvidence,
  sortLocations,
} from "./support/real-project-lsp-authored-utils.ts";
import type { LspAuthoredOracle } from "./support/real-project-lsp-report.ts";

test("real-project LSP exercises authored binding and component boundary features", async () => {
  const workspace = createFixtureWorkspace("vize-authored-lsp-");
  const session = new FakeAuthoredLspSession(workspace);
  try {
    const result = await exerciseAuthoredLspOracle(session, workspace, fixtureOracle());

    assert.equal(result.completion.count, 2);
    assert.equal(result.definition.count, 1);
    assert.equal(result.hover.count, 1);
    assert.equal(result.references.count, 2);
    assert.equal(result.rename.count, 2);
    for (const evidence of [
      result.completion,
      result.componentDefinition,
      result.definition,
      result.hover,
      result.references,
      result.rename,
    ]) {
      assert.ok(Number.isFinite(evidence.durationMs));
      assert.ok(evidence.durationMs >= 0);
    }
    assert.deepEqual(result.dependencyCompletion, {
      baselineContainsProbe: false,
      changedContainsProbe: true,
      repairedContainsProbe: false,
    });
    assert.equal(result.fileLifecycle.createdDefinition.count, 1);
    assert.equal(result.fileLifecycle.renamedDefinition.count, 1);
    assert.equal(result.fileLifecycle.deletedDefinition.count, 0);
    assert.equal(result.fileLifecycle.deletedImporterDiagnostics.count, 1);
    assert.equal(result.fileLifecycle.restoredDefinition.count, 1);
    assertOrderedEvents(session.events, [
      ["textDocument/didOpen:1:Binding.vue", "textDocument/publishDiagnostics:1:Binding.vue"],
      ["textDocument/didOpen:1:FeatureChild.vue", "textDocument/didChange:2:FeatureChild.vue"],
      ["workspace/didCreateFiles", "workspace/symbol"],
      ["workspace/willRenameFiles", "workspace/didRenameFiles"],
      ["workspace/didDeleteFiles", "textDocument/publishDiagnostics:3:FeatureParent.vue"],
      ["workspace/didDeleteFiles", "textDocument/didChange:4:FeatureParent.vue"],
      ["textDocument/didChange:4:FeatureParent.vue", "textDocument/didClose:FeatureParent.vue"],
      ["textDocument/didClose:FeatureParent.vue", "textDocument/didClose:FeatureChild.vue"],
      ["textDocument/didClose:FeatureChild.vue", "textDocument/didClose:Binding.vue"],
    ]);
    assert.deepEqual(session.openFiles, []);
  } finally {
    fs.rmSync(workspace, { recursive: true, force: true });
  }
});

test("authored LSP oracle fails closed when an enabled feature returns no result", async () => {
  const workspace = createFixtureWorkspace("vize-authored-lsp-null-");
  const session = new FakeAuthoredLspSession(workspace, "textDocument/hover");
  try {
    await assert.rejects(
      () => exerciseAuthoredLspOracle(session, workspace, fixtureOracle()),
      /hover must resolve the authored title binding/,
    );
    assert.deepEqual(session.openFiles, []);
  } finally {
    fs.rmSync(workspace, { recursive: true, force: true });
  }
});

test("authored LSP oracle preserves its primary failure when cleanup also fails", async () => {
  const workspace = createFixtureWorkspace("vize-authored-lsp-cleanup-");
  const session = new FakeAuthoredLspSession(workspace, "textDocument/hover", true);
  try {
    await assert.rejects(
      () => exerciseAuthoredLspOracle(session, workspace, fixtureOracle()),
      /hover must resolve the authored title binding/,
    );
    assert.deepEqual(session.openFiles, ["Binding.vue"]);
  } finally {
    fs.rmSync(workspace, { recursive: true, force: true });
  }
});

test("fake authored diagnostics cannot match another document at the same version", async () => {
  const workspace = createFixtureWorkspace("vize-authored-lsp-diagnostic-uri-");
  const session = new FakeAuthoredLspSession(workspace);
  try {
    session.seedDocument("Binding.vue", "<template />\n");
    session.seedDocument("FeatureChild.vue", "<template />\n");
    const bindingUri = pathToFileURL(path.join(workspace, "Binding.vue")).href;
    const diagnostics = (await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (value) => {
        const payload = value as { uri?: string; version?: number };
        return payload.uri === bindingUri && payload.version === 1;
      },
    )) as { uri: string };
    assert.equal(diagnostics.uri, bindingUri);
  } finally {
    fs.rmSync(workspace, { recursive: true, force: true });
  }
});

test("authored LSP utilities fail with portable evidence and diagnostics", () => {
  const workspace = createFixtureWorkspace("vize-authored-lsp-utils-");
  try {
    assert.throws(
      () => readOracleDocument(workspace, "Missing.vue"),
      /authored oracle file is missing: Missing\.vue/,
    );

    const sorted = sortLocations([
      { uri: "file:///workspace/ä.vue", range: range(0, 0, 1) },
      { uri: "file:///workspace/z.vue", range: range(0, 0, 1) },
    ]);
    assert.deepEqual(
      sorted.map((location) => location.uri),
      ["file:///workspace/z.vue", "file:///workspace/ä.vue"],
    );

    const firstOutside = pathToFileURL(path.join(os.tmpdir(), "machine-a", "external.ts")).href;
    const secondOutside = pathToFileURL(path.join(os.tmpdir(), "machine-b", "external.ts")).href;
    assert.deepEqual(
      responseEvidence({ uri: firstOutside, z: 1, ä: 2 }, 1, workspace),
      responseEvidence({ ä: 2, z: 1, uri: secondOutside }, 1, workspace),
    );
  } finally {
    fs.rmSync(workspace, { recursive: true, force: true });
  }
});

function createFixtureWorkspace(prefix: string): string {
  const workspace = fs.mkdtempSync(path.join(os.tmpdir(), prefix));
  fs.writeFileSync(
    path.join(workspace, "Binding.vue"),
    `<script setup lang="ts">\ndefineProps<{ title: string }>()\n</script>\n<template>{{ title }}</template>\n`,
  );
  fs.writeFileSync(
    path.join(workspace, "FeatureChild.vue"),
    `<script setup lang="ts">\ndefineProps<{\n  active?: boolean;\n}>()\n</script>\n`,
  );
  fs.writeFileSync(
    path.join(workspace, "FeatureParent.vue"),
    `<script setup lang="ts">\nimport FeatureChild from './FeatureChild.vue'\n</script>\n<template><FeatureChild /></template>\n`,
  );
  return workspace;
}

function fixtureOracle(): LspAuthoredOracle {
  return {
    componentBoundary: {
      componentFile: "FeatureChild.vue",
      completionItemCount: 2,
      completionItems: [
        { label: "v-if", rank: 0 },
        { label: "active", rank: 1 },
      ],
      dependencyEdit: {
        anchor: "  active?: boolean;\n",
        completionLabel: "vize-oracle-probe",
        replacement: "  active?: boolean;\n  vizeOracleProbe?: boolean;\n",
      },
      importerFile: "FeatureParent.vue",
      tagAnchor: "<FeatureChild ",
      tagName: "FeatureChild",
    },
    fileLifecycle: {
      copiedFile: "__VizeOracleFeatureChild.vue",
      copiedImportSpecifier: "./__VizeOracleFeatureChild.vue",
      markerInsertionAnchor: '<script setup lang="ts">\n',
      markerSymbol: "__vizeFileLifecycleMarker__",
      originalImportSpecifier: "./FeatureChild.vue",
      renamedFile: "__VizeOracleRenamedFeatureChild.vue",
      renamedImportSpecifier: "./__VizeOracleRenamedFeatureChild.vue",
    },
    templateBinding: {
      declarationAnchor: "{ title: string }",
      file: "Binding.vue",
      hoverContains: ["const title: string"],
      renameTo: "renamedTitle",
      symbol: "title",
      usageAnchor: "{{ title }}",
    },
  };
}

function range(line: number, start: number, end: number) {
  return { start: { line, character: start }, end: { line, character: end } };
}
