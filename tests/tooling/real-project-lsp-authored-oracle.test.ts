import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";

import { exerciseAuthoredLspOracle } from "./support/real-project-lsp-authored-oracle.ts";
import type { PublishDiagnosticsParams } from "./support/lsp/protocol.ts";
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
    assert.deepEqual(result.dependencyCompletion, {
      baselineContainsProbe: false,
      changedContainsProbe: true,
      repairedContainsProbe: false,
    });
    assert.deepEqual(session.events, [
      "textDocument/didOpen:1:Binding.vue",
      "textDocument/publishDiagnostics:1:Binding.vue",
      "textDocument/hover:Binding.vue",
      "textDocument/definition:Binding.vue",
      "textDocument/references:Binding.vue",
      "textDocument/prepareRename:Binding.vue",
      "textDocument/rename:Binding.vue",
      "textDocument/didOpen:1:FeatureChild.vue",
      "textDocument/publishDiagnostics:1:FeatureChild.vue",
      "textDocument/didOpen:1:FeatureParent.vue",
      "textDocument/publishDiagnostics:1:FeatureParent.vue",
      "textDocument/definition:FeatureParent.vue",
      "textDocument/completion:FeatureParent.vue",
      "textDocument/didChange:2:FeatureChild.vue",
      "textDocument/publishDiagnostics:2:FeatureChild.vue",
      "textDocument/completion:FeatureParent.vue",
      "textDocument/didChange:3:FeatureChild.vue",
      "textDocument/publishDiagnostics:3:FeatureChild.vue",
      "textDocument/completion:FeatureParent.vue",
      "textDocument/didClose:FeatureParent.vue",
      "textDocument/didClose:FeatureChild.vue",
      "textDocument/didClose:Binding.vue",
    ]);
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

class FakeAuthoredLspSession {
  readonly events: string[] = [];
  readonly openFiles: string[] = [];
  private readonly documents = new Map<string, { text: string; version: number }>();
  private readonly nullMethod: string | null;
  private readonly workspace: string;

  constructor(workspace: string, nullMethod: string | null = null) {
    this.workspace = workspace;
    this.nullMethod = nullMethod;
  }

  notify(method: string, params: unknown): void {
    const payload = params as {
      contentChanges?: Array<{ text: string }>;
      textDocument?: { text?: string; uri: string; version?: number };
    };
    const document = payload.textDocument;
    assert.ok(document);
    const file = path.basename(new URL(document.uri).pathname);
    if (method === "textDocument/didOpen") {
      this.documents.set(document.uri, {
        text: document.text ?? "",
        version: document.version ?? 0,
      });
      this.openFiles.push(file);
      this.events.push(`${method}:${document.version}:${file}`);
    } else if (method === "textDocument/didChange") {
      this.documents.set(document.uri, {
        text: payload.contentChanges?.[0]?.text ?? "",
        version: document.version ?? 0,
      });
      this.events.push(`${method}:${document.version}:${file}`);
    } else if (method === "textDocument/didClose") {
      this.documents.delete(document.uri);
      const index = this.openFiles.indexOf(file);
      assert.ok(index >= 0, `didClose for a document that is not open: ${file}`);
      this.openFiles.splice(index, 1);
      this.events.push(`${method}:${file}`);
    }
  }

  async request(method: string, params: unknown): Promise<unknown> {
    const request = params as { newName?: string; textDocument: { uri: string } };
    const uri = request.textDocument.uri;
    const file = path.basename(new URL(uri).pathname);
    this.events.push(`${method}:${file}`);
    if (method === this.nullMethod) return null;

    const bindingUri = pathToFileURL(path.join(this.workspace, "Binding.vue")).href;
    const childUri = pathToFileURL(path.join(this.workspace, "FeatureChild.vue")).href;
    if (method === "textDocument/hover") {
      return {
        contents: { kind: "markdown", value: "```typescript\nconst title: string\n```" },
        range: range(3, 13, 18),
      };
    }
    if (method === "textDocument/definition" && file === "Binding.vue") {
      return { uri: bindingUri, range: range(1, 14, 19) };
    }
    if (method === "textDocument/references") {
      return [
        { uri: bindingUri, range: range(3, 13, 18) },
        { uri: bindingUri, range: range(1, 14, 19) },
      ];
    }
    if (method === "textDocument/prepareRename") return range(3, 13, 18);
    if (method === "textDocument/rename") {
      return {
        changes: {
          [bindingUri]: [
            { newText: request.newName, range: range(3, 13, 18) },
            { newText: request.newName, range: range(1, 14, 19) },
          ],
        },
      };
    }
    if (method === "textDocument/definition") return { uri: childUri, range: range(0, 0, 0) };
    if (method === "textDocument/completion") {
      const child = [...this.documents.entries()].find(([candidate]) =>
        candidate.endsWith("/FeatureChild.vue"),
      )?.[1];
      const labels = ["v-if", "active"];
      if (child?.text.includes("vizeOracleProbe")) labels.push("vize-oracle-probe");
      return labels.map((label) => ({ label }));
    }
    throw new Error(`unexpected request ${method}`);
  }

  async waitForNotification(method: string, predicate?: (params: unknown) => boolean) {
    assert.equal(method, "textDocument/publishDiagnostics");
    const notification = [...this.documents.entries()]
      .reverse()
      .map(([uri, document]) => ({
        document,
        payload: { diagnostics: [], uri, version: document.version } as PublishDiagnosticsParams,
        uri,
      }))
      .find(({ payload }) => predicate?.(payload));
    assert.ok(notification, "fake session must have a matching diagnostic notification");
    const { document, payload, uri } = notification;
    this.events.push(`${method}:${document.version}:${path.basename(new URL(uri).pathname)}`);
    return payload;
  }
}

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
