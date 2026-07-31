import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { isDiagnosticsForUri } from "./support/lsp/assertions.ts";
import { testOutputRoot } from "./support/lsp/paths.ts";
import { LspSession } from "./support/lsp/session.ts";

/**
 * `textDocument/documentColor` and `textDocument/colorPresentation` for
 * `vize lsp` (#3456).
 *
 * `@vue/language-server` 3.3.8 advertises `colorProvider: true`; Maestro
 * advertised nothing and answered `-32601 Method not found`, so a `.vue` file
 * was the one place in a project where the colour picker did not appear.
 *
 * Both responses are asserted in full with `assert.deepEqual`: a must-include
 * check would pass on a scanner that also painted swatches over the script body
 * or a `:style` expression, which is the failure mode that matters here.
 */

type ColorInformation = {
  range: {
    start: { line: number; character: number };
    end: { line: number; character: number };
  };
  color: { red: number; green: number; blue: number; alpha: number };
};

type ColorPresentation = { label: string; textEdit?: unknown };

/**
 * ```text
 * 0  <script setup lang="ts">
 * 1  const tag = '#123456'
 * 2  </script>
 * 3
 * 4  <template>
 * 5    <div style="color: #f00">#00ff00</div>
 * 6    <span :style="{ color: '#0000ff' }" />
 * 7  </template>
 * 8
 * 9  <style scoped>
 * 10 .a { background: rgba(0, 0, 255, 0.5) }
 * 11 .b { color: hsl(120 100% 25%) }
 * 12 </style>
 * ```
 */
const SOURCE = `<script setup lang="ts">
const tag = '#123456'
</script>

<template>
  <div style="color: #f00">#00ff00</div>
  <span :style="{ color: '#0000ff' }" />
</template>

<style scoped>
.a { background: rgba(0, 0, 255, 0.5) }
.b { color: hsl(120 100% 25%) }
</style>
`;

async function withDocument(
  run: (
    colors: () => Promise<ColorInformation[]>,
    presentations: (color: ColorInformation["color"]) => Promise<ColorPresentation[]>,
  ) => Promise<void>,
): Promise<void> {
  const testRootDir = path.join(testOutputRoot, "lsp-document-color");
  fs.mkdirSync(testRootDir, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(testRootDir, "workspace-"));
  const session = new LspSession();

  try {
    await session.initialize(workspaceDir, { editor: true, lint: false, typecheck: false });

    const filePath = path.join(workspaceDir, "App.vue");
    const uri = pathToFileURL(filePath).href;
    fs.writeFileSync(filePath, SOURCE, "utf8");
    session.notify("textDocument/didOpen", {
      textDocument: { uri, languageId: "vue", version: 1, text: SOURCE },
    });
    await session.waitForNotification("textDocument/publishDiagnostics", (params) =>
      isDiagnosticsForUri(params, uri),
    );

    await run(
      () =>
        session.request("textDocument/documentColor", {
          textDocument: { uri },
        }) as Promise<ColorInformation[]>,
      (color) =>
        session.request("textDocument/colorPresentation", {
          textDocument: { uri },
          color,
          range: { start: { line: 5, character: 21 }, end: { line: 5, character: 25 } },
        }) as Promise<ColorPresentation[]>,
    );
  } finally {
    await session.shutdown();
    fs.rmSync(workspaceDir, { recursive: true, force: true });
    fs.rmSync(testRootDir, { recursive: true, force: true });
  }
}

test("documentColor reports every supported CSS colour and nothing else", async () => {
  await withDocument(async (colors) => {
    // Exactly three: the static attribute, rgba(), and hsl() in `<style>`.
    // Deliberately absent, and the reason this is a full-list assertion:
    //   - `'#123456'` on line 1 is a string in the script body,
    //   - `#00ff00` on line 5 is a text node,
    //   - `'#0000ff'` on line 6 is inside a bound `:style` expression.
    assert.deepEqual(await colors(), [
      {
        range: {
          start: { line: 5, character: 21 },
          end: { line: 5, character: 25 },
        },
        color: { red: 1, green: 0, blue: 0, alpha: 1 },
      },
      {
        range: {
          start: { line: 10, character: 17 },
          end: { line: 10, character: 37 },
        },
        color: { red: 0, green: 0, blue: 1, alpha: 0.5 },
      },
      {
        range: {
          start: { line: 11, character: 12 },
          end: { line: 11, character: 29 },
        },
        color: { red: 0, green: 0.5, blue: 0, alpha: 1 },
      },
    ]);
  });
});

test("colorPresentation offers hex, rgb, and hsl notation", async () => {
  await withDocument(async (_colors, presentations) => {
    assert.deepEqual(await presentations({ red: 1, green: 0, blue: 0, alpha: 1 }), [
      { label: "#ff0000" },
      { label: "rgb(255, 0, 0)" },
      { label: "hsl(0 100% 50%)" },
    ]);

    // A translucent colour switches to the 8-digit hex and `rgba()`, with the
    // alpha written the way CSS writes it.
    assert.deepEqual(await presentations({ red: 0, green: 0, blue: 1, alpha: 0.5 }), [
      { label: "#0000ff80" },
      { label: "rgba(0, 0, 255, 0.5)" },
      { label: "hsl(240 100% 50% / 0.5)" },
    ]);
  });
});
