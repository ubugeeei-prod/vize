import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { isDiagnosticsForUri } from "./support/lsp/assertions.ts";
import { testOutputRoot } from "./support/lsp/paths.ts";
import { LspSession } from "./support/lsp/session.ts";

/**
 * `textDocument/rangeFormatting` ("Format Selection") for `vize lsp`.
 *
 * The handler existed before #3456 but ignored `params.range` and returned the
 * whole-document edit, which is why `documentRangeFormattingProvider` was
 * deliberately not advertised: turning it on as it was would have made Format
 * Selection silently reformat the entire file.
 *
 * The fixture has two equally badly formatted blocks so "only the selected one
 * changes" is observable, and every assertion is a full `assert.deepEqual` on
 * the complete `TextEdit[]` — an assertion on just the selected block would
 * pass on a whole-document edit.
 */

type TextEdit = {
  range: {
    start: { line: number; character: number };
    end: { line: number; character: number };
  };
  newText: string;
};

/**
 * ```text
 * 0  <script setup lang="ts">
 * 1  const   a=1
 * 2  const   b=2
 * 3  </script>
 * 4
 * 5  <template>
 * 6    <div   class="x"   >{{ a }}</div>
 * 7  </template>
 * ```
 */
const SOURCE = `<script setup lang="ts">
const   a=1
const   b=2
</script>

<template>
  <div   class="x"   >{{ a }}</div>
</template>
`;

/** Block *content* spans: from just after the open tag to just before the close tag. */
const SCRIPT_CONTENT = {
  start: { line: 0, character: 24 },
  end: { line: 3, character: 0 },
};
const TEMPLATE_CONTENT = {
  start: { line: 5, character: 10 },
  end: { line: 7, character: 0 },
};

/** What whole-document `vize fmt` makes of each block's content. */
const FORMATTED_SCRIPT = "\nconst a = 1;\nconst b = 2;\n";
const FORMATTED_TEMPLATE = '\n  <div class="x">\n    {{ a }}\n  </div>\n';

async function withDocument(
  run: (
    ask: (range: {
      start: { line: number; character: number };
      end: { line: number; character: number };
    }) => Promise<TextEdit[] | null>,
    askWholeDocument: () => Promise<TextEdit[] | null>,
  ) => Promise<void>,
): Promise<void> {
  const testRootDir = path.join(testOutputRoot, "lsp-range-formatting");
  fs.mkdirSync(testRootDir, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(testRootDir, "workspace-"));
  const session = new LspSession();

  try {
    await session.initialize(workspaceDir, {
      editor: true,
      lint: false,
      typecheck: false,
      formatting: true,
    });

    const filePath = path.join(workspaceDir, "App.vue");
    const uri = pathToFileURL(filePath).href;
    fs.writeFileSync(filePath, SOURCE, "utf8");
    session.notify("textDocument/didOpen", {
      textDocument: { uri, languageId: "vue", version: 1, text: SOURCE },
    });
    await session.waitForNotification("textDocument/publishDiagnostics", (params) =>
      isDiagnosticsForUri(params, uri),
    );

    const options = { tabSize: 2, insertSpaces: true };
    await run(
      (range) =>
        session.request("textDocument/rangeFormatting", {
          textDocument: { uri },
          range,
          options,
        }) as Promise<TextEdit[] | null>,
      () =>
        session.request("textDocument/formatting", {
          textDocument: { uri },
          options,
        }) as Promise<TextEdit[] | null>,
    );
  } finally {
    await session.shutdown();
    fs.rmSync(workspaceDir, { recursive: true, force: true });
    fs.rmSync(testRootDir, { recursive: true, force: true });
  }
}

test("rangeFormatting edits only the blocks the selection touches", async () => {
  await withDocument(async (ask, askWholeDocument) => {
    // Lines 1-2 are entirely inside `<script setup>`. The equally badly
    // formatted template must not appear at all.
    assert.deepEqual(
      await ask({ start: { line: 1, character: 0 }, end: { line: 2, character: 11 } }),
      [{ range: SCRIPT_CONTENT, newText: FORMATTED_SCRIPT }],
    );

    // Line 6 is entirely inside `<template>`; the script is left alone.
    assert.deepEqual(
      await ask({ start: { line: 6, character: 0 }, end: { line: 6, character: 35 } }),
      [{ range: TEMPLATE_CONTENT, newText: FORMATTED_TEMPLATE }],
    );

    // Line 4 is the blank line between the blocks: inside no block content.
    assert.deepEqual(
      await ask({ start: { line: 4, character: 0 }, end: { line: 4, character: 0 } }),
      [],
    );

    // A selection over both blocks edits both, in document order.
    assert.deepEqual(
      await ask({ start: { line: 0, character: 0 }, end: { line: 7, character: 11 } }),
      [
        { range: SCRIPT_CONTENT, newText: FORMATTED_SCRIPT },
        { range: TEMPLATE_CONTENT, newText: FORMATTED_TEMPLATE },
      ],
    );

    // Format Document is still one whole-file replacement, so the two commands
    // are visibly different shapes rather than the same edit under two names.
    const whole = await askWholeDocument();
    assert.equal(whole?.length, 1);
    assert.deepEqual(whole?.[0].range.start, { line: 0, character: 0 });
    assert.deepEqual(whole?.[0].range.end, { line: 8, character: 0 });
  });
});

test("rangeFormatting refuses a range the document does not contain", async () => {
  await withDocument(async (ask) => {
    assert.equal(
      await ask({ start: { line: 0, character: 0 }, end: { line: 900, character: 0 } }),
      null,
    );
  });
});
