import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";

import { offsetToPosition } from "./support/lsp/assertions.ts";
import type { LspRange, PublishDiagnosticsParams } from "./support/lsp/protocol.ts";
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
  const session = new FakeFileLifecycleSession(workspace, importerUri, importerSource);
  const oracle = fixtureOracle();
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
    assert.equal(fs.existsSync(path.join(workspace, "__VizeOracleChild.vue")), false);
    assert.equal(fs.existsSync(path.join(workspace, "__VizeOracleRenamedChild.vue")), false);
    assert.deepEqual(session.events, [
      "workspace/didCreateFiles:__VizeOracleChild.vue",
      "workspace/symbol:__vizeFileLifecycleMarker__",
      "textDocument/didChange:2:Importer.vue",
      "textDocument/publishDiagnostics:2:Importer.vue",
      "textDocument/definition:Importer.vue",
      "workspace/willRenameFiles:__VizeOracleChild.vue->__VizeOracleRenamedChild.vue",
      "workspace/didRenameFiles:__VizeOracleChild.vue->__VizeOracleRenamedChild.vue",
      "textDocument/didChange:3:Importer.vue",
      "textDocument/publishDiagnostics:3:Importer.vue",
      "textDocument/definition:Importer.vue",
      "workspace/symbol:__vizeFileLifecycleMarker__",
      "textDocument/documentSymbol:__VizeOracleChild.vue",
      "workspace/didDeleteFiles:__VizeOracleRenamedChild.vue",
      "textDocument/publishDiagnostics:3:Importer.vue",
      "textDocument/definition:Importer.vue",
      "workspace/symbol:__vizeFileLifecycleMarker__",
      "textDocument/documentSymbol:__VizeOracleRenamedChild.vue",
      "textDocument/didChange:4:Importer.vue",
      "textDocument/publishDiagnostics:4:Importer.vue",
      "textDocument/definition:Importer.vue",
    ]);
  } finally {
    fs.rmSync(workspace, { recursive: true, force: true });
  }
});

class FakeFileLifecycleSession {
  readonly events: string[] = [];
  private importerSource: string;
  private importerVersion = 1;
  private readonly importerUri: string;
  private readonly workspace: string;
  private fileDeleted = false;

  constructor(workspace: string, importerUri: string, importerSource: string) {
    this.workspace = workspace;
    this.importerUri = importerUri;
    this.importerSource = importerSource;
  }

  notify(method: string, params: unknown): void {
    if (method === "textDocument/didChange") {
      const payload = params as {
        contentChanges: Array<{ text: string }>;
        textDocument: { uri: string; version: number };
      };
      this.importerSource = payload.contentChanges[0]!.text;
      this.importerVersion = payload.textDocument.version;
      this.events.push(`${method}:${this.importerVersion}:Importer.vue`);
      return;
    }
    if (method === "workspace/didCreateFiles" || method === "workspace/didDeleteFiles") {
      const uri = (params as { files: Array<{ uri: string }> }).files[0]!.uri;
      if (method === "workspace/didDeleteFiles") this.fileDeleted = true;
      this.events.push(`${method}:${fileName(uri)}`);
      return;
    }
    if (method === "workspace/didRenameFiles") {
      const file = (params as { files: Array<{ newUri: string; oldUri: string }> }).files[0]!;
      this.events.push(`${method}:${fileName(file.oldUri)}->${fileName(file.newUri)}`);
    }
  }

  async request(method: string, params: unknown): Promise<unknown> {
    if (method === "workspace/symbol") {
      const marker = (params as { query: string }).query;
      this.events.push(`${method}:${marker}`);
      const target = ["__VizeOracleChild.vue", "__VizeOracleRenamedChild.vue"]
        .map((file) => path.join(this.workspace, file))
        .find((file) => fs.existsSync(file));
      return target == null
        ? null
        : [{ name: marker, location: { uri: pathToFileURL(target).href, range: zeroRange() } }];
    }
    if (method === "workspace/willRenameFiles") {
      const file = (params as { files: Array<{ newUri: string; oldUri: string }> }).files[0]!;
      this.events.push(`${method}:${fileName(file.oldUri)}->${fileName(file.newUri)}`);
      const specifier = "./__VizeOracleChild.vue";
      const offset = this.importerSource.indexOf(specifier);
      assert.notEqual(offset, -1);
      return {
        changes: {
          [this.importerUri]: [
            {
              newText: "./__VizeOracleRenamedChild.vue",
              range: rangeAt(this.importerSource, offset, specifier.length),
            },
          ],
        },
      };
    }

    const uri = (params as { textDocument: { uri: string } }).textDocument.uri;
    this.events.push(`${method}:${fileName(uri)}`);
    if (method === "textDocument/documentSymbol") return null;
    assert.equal(method, "textDocument/definition");
    const specifier = /import Child from "([^"]+)"/.exec(this.importerSource)?.[1];
    assert.ok(specifier);
    const target = path.resolve(this.workspace, specifier);
    return fs.existsSync(target) ? { uri: pathToFileURL(target).href, range: zeroRange() } : null;
  }

  async waitForNotification(method: string, predicate?: (params: unknown) => boolean) {
    assert.equal(method, "textDocument/publishDiagnostics");
    const specifier = "./__VizeOracleRenamedChild.vue";
    const offset = this.importerSource.indexOf(specifier);
    const payload: PublishDiagnosticsParams = {
      diagnostics:
        this.fileDeleted && offset !== -1
          ? [
              {
                code: 2307,
                message: `Cannot find module '${specifier}' or its corresponding type declarations.`,
                range: rangeAt(this.importerSource, offset - 1, specifier.length + 2),
                severity: 1,
                source: "vize/types",
              },
            ]
          : [],
      uri: this.importerUri,
      version: this.importerVersion,
    };
    assert.ok(predicate?.(payload));
    this.events.push(`${method}:${this.importerVersion}:Importer.vue`);
    return payload;
  }
}

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

function zeroRange(): LspRange {
  return {
    start: { line: 0, character: 0 },
    end: { line: 0, character: 0 },
  };
}

function fileName(uri: string): string {
  return path.basename(new URL(uri).pathname);
}
