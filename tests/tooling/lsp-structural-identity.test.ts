import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { isDiagnosticsForUri, offsetToPosition } from "./support/lsp/assertions.ts";
import { LspSession } from "./support/lsp/session.ts";

test(
  "checker-free references and rename preserve lexical identity across edits",
  {
    skip: process.platform === "win32" ? "POSIX executable backend-start probe" : false,
  },
  async () => {
    for (const newline of ["\n", "\r\n"]) {
      const workspace = fs.mkdtempSync(path.join(os.tmpdir(), "vize lexical "));
      const marker = path.join(workspace, "checker-started");
      const runtime = path.join(workspace, "unexpected-checker");
      fs.writeFileSync(
        runtime,
        `#!${process.execPath}\nrequire('node:fs').writeFileSync(${JSON.stringify(marker)}, 'started'); process.exit(99);\n`,
        { mode: 0o755 },
      );
      fs.writeFileSync(
        path.join(workspace, "vize.config.json"),
        JSON.stringify({
          typeChecker: { corsaPath: runtime },
          lsp: { editor: true, lint: false, typecheck: false, crossFile: false },
        }),
      );
      const session = new LspSession();
      try {
        await session.initialize(workspace, {
          editor: true,
          lint: false,
          typecheck: false,
          crossFile: false,
        });
        const source = `<script setup lang="ts">
const emoji = '😀'; const café = 'fixed';
function inner(café: number) { return café + 1; }
const text = 'café'; // café
const object = { café: 1 };
</script>
<template>{{ café.toUpperCase() }}</template>
<style>/* v-bind(café) */ .x { color: v-bind(café); }</style>
`.replaceAll("\n", newline);
        const uri = pathToFileURL(path.join(workspace, "App.vue")).href;
        session.notify("textDocument/didOpen", {
          textDocument: { uri, languageId: "vue", version: 1, text: source },
        });
        await session.waitForNotification(
          "textDocument/publishDiagnostics",
          (params) => isDiagnosticsForUri(params, uri) && params.version === 1,
        );
        const range = (text: string, needle: string) => {
          const start = text.indexOf(needle);
          assert.notEqual(start, -1, needle);
          return {
            start: offsetToPosition(text, start),
            end: offsetToPosition(text, start + "café".length),
          };
        };
        const request = (method: string, text: string, needle: string, extra = {}) =>
          session.request(method, {
            textDocument: { uri },
            position: range(text, needle).start,
            ...extra,
          });
        const assertIdentity = async (text: string, needles: string[]) => {
          for (const query of needles) {
            for (const includeDeclaration of [true, false]) {
              const expected = needles
                .slice(includeDeclaration ? 0 : 1)
                .map((needle) => ({ uri, range: range(text, needle) }));
              assert.deepEqual(
                await request("textDocument/references", text, query, {
                  context: { includeDeclaration },
                }),
                expected,
              );
            }
            assert.deepEqual(
              await request("textDocument/rename", text, query, { newName: "label" }),
              {
                changes: {
                  [uri]: needles.map((needle) => ({
                    range: range(text, needle),
                    newText: "label",
                  })),
                },
              },
            );
          }
        };
        const outer = ["café =", "café.toUpperCase", "café); }"];
        const inner = ["café: number", "café + 1"];
        await assertIdentity(source, outer);
        await assertIdentity(source, inner);
        for (const needle of [
          "café';",
          "café\n".replaceAll("\n", newline),
          "café: 1",
          "café) */",
        ]) {
          assert.equal(await request("textDocument/prepareRename", source, needle), null);
        }
        const edited = source.replace("café.toUpperCase()", "café + café.trim()");
        session.notify("textDocument/didChange", {
          textDocument: { uri, version: 2 },
          contentChanges: [{ text: edited }],
        });
        await session.waitForNotification(
          "textDocument/publishDiagnostics",
          (params) => isDiagnosticsForUri(params, uri) && params.version === 2,
        );
        await assertIdentity(edited, ["café =", "café + café", "café.trim", "café); }"]);
        await assertIdentity(edited, inner);
        session.notify("textDocument/didChange", {
          textDocument: { uri, version: 3 },
          contentChanges: [{ text: source }],
        });
        await session.waitForNotification(
          "textDocument/publishDiagnostics",
          (params) => isDiagnosticsForUri(params, uri) && params.version === 3,
        );
        await assertIdentity(source, outer);
      } finally {
        await session.shutdown();
        assert.equal(fs.existsSync(marker), false, "disabled native checker must never start");
        fs.rmSync(workspace, { recursive: true, force: true });
      }
    }
  },
);
