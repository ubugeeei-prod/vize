import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { isDiagnosticsForUri } from "./support/lsp/assertions.ts";
import { root, testOutputRoot } from "./support/lsp/paths.ts";
import type { PublishDiagnosticsParams } from "./support/lsp/protocol.ts";
import { LspSession } from "./support/lsp/session.ts";

const corpus = path.join(
  root,
  "tests/_fixtures/differential/lsp/workspace-tsconfig-inheritance-9842",
);

test("package SFC inherits the workspace tsconfig ambient declarations", async () => {
  fs.mkdirSync(testOutputRoot, { recursive: true });
  const directory = fs.mkdtempSync(path.join(testOutputRoot, "workspace-tsconfig-9842-"));
  const files = new Map([
    ["package.json.txt", "package.json"],
    ["tsconfig.json.txt", "tsconfig.json"],
    ["globals.d.ts.txt", "types/globals.d.ts"],
    ["package-ui.json.txt", "packages/ui/package.json"],
    ["MyFooter.vue.txt", "packages/ui/src/MyFooter.vue"],
  ]);
  for (const [source, destination] of files) {
    const output = path.join(directory, destination);
    fs.mkdirSync(path.dirname(output), { recursive: true });
    fs.copyFileSync(path.join(corpus, source), output);
  }

  const session = new LspSession();
  const file = path.join(directory, "packages/ui/src/MyFooter.vue");
  const uri = pathToFileURL(file).href;
  const source = fs.readFileSync(file, "utf8");
  try {
    await session.initialize(directory, { editor: true, lint: false, typecheck: true });
    session.notify("textDocument/didOpen", {
      textDocument: { uri, languageId: "vue", version: 1, text: source },
    });
    const publish = (await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) => isDiagnosticsForUri(params, uri) && params.version === 1,
    )) as PublishDiagnosticsParams;
    assert.deepEqual(
      publish.diagnostics,
      [],
      `workspace globals must resolve __APP_VERSION__: ${JSON.stringify(publish.diagnostics)}`,
    );

    const folding = await session.request("textDocument/foldingRange", {
      textDocument: { uri },
    });
    assert.deepEqual(
      folding,
      JSON.parse(fs.readFileSync(path.join(corpus, "response.expected.json"), "utf8")),
    );
  } finally {
    await session.shutdown();
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
