import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";

import { offsetToPosition } from "./support/lsp/assertions.ts";
import type { LspRange, PublishDiagnosticsParams } from "./support/lsp/protocol.ts";
import {
  assertOrderedEvents,
  FakeAuthoredLspSession,
} from "./support/fake-authored-lsp-session.ts";
import { exerciseAuthoredFileLifecycle } from "./support/real-project-lsp-file-lifecycle.ts";
import type { LspAuthoredOracle } from "./support/real-project-lsp-report.ts";

test("authored file lifecycle creates, renames, deletes, and repairs a dependency", async () => {
  const workspace = fs.mkdtempSync(path.join(os.tmpdir(), "vize-authored-file-lifecycle-"));
  const importerSource = `<script setup lang="ts">
import Child from "./Child.vue"
</script>
<template><Child /></template>
`;
  const dependencySource = `<script setup lang="ts">
const childValue = 1
</script>
<template>{{ childValue }}</template>
`;
  const importerPath = path.join(workspace, "Importer.vue");
  const dependencyPath = path.join(workspace, "Child.vue");
  fs.writeFileSync(importerPath, importerSource);
  fs.writeFileSync(dependencyPath, dependencySource);
  const importerUri = pathToFileURL(importerPath).href;
  const oracle = fixtureOracle();
  const session = new FakeAuthoredLspSession(
    workspace,
    null,
    false,
    {
      copiedFile: oracle.fileLifecycle.copiedFile,
      copiedImportSpecifier: oracle.fileLifecycle.copiedImportSpecifier,
      importerFile: oracle.componentBoundary.importerFile,
      renamedFile: oracle.fileLifecycle.renamedFile,
      renamedImportSpecifier: oracle.fileLifecycle.renamedImportSpecifier,
    },
    true,
    true,
  );
  session.seedDocument(oracle.componentBoundary.importerFile, importerSource);
  const tagOffset = importerSource.indexOf("Child />");
  const tagRange = rangeAt(importerSource, tagOffset, "Child".length);
  const baselineDiagnostics: PublishDiagnosticsParams = {
    diagnostics: [
      {
        code: 2307,
        message: "Cannot find module '@transient/package' or its corresponding type declarations.",
        range: {
          end: { character: 34, line: 1 },
          start: { character: 14, line: 1 },
        },
        severity: 1,
        source: "vize/types",
      },
    ],
    uri: importerUri,
    version: 1,
  };

  try {
    const result = await exerciseAuthoredFileLifecycle(
      session,
      workspace,
      oracle,
      { source: importerSource, uri: importerUri },
      { source: dependencySource, uri: pathToFileURL(dependencyPath).href },
      tagRange,
      baselineDiagnostics,
      () => 1_000,
    );

    assert.equal(result.createdDefinition.count, 1);
    assert.equal(result.createdWorkspaceSymbols.count, 1);
    assert.equal(result.renameEdit.count, 1);
    assert.equal(result.renamedDefinition.count, 1);
    assert.equal(result.renamedWorkspaceSymbols.count, 1);
    assert.equal(result.staleCopiedDocumentSymbols.count, 0);
    assert.equal(result.deletedDefinition.count, 0);
    assert.equal(result.deletedWorkspaceSymbols.count, 0);
    assert.equal(result.deletedDocumentSymbols.count, 0);
    assert.equal(result.deletedImporterDiagnostics.count, 1);
    assert.equal(result.restoredDefinition.count, 1);
    assert.deepEqual(result.repairedDiagnostics, {
      count: 0,
      sha256: "01ba4719c80b6fe911b091a7c05124b64eeece964e09c058ef8f9805daca546b",
    });
    assert.ok(
      session.events.some((event) => event.endsWith(":stale-delete-skipped")),
      "delete diagnostics wait must skip stale same-version payloads",
    );
    assert.equal(fs.existsSync(path.join(workspace, "__VizeOracleChild.vue")), false);
    assert.equal(fs.existsSync(path.join(workspace, "__VizeOracleRenamedChild.vue")), false);
    assertOrderedEvents(session.events, [
      ["workspace/didCreateFiles", "workspace/symbol"],
      ["workspace/didCreateFiles", "textDocument/didChange:2"],
      ["workspace/willRenameFiles", "workspace/didRenameFiles"],
      ["workspace/didRenameFiles", "textDocument/didChange:3"],
      ["workspace/didDeleteFiles", "textDocument/publishDiagnostics:3"],
      ["workspace/didDeleteFiles", "textDocument/definition"],
      ["workspace/didDeleteFiles", "textDocument/didChange:4"],
    ]);
  } finally {
    fs.rmSync(workspace, { recursive: true, force: true });
  }
});

test("authored file lifecycle can skip projects without deleted import diagnostics", async () => {
  const workspace = fs.mkdtempSync(path.join(os.tmpdir(), "vize-authored-file-lifecycle-"));
  const importerSource = `<script setup lang="ts">
import Child from "./Child.vue"
</script>
<template><Child /></template>
`;
  const dependencySource = `<script setup lang="ts">
const childValue = 1
</script>
<template>{{ childValue }}</template>
`;
  const importerPath = path.join(workspace, "Importer.vue");
  const dependencyPath = path.join(workspace, "Child.vue");
  fs.writeFileSync(importerPath, importerSource);
  fs.writeFileSync(dependencyPath, dependencySource);
  const importerUri = pathToFileURL(importerPath).href;
  const oracle = fixtureOracle();
  oracle.fileLifecycle.requireDeletedImportDiagnostic = false;
  const session = new FakeAuthoredLspSession(
    workspace,
    null,
    false,
    {
      copiedFile: oracle.fileLifecycle.copiedFile,
      copiedImportSpecifier: oracle.fileLifecycle.copiedImportSpecifier,
      importerFile: oracle.componentBoundary.importerFile,
      renamedFile: oracle.fileLifecycle.renamedFile,
      renamedImportSpecifier: oracle.fileLifecycle.renamedImportSpecifier,
    },
    false,
  );
  session.seedDocument(oracle.componentBoundary.importerFile, importerSource);
  const tagOffset = importerSource.indexOf("Child />");
  const tagRange = rangeAt(importerSource, tagOffset, "Child".length);
  const baselineDiagnostics: PublishDiagnosticsParams = {
    diagnostics: [],
    uri: importerUri,
    version: 1,
  };

  try {
    const result = await exerciseAuthoredFileLifecycle(
      session,
      workspace,
      oracle,
      { source: importerSource, uri: importerUri },
      { source: dependencySource, uri: pathToFileURL(dependencyPath).href },
      tagRange,
      baselineDiagnostics,
      () => 1_000,
    );

    assert.equal(result.deletedDefinition.count, 0);
    assert.equal(result.deletedImporterDiagnostics.count, 0);
    assert.equal(result.restoredDefinition.count, 1);
  } finally {
    fs.rmSync(workspace, { recursive: true, force: true });
  }
});

function fixtureOracle(): LspAuthoredOracle {
  return {
    componentBoundary: {
      componentFile: "Child.vue",
      completionItemCount: 1,
      completionItems: [{ label: "value", rank: 0 }],
      dependencyEdit: {
        anchor: "const childValue = 1",
        completionLabel: "probe",
        replacement: "const childValue = 1\nconst probe = 1",
      },
      importerFile: "Importer.vue",
      tagAnchor: "<Child ",
      tagName: "Child",
    },
    fileLifecycle: {
      copiedFile: "__VizeOracleChild.vue",
      copiedImportSpecifier: "./__VizeOracleChild.vue",
      markerInsertionAnchor: '<script setup lang="ts">\n',
      markerSymbol: "__vizeFileLifecycleMarker__",
      originalImportSpecifier: "./Child.vue",
      renamedFile: "__VizeOracleRenamedChild.vue",
      renamedImportSpecifier: "./__VizeOracleRenamedChild.vue",
    },
    templateBinding: {
      declarationAnchor: "childValue = 1",
      file: "Child.vue",
      hoverContains: ["number"],
      renameTo: "renamedChildValue",
      symbol: "childValue",
      usageAnchor: "{{ childValue }}",
    },
  };
}

function rangeAt(source: string, offset: number, length: number): LspRange {
  return {
    start: offsetToPosition(source, offset),
    end: offsetToPosition(source, offset + length),
  };
}
