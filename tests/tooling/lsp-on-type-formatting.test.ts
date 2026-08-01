import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { isDiagnosticsForUri } from "./support/lsp/assertions.ts";
import { testOutputRoot } from "./support/lsp/paths.ts";
import { LspSession } from "./support/lsp/session.ts";

/**
 * `textDocument/onTypeFormatting` ("Format On Type") for `vize lsp`.
 *
 * The server advertises `;`, `}` and `\n` as triggers — the same set
 * `@vue/language-server` advertises (#3456) — so an editor with
 * `editor.formatOnType` on calls this on those keystrokes.
 *
 * Because it fires under the caret, the handler is only allowed to move a
 * line's leading whitespace, and only on the requested line. The fixture below
 * therefore carries two independently mis-indented lines, one in `<script>` and
 * one in `<template>`, so "it re-indented exactly the line I asked about" is
 * observable: every assertion is a full `assert.deepEqual` on the complete
 * `TextEdit[]`, which a whole-document edit or a second stray line would fail.
 */

type TextEdit = {
  range: {
    start: { line: number; character: number };
    end: { line: number; character: number };
  };
  newText: string;
};

/**
 * A `vize fmt`-clean SFC — the state a file is in while `editor.formatOnType`
 * keeps up with the typist — apart from the closing brace of a TypeScript
 * function *and* of a CSS rule, each left one level too deep. Two embedded
 * languages, one trigger character, and a `<template>` in between that must
 * stay untouched.
 *
 * ```text
 *  0  <script setup lang="ts">
 *  1  function greet() {
 *  2    const name = "vize";
 *  3    return name;
 *  4    }                      <- `}` typed at the wrong indent
 *  5  </script>
 *  6
 *  7  <template>
 *  8    <p>
 *  9      hello
 * 10    </p>
 * 11  </template>
 * 12
 * 13  <style scoped>
 * 14  .box {
 * 15    color: red;
 * 16    }                      <- and again, in CSS
 * 17  </style>
 * ```
 */
const SOURCE = `<script setup lang="ts">
function greet() {
  const name = "vize";
  return name;
  }
</script>

<template>
  <p>
    hello
  </p>
</template>

<style scoped>
.box {
  color: red;
  }
</style>
`;

async function withDocument(
  run: (
    ask: (line: number, character: number, ch: string) => Promise<TextEdit[] | null>,
    askWholeDocument: () => Promise<TextEdit[] | null>,
  ) => Promise<void>,
): Promise<void> {
  const testRootDir = path.join(testOutputRoot, "lsp-on-type-formatting");
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
      (line, character, ch) =>
        session.request("textDocument/onTypeFormatting", {
          textDocument: { uri },
          position: { line, character },
          ch,
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

test("onTypeFormatting re-indents only the line the character was typed on", async () => {
  await withDocument(async (ask, askWholeDocument) => {
    // Closing `greet()` on line 4: the stray two-space indent goes, and the
    // equally mis-indented CSS line 16 does not come along.
    assert.deepEqual(await ask(4, 3, "}"), [
      {
        range: { start: { line: 4, character: 0 }, end: { line: 4, character: 2 } },
        newText: "",
      },
    ]);

    // The same keystroke closing the CSS rule, with the script left alone.
    assert.deepEqual(await ask(16, 3, "}"), [
      {
        range: { start: { line: 16, character: 0 }, end: { line: 16, character: 2 } },
        newText: "",
      },
    ]);

    // A line that is already correctly indented yields nothing, even though the
    // document as a whole is unformatted.
    assert.deepEqual(await ask(2, 22, ";"), []);

    // Format Document, by contrast, is one whole-file replacement — proof the
    // per-line answers above are not that edit under another name.
    const whole = await askWholeDocument();
    assert.equal(whole?.length, 1);
    assert.deepEqual(whole?.[0].range.start, { line: 0, character: 0 });
    assert.deepEqual(whole?.[0].range.end, { line: 18, character: 0 });
  });
});

test("onTypeFormatting refuses a line the document does not have", async () => {
  await withDocument(async (ask) => {
    assert.equal(await ask(900, 0, "}"), null);
  });
});
