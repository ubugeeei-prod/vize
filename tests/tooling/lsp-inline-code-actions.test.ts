import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { isDiagnosticsForUri, offsetToPosition } from "./support/lsp/assertions.ts";
import { testOutputRoot } from "./support/lsp/paths.ts";
import type { LspRange, PublishDiagnosticsParams } from "./support/lsp/protocol.ts";
import { LspSession } from "./support/lsp/session.ts";

const cases = [
  { name: "inline", prefix: "<template>", suffix: "</template>", newline: "\n", indent: "" },
  {
    name: "unicode-prefix",
    prefix: "<!-- \u{1f600} --><template>",
    suffix: "</template>",
    newline: "\n",
    indent: "",
  },
  {
    name: "multiline-header",
    prefix: "<template\n  >",
    suffix: "</template>",
    newline: "\n",
    indent: "",
  },
  {
    name: "crlf",
    prefix: "<!-- \u{1f600} -->\r\n<template>\r\n",
    suffix: "\r\n</template>\r\n",
    newline: "\r\n",
    indent: "\t",
  },
];

for (const fixture of cases) {
  test(`LSP quickfix and suppression preserve authored ${fixture.name} template positions`, async () => {
    fs.mkdirSync(testOutputRoot, { recursive: true });
    const workspace = fs.mkdtempSync(path.join(testOutputRoot, "inline-actions-"));
    const session = new LspSession();
    const source = `${fixture.prefix}${fixture.indent}<div title="\u{1f600}"  class="a">text</div>${fixture.suffix}`;
    const file = path.join(workspace, "Inline.vue");
    const uri = pathToFileURL(file).href;
    const gap = source.indexOf("  class=");
    const range: LspRange = {
      start: offsetToPosition(source, gap),
      end: offsetToPosition(source, gap + 2),
    };
    const insert = offsetToPosition(source, fixture.prefix.length);
    const suppression = `${fixture.indent}<!-- @vize:forget vue/no-multi-spaces -->${fixture.newline}`;

    const publish = (version: number) =>
      session.waitForNotification(
        "textDocument/publishDiagnostics",
        (params) => isDiagnosticsForUri(params, uri) && params.version === version,
      ) as Promise<PublishDiagnosticsParams>;
    const request = (requestedRange: LspRange) =>
      session.request("textDocument/codeAction", {
        textDocument: { uri },
        range: requestedRange,
        context: { diagnostics: [], only: ["quickfix"] },
      });

    try {
      fs.writeFileSync(file, source);
      await session.initialize(workspace, {
        editor: true,
        lint: true,
        codeActions: true,
        typecheck: false,
      });
      session.notify("textDocument/didOpen", {
        textDocument: { uri, languageId: "vue", version: 1, text: source },
      });
      const initial = await publish(1);
      assert.deepEqual(
        initial.diagnostics.map(({ code, range }) => ({ code, range })),
        [{ code: "vue/no-multi-spaces", range }],
      );
      assert.deepEqual(await request(range), [
        {
          title: "Fix: Replace multiple spaces with single space",
          kind: "quickfix",
          edit: { changes: { [uri]: [{ range, newText: " " }] } },
          isPreferred: true,
        },
        {
          title: "Suppress with @vize:forget (vue/no-multi-spaces)",
          kind: "quickfix",
          edit: {
            changes: { [uri]: [{ range: { start: insert, end: insert }, newText: suppression }] },
          },
          isPreferred: false,
        },
      ]);
      const outside = offsetToPosition(source, source.indexOf("<template") + 1);
      assert.equal(await request({ start: outside, end: outside }), null);

      session.notify("textDocument/didChange", {
        textDocument: { uri, version: 2 },
        contentChanges: [{ range, text: " " }],
      });
      assert.deepEqual((await publish(2)).diagnostics, []);
      assert.equal(await request(range), null);

      session.notify("textDocument/didChange", {
        textDocument: { uri, version: 3 },
        contentChanges: [{ text: source }],
      });
      assert.deepEqual((await publish(3)).diagnostics, initial.diagnostics);
      session.notify("textDocument/didChange", {
        textDocument: { uri, version: 4 },
        contentChanges: [{ range: { start: insert, end: insert }, text: suppression }],
      });
      assert.deepEqual((await publish(4)).diagnostics, []);
    } finally {
      await session.shutdown();
      fs.rmSync(workspace, { recursive: true, force: true });
    }
  });
}
