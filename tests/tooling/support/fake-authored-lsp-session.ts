import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";

import type { PublishDiagnosticsParams } from "./lsp/protocol.ts";

export class FakeAuthoredLspSession {
  readonly events: string[] = [];
  readonly openFiles: string[] = [];
  private readonly documents = new Map<string, { text: string; version: number }>();
  private readonly nullMethod: string | null;
  private readonly throwOnClose: boolean;
  private readonly workspace: string;
  private fileDeleted = false;

  constructor(workspace: string, nullMethod: string | null = null, throwOnClose = false) {
    this.workspace = workspace;
    this.nullMethod = nullMethod;
    this.throwOnClose = throwOnClose;
  }

  notify(method: string, params: unknown): void {
    if (method === "workspace/didCreateFiles" || method === "workspace/didDeleteFiles") {
      const uri = (params as { files: Array<{ uri: string }> }).files[0]!.uri;
      if (method === "workspace/didDeleteFiles") this.fileDeleted = true;
      this.events.push(`${method}:${fileName(uri)}`);
      return;
    }
    if (method === "workspace/didRenameFiles") {
      const file = (params as { files: Array<{ newUri: string; oldUri: string }> }).files[0]!;
      this.events.push(`${method}:${fileName(file.oldUri)}->${fileName(file.newUri)}`);
      return;
    }
    const payload = params as {
      contentChanges?: Array<{ text: string }>;
      textDocument?: { text?: string; uri: string; version?: number };
    };
    const document = payload.textDocument;
    assert.ok(document);
    const file = fileName(document.uri);
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
      if (this.throwOnClose) throw new Error("cleanup transport failed");
      this.documents.delete(document.uri);
      const index = this.openFiles.indexOf(file);
      assert.ok(index >= 0, `didClose for a document that is not open: ${file}`);
      this.openFiles.splice(index, 1);
      this.events.push(`${method}:${file}`);
    }
  }

  async request(method: string, params: unknown): Promise<unknown> {
    if (method === "workspace/symbol") return this.workspaceSymbol(params);
    if (method === "workspace/willRenameFiles") return this.willRename(params);

    const request = params as { newName?: string; textDocument: { uri: string } };
    const uri = request.textDocument.uri;
    const file = fileName(uri);
    this.events.push(`${method}:${file}`);
    if (method === this.nullMethod) return null;

    const bindingUri = pathToFileURL(path.join(this.workspace, "Binding.vue")).href;
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
    if (method === "textDocument/documentSymbol") return null;
    if (method === "textDocument/definition") return this.componentDefinition(uri);
    if (method === "textDocument/completion") return this.componentCompletion();
    throw new Error(`unexpected request ${method}`);
  }

  async waitForNotification(method: string, predicate?: (params: unknown) => boolean) {
    assert.equal(method, "textDocument/publishDiagnostics");
    const notification = [...this.documents.entries()]
      .reverse()
      .map(([uri, document]) => {
        const diagnostics = this.deletedDependencyDiagnostics(document.text);
        return {
          document,
          payload: { diagnostics, uri, version: document.version } as PublishDiagnosticsParams,
          uri,
        };
      })
      .find(({ payload }) => predicate?.(payload));
    assert.ok(notification, "fake session must have a matching diagnostic notification");
    const { document, payload, uri } = notification;
    this.events.push(`${method}:${document.version}:${fileName(uri)}`);
    return payload;
  }

  private workspaceSymbol(params: unknown): unknown {
    const marker = (params as { query: string }).query;
    this.events.push(`workspace/symbol:${marker}`);
    const target = ["__VizeOracleFeatureChild.vue", "__VizeOracleRenamedFeatureChild.vue"]
      .map((file) => path.join(this.workspace, file))
      .find((file) => fs.existsSync(file));
    return target == null
      ? null
      : [{ name: marker, location: { uri: pathToFileURL(target).href, range: range(0, 0, 0) } }];
  }

  private willRename(params: unknown): unknown {
    const file = (params as { files: Array<{ newUri: string; oldUri: string }> }).files[0]!;
    this.events.push(
      `workspace/willRenameFiles:${fileName(file.oldUri)}->${fileName(file.newUri)}`,
    );
    const importerUri = pathToFileURL(path.join(this.workspace, "FeatureParent.vue")).href;
    const source = this.documents.get(importerUri)?.text ?? "";
    const specifier = "./__VizeOracleFeatureChild.vue";
    const offset = source.indexOf(specifier);
    assert.notEqual(offset, -1);
    return {
      changes: {
        [importerUri]: [
          {
            newText: "./__VizeOracleRenamedFeatureChild.vue",
            range: rangeAt(source, offset, specifier.length),
          },
        ],
      },
    };
  }

  private componentDefinition(uri: string): unknown {
    const source = this.documents.get(uri)?.text ?? "";
    const specifier = /import FeatureChild from ['"]([^'"]+)['"]/.exec(source)?.[1];
    assert.ok(specifier);
    const target = path.resolve(this.workspace, specifier);
    return fs.existsSync(target)
      ? { uri: pathToFileURL(target).href, range: range(0, 0, 0) }
      : null;
  }

  private componentCompletion(): Array<{ label: string }> {
    const child = [...this.documents.entries()].find(([candidate]) =>
      candidate.endsWith("/FeatureChild.vue"),
    )?.[1];
    const labels = ["v-if", "active"];
    if (child?.text.includes("vizeOracleProbe")) labels.push("vize-oracle-probe");
    return labels.map((label) => ({ label }));
  }

  private deletedDependencyDiagnostics(source: string) {
    const specifier = "./__VizeOracleRenamedFeatureChild.vue";
    const offset = source.indexOf(specifier);
    if (!this.fileDeleted || offset === -1) return [];
    return [
      {
        code: 2307,
        message: `Cannot find module '${specifier}' or its corresponding type declarations.`,
        range: rangeAt(source, offset - 1, specifier.length + 2),
        severity: 1,
        source: "vize/types",
      },
    ];
  }
}

function range(line: number, start: number, end: number) {
  return { start: { line, character: start }, end: { line, character: end } };
}

function rangeAt(source: string, offset: number, length: number) {
  const prefix = source.slice(0, offset);
  const line = prefix.split("\n").length - 1;
  const character = offset - (prefix.lastIndexOf("\n") + 1);
  return range(line, character, character + length);
}

function fileName(uri: string): string {
  return path.basename(new URL(uri).pathname);
}
