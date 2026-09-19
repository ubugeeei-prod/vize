import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { isDiagnosticsForUri } from "./support/lsp/assertions.ts";
import { LspSession } from "./support/lsp/session.ts";
import { check, workspace } from "./support/upstream/vue-language-tools.ts";

test(
  "CLI and LSP preserve external declarations, including unsaved dependency edits",
  { timeout: 120_000 },
  async () => {
    const directory = workspace("external-declaration-contract-");
    const app = path.join(directory, "app");
    fs.mkdirSync(app);
    fs.writeFileSync(
      path.join(app, "tsconfig.json"),
      JSON.stringify({
        compilerOptions: {
          strict: true,
          noEmit: true,
          moduleResolution: "bundler",
          module: "esnext",
          skipLibCheck: true,
        },
        include: ["*.vue"],
      }),
    );
    const declaration = "export declare function expectString(value: string): void;";
    const dependency = path.join(directory, "shared.d.ts");
    fs.writeFileSync(dependency, declaration);
    const source =
      '<script setup lang="ts">\nimport { expectString } from "../shared";\nexpectString(1);\n</script>';
    const file = path.join(app, "App.vue");
    fs.writeFileSync(file, source);
    const uri = pathToFileURL(file).href;
    const dependencyUri = pathToFileURL(dependency).href;
    const session = new LspSession();
    const diagnostic = async (version: number, code?: number) => {
      const result = await session.waitForNotification(
        "textDocument/publishDiagnostics",
        (p) =>
          isDiagnosticsForUri(p, uri) &&
          p.version === version &&
          (code == null
            ? p.diagnostics.length === 0
            : p.diagnostics.some((d) => Number(d.code) === code)),
      );
      assert.ok(isDiagnosticsForUri(result, uri));
      assert.deepEqual(
        result.diagnostics.filter((d) => d.severity === 1).map((d) => Number(d.code)),
        code == null ? [] : [code],
      );
    };
    try {
      assert.deepEqual(
        (await check(app)).map((d) => ({ code: d.code, line: d.line, column: d.column })),
        [{ code: 2345, line: 3, column: 14 }],
      );
      await session.initialize(app, { editor: true, typecheck: true, lint: false });
      session.notify("textDocument/didOpen", {
        textDocument: { uri, languageId: "vue", version: 1, text: source },
      });
      await diagnostic(1, 2345);
      session.notify("textDocument/didChange", {
        textDocument: { uri, version: 2 },
        contentChanges: [{ text: source.replace("expectString(1)", "expectString('valid')") }],
      });
      await diagnostic(2);
      session.notify("textDocument/didOpen", {
        textDocument: {
          uri: dependencyUri,
          languageId: "typescript",
          version: 1,
          text: declaration.replace("value: string", "value: number"),
        },
      });
      await diagnostic(2, 2345);
      session.notify("textDocument/didChange", {
        textDocument: { uri: dependencyUri, version: 2 },
        contentChanges: [{ text: declaration }],
      });
      await diagnostic(2);
      session.notify("textDocument/didChange", {
        textDocument: { uri: dependencyUri, version: 3 },
        contentChanges: [{ text: declaration.replace("value: string", "value: number") }],
      });
      await diagnostic(2, 2345);
      session.notify("textDocument/didClose", { textDocument: { uri: dependencyUri } });
      await diagnostic(2);
    } finally {
      await session.shutdown();
      fs.rmSync(directory, { recursive: true, force: true });
    }
  },
);
