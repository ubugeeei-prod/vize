import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { test } from "vite-plus/test";

const require = createRequire(import.meta.url);
const native = require("@vizejs/native") as {
  compileSfc(
    source: string,
    options?: { filename?: string; isTs?: boolean },
  ): { code: string; errors: unknown[] };
};

/** Deeper than any of these shapes reached on a host thread before. */
const DEPTH = 50_000;

function nested(open: string, middle: string, close: string): string {
  return open.repeat(DEPTH) + middle + close.repeat(DEPTH);
}

/** A `<script setup>` holding `script`, compiled on the caller's thread. */
function compileScript(script: string, isTs: boolean): { code: string; errors: unknown[] } {
  const source = `<script setup lang="ts">\n${script}\n</script>\n<template><p>{{ 1 }}</p></template>\n`;
  return native.compileSfc(source, { filename: "Deep.vue", isTs });
}

test("deeply nested script compiles without exhausting the caller's stack", () => {
  const cases: Array<[string, string]> = [
    ["object", `const x = ${nested("{a:", "1", "}")}`],
    ["call", `const x = ${nested("f(", "1", ")")}`],
    ["template", `const x = ${nested("`${", "1", "}`")}`],
    ["conditional", `const x = ${nested("a ? b : ", "c", "")}`],
    ["arrow", `const x = ${nested("() => ", "0", "")}`],
    ["unary", `const x = ${nested("!", "a", "")}`],
    ["type", `type T = ${nested("A<", "B", ">")}`],
    ["array", `const x = ${nested("[", "1", "]")}`],
    ["sum", `const x = ${nested("a + ", "a", "")}`],
    ["member", `const x = a${nested(".b", "", "")}`],
  ];
  for (const [name, script] of cases) {
    // `isTs: false` also strips TypeScript from the output, one more pass.
    for (const isTs of [true, false]) {
      const result = compileScript(script, isTs);
      assert.deepEqual(result.errors, [], `${name} isTs=${isTs}`);
      assert.ok(result.code.length > 0, `${name} isTs=${isTs}`);
    }
  }
}, 60_000);

test("deeply nested statements compile without exhausting the caller's stack", () => {
  // Kept as TypeScript output: reprinting JavaScript indents every level, so
  // the stripped module would outgrow a JavaScript string at this depth.
  const cases: Array<[string, string]> = [
    ["block", `function f() {${nested("{", "", "}")}}`],
    ["function", nested("function f() {", "", "}")],
    ["if", `function f() {${nested("if (a) ", ";", "")}}`],
  ];
  for (const [name, script] of cases) {
    const result = compileScript(script, true);
    assert.deepEqual(result.errors, [], name);
    assert.ok(result.code.length > 0, name);
  }
}, 60_000);
