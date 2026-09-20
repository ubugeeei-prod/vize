import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import { PatternSession } from "./support/upstream/pattern-lsp.ts";
import {
  check,
  compareIdentity,
  diagnosticIdentity,
} from "./support/upstream/vue-language-tools.ts";
import { vueTscDiagnostics } from "./support/vue-tsc-oracle.ts";

const cases = [
  ["ambient initializer", "declare let label = 1;", "declare let label: number;"],
  ["missing expression", "const label = ;", "const label = 1;"],
  ["expression at block end", "const label =", "const label = 1;"],
  ["UTF-16 position", "const 雪 = '🌸'; const label = ;", "const 雪 = '🌸'; const label = 1;"],
] as const;

test("retained scripts preserve one native syntax diagnostic and clear it after repair", async () => {
  const editor = new PatternSession();
  try {
    await editor.initialize();
    for (const setup of [" setup", ""])
      for (const newline of ["\n", "\r\n"])
        for (const [label, invalid, repaired] of cases)
          for (const [script, count] of [
            [invalid, 1],
            [repaired, 0],
          ] as const) {
            const source =
              `<script${setup} lang="ts">\n${script}\n</script>\n<template />`.replaceAll(
                "\n",
                newline,
              );
            fs.writeFileSync(editor.file, source);
            const expected = vueTscDiagnostics(editor.directory).sort(compareIdentity);
            const context = `${label}/${setup || "module"}/${JSON.stringify(newline)}/${script}`;
            assert.equal(expected.length, count, context);
            assert.deepEqual(
              (await check(editor.directory)).map(diagnosticIdentity).sort(compareIdentity),
              expected,
              `CLI ${context}`,
            );
            assert.deepEqual(
              (await editor.update(source))
                .map((d) => ({
                  file: "App.vue",
                  line: d.range.start.line + 1,
                  column: d.range.start.character + 1,
                  code: Number(d.code),
                }))
                .sort(compareIdentity),
              expected,
              `LSP ${context}`,
            );
          }
  } finally {
    await editor.close();
  }
});

test("unchecked JavaScript retains syntax diagnostics", async () => {
  const editor = new PatternSession();
  try {
    await editor.initialize();
    const source = "<script setup>\nconst label = ;\n</script><template />";
    fs.writeFileSync(editor.file, source);
    assert.deepEqual(vueTscDiagnostics(editor.directory), [
      { file: "App.vue", line: 2, column: 15, code: 1109 },
    ]);
    // The JavaScript checkJs gate currently uses OXC's syntax fallback. Keep
    // this boundary explicit until native syntax and semantic gating separate.
    assert.deepEqual(await check(editor.directory), [
      {
        file: "App.vue",
        severity: "error",
        line: 2,
        column: 15,
        message: "Script parse error: Unexpected token",
      },
    ]);
  } finally {
    await editor.close();
  }
});
