import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { isDiagnosticsForUri, offsetToPosition } from "./support/lsp/assertions.ts";
import { root } from "./support/lsp/paths.ts";
import type { PublishDiagnosticsParams } from "./support/lsp/protocol.ts";
import { LspSession } from "./support/lsp/session.ts";
import { consumer, createN8nWorkspace } from "./support/n8n-authored-editor.ts";
import { locations, sortLocations } from "./support/real-project-lsp-authored-utils.ts";
import {
  requireTypecheckDependency,
  resolveTypecheckRuntime,
} from "./support/typecheck-dependency.ts";

test("n8n original component and TS barrel retain navigation and dirty/restore type diagnostics", async (t) => {
  const corsaPath = requireTypecheckDependency(
    t,
    resolveTypecheckRuntime(root),
    "the actual typechecker for n8n authored acceptance",
    "typechecker is unavailable",
  );
  if (!corsaPath) return;
  const { workspace, source } = createN8nWorkspace(corsaPath);
  const session = new LspSession();
  const uri = (file: string) => pathToFileURL(path.join(workspace, file)).href;
  const bindingUri = uri("NullEmptyCellRenderer.vue");
  const childUri = uri("N8nBlockUi/BlockUi.vue");
  const importerUri = uri("N8nBlockUi/index.ts");
  const appUri = uri("App.vue");
  const tokenRange = (text: string, anchor: string, token: string) => {
    const anchorOffset = text.indexOf(anchor);
    assert.notEqual(anchorOffset, -1, `missing authored anchor: ${anchor}`);
    assert.equal(text.split(anchor).length, 2, `ambiguous authored anchor: ${anchor}`);
    const tokenOffset = anchor.indexOf(token);
    assert.notEqual(tokenOffset, -1);
    const start = anchorOffset + tokenOffset;
    return {
      start: offsetToPosition(text, start),
      end: offsetToPosition(text, start + token.length),
    };
  };
  const declaration = tokenRange(source, "const props =", "props");
  const usage = tokenRange(source, "{{ props.params.value }}", "props");
  const query = (document: string, position: { line: number; character: number }) => ({
    textDocument: { uri: document },
    position,
  });
  const open = (file: string, languageId: string, text: string) =>
    session.notify("textDocument/didOpen", {
      textDocument: { uri: file, languageId, text, version: 1 },
    });
  const diagnostics = async (version: number, hasErrors: boolean) =>
    (await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) =>
        isDiagnosticsForUri(params, bindingUri) &&
        params.version === version &&
        (hasErrors ? params.diagnostics.length > 0 : params.diagnostics.length === 0),
      120_000,
    )) as PublishDiagnosticsParams;
  try {
    await session.initialize(workspace, { editor: true, lint: false, typecheck: true });
    open(bindingUri, "vue", source);
    assert.deepEqual((await diagnostics(1, false)).diagnostics, []);
    const definition = locations(
      await session.request("textDocument/definition", query(bindingUri, usage.start)),
      "n8n props definition",
    );
    assert.deepEqual(definition, [{ uri: bindingUri, range: declaration }]);
    const references = locations(
      await session.request("textDocument/references", {
        ...query(bindingUri, usage.start),
        context: { includeDeclaration: true },
      }),
      "n8n props references",
    );
    assert.deepEqual(
      sortLocations(references),
      sortLocations([
        { uri: bindingUri, range: declaration },
        { uri: bindingUri, range: usage },
      ]),
    );
    assert.deepEqual(
      locations(
        await session.request("textDocument/references", {
          ...query(bindingUri, usage.start),
          context: { includeDeclaration: false },
        }),
        "n8n props references without declaration",
      ),
      [{ uri: bindingUri, range: usage }],
    );
    for (const includeDeclaration of [true, false]) {
      assert.deepEqual(
        sortLocations(
          locations(
            await session.request("textDocument/references", {
              ...query(bindingUri, declaration.start),
              context: { includeDeclaration },
            }),
            "n8n script props references",
          ),
        ),
        sortLocations(
          includeDeclaration
            ? [
                { uri: bindingUri, range: declaration },
                { uri: bindingUri, range: usage },
              ]
            : [{ uri: bindingUri, range: usage }],
        ),
      );
    }
    open(childUri, "vue", fs.readFileSync(path.join(workspace, "N8nBlockUi/BlockUi.vue"), "utf8"));
    open(
      importerUri,
      "typescript",
      fs.readFileSync(path.join(workspace, "N8nBlockUi/index.ts"), "utf8"),
    );
    open(appUri, "vue", consumer);
    const tag = tokenRange(consumer, "<N8nBlockUi :", "N8nBlockUi");
    const boundary = locations(
      await session.request("textDocument/definition", query(appUri, tag.start)),
      "n8n TS barrel component definition",
    );
    assert.deepEqual(boundary, [
      {
        uri: childUri,
        range: { start: { line: 0, character: 0 }, end: { line: 0, character: 0 } },
      },
    ]);
    const dirty = source.replace("props.params.value", "props.params.missing");
    assert.notEqual(dirty, source);
    session.notify("textDocument/didChange", {
      textDocument: { uri: bindingUri, version: 2 },
      contentChanges: [{ text: dirty }],
    });
    const broken = await diagnostics(2, true);
    assert.deepEqual(broken.diagnostics, [
      {
        code: 2339,
        message: "Property 'missing' does not exist on type '{ value: string; }'.",
        range: tokenRange(dirty, "props.params.missing", "missing"),
        severity: 1,
        source: "vize/types",
      },
    ]);
    assert.equal(
      fs.readFileSync(path.join(workspace, "NullEmptyCellRenderer.vue"), "utf8"),
      source,
      "unsaved editor changes must leave original source bytes intact",
    );
    session.notify("textDocument/didChange", {
      textDocument: { uri: bindingUri, version: 3 },
      contentChanges: [{ text: source }],
    });
    assert.deepEqual((await diagnostics(3, false)).diagnostics, []);
    assert.deepEqual(
      locations(
        await session.request("textDocument/definition", query(bindingUri, usage.start)),
        "restored n8n props definition",
      ),
      definition,
    );
  } finally {
    await session.shutdown();
    fs.rmSync(workspace, { recursive: true, force: true });
  }
});
