import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { check, workspace } from "./support/upstream/vue-language-tools.ts";
import { offsetToPosition } from "./support/lsp/assertions.ts";
import { LspSession } from "./support/lsp/session.ts";

type Position = { line: number; character: number };
type Edit = { range: { start: Position; end: Position }; newText: string };

const source = `<!-- @strictCssModules true -->
<script setup lang="ts">
import { useCssModule as stylesFor } from 'vue';
const colors = stylesFor();
const nested = () => stylesFor().root;
function shadow(stylesFor: () => { own: 1 }) { return stylesFor().own; }
void shadow(() => ({ own: 1 })); void nested();
// @ts-expect-error
colors.absent;
</script>
<template><div :class="colors.root" /></template>
<style module>.root { color: red; }</style>`;

test(
  "CSS module signatures preserve native hover, rename and save-time responsiveness",
  { timeout: 60_000 },
  async () => {
    const directory = workspace("lsp-css-contract-");
    const session = new LspSession();
    try {
      fs.writeFileSync(
        path.join(directory, "tsconfig.json"),
        JSON.stringify({
          compilerOptions: {
            strict: true,
            module: "ESNext",
            moduleResolution: "Bundler",
            noEmit: true,
          },
          include: ["*.vue"],
        }),
      );
      const file = path.join(directory, "App.vue");
      fs.writeFileSync(file, source);
      assert.deepEqual(await check(directory), []);
      await session.initialize(directory, {
        editor: true,
        lint: false,
        typecheck: true,
        semanticTokens: true,
      });
      const uri = pathToFileURL(file).href;
      session.notify("textDocument/didOpen", {
        textDocument: { uri, languageId: "vue", version: 1, text: source },
      });
      const offsets = [
        source.indexOf("stylesFor"),
        source.indexOf("stylesFor()"),
        source.indexOf("stylesFor().root"),
      ];
      const expected = offsets.map((offset) => ({
        range: {
          start: offsetToPosition(source, offset),
          end: offsetToPosition(source, offset + "stylesFor".length),
        },
        newText: "moduleStyles",
      }));
      const sorted = (edits: Edit[]) =>
        edits.sort(
          (a, b) =>
            a.range.start.line - b.range.start.line ||
            a.range.start.character - b.range.start.character,
        );
      for (const offset of offsets) {
        const result = (await session.request("textDocument/rename", {
          textDocument: { uri },
          position: offsetToPosition(source, offset + 2),
          newName: "moduleStyles",
        })) as { changes: Record<string, Edit[]> };
        assert.deepEqual(sorted(result.changes[uri]), expected, `rename at ${offset}`);
      }
      const hover = await session.request("textDocument/hover", {
        textDocument: { uri },
        position: offsetToPosition(source, source.indexOf("colors.root") + "colors.".length),
      });
      assert.match(JSON.stringify(hover), /string/);
      for (let version = 2; version <= 9; version++) {
        const text = `${source}\n<!-- saved revision ${version} -->`;
        fs.writeFileSync(file, text);
        session.notify("textDocument/didChange", {
          textDocument: { uri, version },
          contentChanges: [{ text }],
        });
        session.notify("textDocument/didSave", { textDocument: { uri } });
        const tokens = (await session.request("textDocument/semanticTokens/full", {
          textDocument: { uri },
        })) as { data: number[] };
        assert.ok(tokens.data.length > 0);
      }
    } finally {
      await session.shutdown();
      fs.rmSync(directory, { recursive: true, force: true });
    }
  },
);
