import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import ts from "typescript";
import { pathToFileURL } from "node:url";
import { isDiagnosticsForUri, offsetToPosition } from "../lsp/assertions.ts";
import type { LspDiagnostic, LspRange } from "../lsp/protocol.ts";
import { LspSession } from "../lsp/session.ts";
import { patternRoot, workspace } from "./vue-language-tools.ts";

export function referenceLspSources(): Map<string, string> {
  const source = fs.readFileSync(
    path.join(
      patternRoot,
      "upstream/language-tools/packages/language-server/tests/patternedTemplates.spec.ts",
    ),
    "utf8",
  );
  const ast = ts.createSourceFile("reference.ts", source, ts.ScriptTarget.Latest, true);
  const results = new Map<string, string>();
  function literal(node: ts.Node, name: string): string | undefined {
    if (
      ts.isVariableDeclaration(node) &&
      ts.isIdentifier(node.name) &&
      node.name.text === name &&
      node.initializer &&
      ts.isNoSubstitutionTemplateLiteral(node.initializer)
    )
      return node.initializer.text;
    return ts.forEachChild(node, (child) => literal(child, name));
  }
  results.set("source", literal(ast, "source")!);
  for (const statement of ast.statements) {
    if (!ts.isExpressionStatement(statement) || !ts.isCallExpression(statement.expression))
      continue;
    const call = statement.expression;
    const title = call.arguments[0];
    if (!title || !ts.isStringLiteral(title)) continue;
    const text = literal(call, "text");
    if (text) results.set(title.text, text);
  }
  assert.equal(results.size, 5, "all literal fixtures in the upstream LSP suite must be ported");
  return results;
}

export class PatternSession {
  readonly directory = workspace("pattern-lsp-reference-");
  readonly file = path.join(this.directory, "App.vue");
  readonly uri = pathToFileURL(this.file).href;
  readonly session = new LspSession();
  source = "";
  version = 0;
  lastRenameEdits: Array<{ range: LspRange; newText: string }> = [];

  async initialize(): Promise<void> {
    fs.writeFileSync(
      path.join(this.directory, "tsconfig.json"),
      JSON.stringify({
        compilerOptions: {
          strict: true,
          skipLibCheck: true,
          target: "ESNext",
          module: "ESNext",
          moduleResolution: "Bundler",
        },
        include: ["*.vue"],
      }),
    );
    fs.writeFileSync(
      path.join(this.directory, "vize.config.json"),
      JSON.stringify({
        experimentals: { patternedTemplate: true },
        lsp: { editor: true, typecheck: true, lint: false },
      }),
    );
    fs.writeFileSync(this.file, "<template />");
    await this.session.initialize(this.directory, { editor: true, typecheck: true, lint: false });
  }

  async update(source: string): Promise<LspDiagnostic[]> {
    this.source = source;
    this.version++;
    if (this.version === 1)
      this.session.notify("textDocument/didOpen", {
        textDocument: { uri: this.uri, languageId: "vue", version: this.version, text: source },
      });
    else
      this.session.notify("textDocument/didChange", {
        textDocument: { uri: this.uri, version: this.version },
        contentChanges: [{ text: source }],
      });
    const result = await this.session.waitForNotification(
      "textDocument/publishDiagnostics",
      (p) => isDiagnosticsForUri(p, this.uri) && p.version === this.version,
    );
    assert.ok(isDiagnosticsForUri(result, this.uri));
    return result.diagnostics.filter((d) => d.severity === 1 || d.severity === 2);
  }

  request(method: string, offset: number, extra = {}): Promise<unknown> {
    return this.session.request(`textDocument/${method}`, {
      textDocument: { uri: this.uri },
      position: offsetToPosition(this.source, offset),
      ...extra,
    });
  }

  async rename(offset: number): Promise<number[]> {
    const result = (await this.request("rename", offset, { newName: "renamed" })) as {
      changes?: Record<string, Array<{ range: LspRange }>>;
      documentChanges?: Array<{ textDocument: { uri: string }; edits: Array<{ range: LspRange }> }>;
    };
    assert.ok(result, "binding must support rename");
    const changes =
      result.changes ??
      Object.fromEntries((result.documentChanges ?? []).map((c) => [c.textDocument.uri, c.edits]));
    assert.deepEqual(Object.keys(changes), [this.uri]);
    this.lastRenameEdits = changes[this.uri] as Array<{ range: LspRange; newText: string }>;
    return changes[this.uri]
      .map(({ range }) => {
        const lines = this.source.split("\n");
        return (
          lines.slice(0, range.start.line).reduce((n, line) => n + line.length + 1, 0) +
          range.start.character
        );
      })
      .sort((a, b) => a - b);
  }

  async close(): Promise<void> {
    await this.session.shutdown();
    fs.rmSync(this.directory, { recursive: true, force: true });
  }
}
