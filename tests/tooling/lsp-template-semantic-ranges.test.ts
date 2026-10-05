import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { createHash } from "node:crypto";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { LspSession } from "./support/lsp/session.ts";
import { offsetToPosition } from "./support/lsp/assertions.ts";
import { semanticTokensLegend } from "./support/lsp/authored-ranges.ts";

const corpus = new URL(
  "../_fixtures/differential/lsp/template-semantic-ranges-8014/",
  import.meta.url,
);
const manifest = JSON.parse(fs.readFileSync(new URL("manifest.json", corpus), "utf8"));
type Token = [number, number, number, number, number];
assert.equal(
  createHash("sha256")
    .update(fs.readFileSync(new URL(manifest.expectedTokens.path, corpus)))
    .digest("hex"),
  manifest.expectedTokens.sha256,
);
const expected = JSON.parse(
  fs.readFileSync(new URL("expected-tokens.json", corpus), "utf8"),
) as Record<string, Token[]>;

function encode(tokens: Token[]): number[] {
  let line = 0,
    start = 0;
  return tokens.flatMap(([nextLine, nextStart, length, kind, modifiers]) => {
    const value = [
      nextLine - line,
      nextLine === line ? nextStart - start : nextStart,
      length,
      kind,
      modifiers,
    ];
    line = nextLine;
    start = nextStart;
    return value;
  });
}

async function initialize(session: LspSession, workspace: string, typecheck: boolean) {
  const uri = pathToFileURL(workspace).href;
  const result = await session.request("initialize", {
    processId: process.pid,
    rootUri: uri,
    capabilities: {
      textDocument: {
        semanticTokens: {
          requests: { full: true, range: true },
          formats: ["relative"],
          tokenTypes: ["variable", "comment", "string"],
          tokenModifiers: [],
          multilineTokenSupport: false,
          overlappingTokenSupport: false,
        },
      },
    },
    initializationOptions: { editor: true, lint: false, typecheck, semanticTokens: true },
    workspaceFolders: [{ uri, name: "original-8014" }],
  });
  session.notify("initialized", {});
  const legend = semanticTokensLegend(result);
  assert.equal(legend.tokenTypes[17], "comment");
  assert.equal(legend.tokenTypes[18], "string");
  return legend;
}

async function assertTokens(session: LspSession, uri: string, source: string, tokens: Token[]) {
  assert.deepEqual(
    await session.request("textDocument/semanticTokens/full", { textDocument: { uri } }),
    { data: encode(tokens) },
  );
  const lines = source.split("\n").map((line) => line.replace(/\r$/, ""));
  for (const [line, start, length] of tokens) {
    assert.ok(length > 0 && start + length <= lines[line].length, "single-line UTF-16 span");
  }
  const range = {
    start: { line: tokens[0][0], character: tokens[0][1] },
    end: { line: lines.length, character: 0 },
  };
  assert.deepEqual(
    await session.request("textDocument/semanticTokens/range", { textDocument: { uri }, range }),
    { data: encode(tokens) },
  );
}

for (const typecheck of [false, true]) {
  test(
    `original8014 complete single-line tokens and comment navigation (typecheck=${typecheck})`,
    { timeout: 120_000 },
    async () => {
      const workspace = fs.mkdtempSync(path.join(os.tmpdir(), "vize-lsp-8014-"));
      const session = new LspSession();
      try {
        await initialize(session, workspace, typecheck);
        for (const file of manifest.files) {
          const bytes = fs.readFileSync(new URL(file.path, corpus));
          assert.equal(bytes.length, file.bytes);
          assert.equal(createHash("sha256").update(bytes).digest("hex"), file.sha256);
          const source = bytes.toString("utf8"),
            pathname = path.join(workspace, file.runtimePath);
          fs.writeFileSync(pathname, bytes);
          const uri = pathToFileURL(pathname).href;
          session.notify("textDocument/didOpen", {
            textDocument: { uri, languageId: "vue", version: 1, text: source },
          });
          await assertTokens(session, uri, source, expected[file.runtimePath]);
          if (file.runtimePath !== "App.vue") continue;
          for (const marker of [
            "// always",
            "close here",
            "/* the count",
            "count value",
            "/* explanation",
            "explanation text",
          ]) {
            const offset = source.indexOf(marker) + 2;
            const params = { textDocument: { uri }, position: offsetToPosition(source, offset) };
            assert.equal(await session.request("textDocument/definition", params), null, marker);
            assert.equal(await session.request("textDocument/hover", params), null, marker);
          }
          for (const [marker, line, character, length] of [
            ["close()\n", 1, 9, 5],
            ["count /*", 2, 6, 5],
          ] as const) {
            const offset = source.indexOf(marker, source.indexOf("<template>")) + 2;
            assert.deepEqual(
              await session.request("textDocument/definition", {
                textDocument: { uri },
                position: offsetToPosition(source, offset),
              }),
              {
                uri,
                range: { start: { line, character }, end: { line, character: character + length } },
              },
            );
          }
        }
      } finally {
        await session.shutdown();
        fs.rmSync(workspace, { recursive: true, force: true });
      }
    },
  );
}

test(
  "LF/CRLF, astral comments and nested quasi expressions retain complete UTF-16 vectors",
  { timeout: 60_000 },
  async () => {
    const workspace = fs.mkdtempSync(path.join(os.tmpdir(), "vize-lsp-8014-unicode-"));
    const session = new LspSession();
    try {
      await initialize(session, workspace, false);
      for (const newline of ["\n", "\r\n"]) {
        const source = [
          "<script setup>const count=1; const open=true</script>",
          "<template>",
          "<p>{{ '😀' + count /* 😀 日本語 count */ }}</p>",
          "<i :class=\"`😀-${open ? `nested ${count /* note */}` : 'right'}",
          '尾`" />',
          "</template>",
        ].join(newline);
        const pathname = path.join(workspace, newline === "\n" ? "Lf.vue" : "Crlf.vue"),
          uri = pathToFileURL(pathname).href;
        fs.writeFileSync(pathname, source);
        session.notify("textDocument/didOpen", {
          textDocument: { uri, languageId: "vue", version: 1, text: source },
        });
        const tokens: Token[] = [
          [2, 6, 4, 18, 0],
          [2, 11, 1, 21, 0],
          [2, 13, 5, 8, 0],
          [2, 19, 18, 17, 0],
          [3, 3, 6, 9, 0],
          [3, 11, 4, 18, 0],
          [3, 17, 4, 8, 0],
          [3, 22, 1, 21, 0],
          [3, 24, 8, 18, 0],
          [3, 34, 5, 8, 0],
          [3, 40, 10, 17, 0],
          [3, 50, 2, 18, 0],
          [3, 53, 1, 21, 0],
          [3, 55, 7, 18, 0],
          [3, 62, 1, 18, 0],
          [4, 0, 2, 18, 0],
        ];
        await assertTokens(session, uri, source, tokens);
        for (const marker of ["😀 日本語", "count */", "note */"]) {
          const params = {
            textDocument: { uri },
            position: offsetToPosition(source, source.indexOf(marker) + 1),
          };
          assert.equal(await session.request("textDocument/definition", params), null);
          assert.equal(await session.request("textDocument/hover", params), null);
        }
      }
      for (const [entity, entityTokens] of [
        [
          "&quot;",
          [
            ["&", 21],
            ["quot", 8],
          ],
        ],
        [
          "&#34;",
          [
            ["&", 21],
            ["34", 19],
          ],
        ],
      ] as const) {
        const source = `<template><p :title="${entity}/* data */${entity} + count /* note */" /></template>`;
        const uri = pathToFileURL(
          path.join(workspace, entity === "&quot;" ? "Named.vue" : "Numeric.vue"),
        ).href;
        session.notify("textDocument/didOpen", {
          textDocument: { uri, languageId: "vue", version: 1, text: source },
        });
        const spans: ReadonlyArray<readonly [string, number]> = [
          [":title", 9],
          ...entityTokens,
          ["/", 21],
          ["*", 21],
          ["data", 8],
          ["*", 21],
          ["/", 21],
          ...entityTokens,
          ["+", 21],
          ["count", 8],
          ["/* note */", 17],
        ];
        let cursor = 0;
        const tokens = spans.map(([text, kind]): Token => {
          const start = source.indexOf(text, cursor);
          assert.ok(start >= cursor);
          cursor = start + text.length;
          const position = offsetToPosition(source, start);
          return [position.line, position.character, text.length, kind, 0];
        });
        await assertTokens(session, uri, source, tokens);
        const params = {
          textDocument: { uri },
          position: offsetToPosition(source, source.indexOf("note")),
        };
        assert.equal(await session.request("textDocument/definition", params), null);
        assert.equal(await session.request("textDocument/hover", params), null);
      }
    } finally {
      await session.shutdown();
      fs.rmSync(workspace, { recursive: true, force: true });
    }
  },
);
