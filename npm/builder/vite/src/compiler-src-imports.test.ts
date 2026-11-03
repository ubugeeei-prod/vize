/**
 * `inlineSfcSrcBlocks` must keep inlining `src` blocks, and must not pay for a
 * second whole-SFC descriptor parse when the source cannot carry one.
 */
import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

import { parseSfc } from "@vizejs/native";
import { inlineSfcSrcBlocks } from "../../shared/sfc-src.ts";
import { compileFile } from "./compiler.ts";

const dir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-src-imports-"));
const sfcPath = path.join(dir, "Component.vue");

fs.writeFileSync(path.join(dir, "setup.ts"), "export default { name: 'FromSrc' };\n");
fs.writeFileSync(path.join(dir, "template.html"), "<p>from src</p>");
fs.writeFileSync(path.join(dir, "styles.css"), ".root { color: seagreen; }\n");

// --- sources that provably have no block `src` skip the descriptor parse ------

const plainSources = [
  `<template><div>hi</div></template>`,
  `<script setup lang="ts">const a: number = 1</script>\n<template><b>{{ a }}</b></template>`,
  `<template><div class="src">no attribute here</div></template>`,
  ``,
];

for (const source of plainSources) {
  const resolved = inlineSfcSrcBlocks(parseSfc, sfcPath, source);
  assert.equal(
    resolved.source,
    source,
    "source without a src attribute must pass through verbatim",
  );
  assert.deepEqual(resolved.dependencies, [], "source without a src attribute has no dependencies");
}

// --- every spelling the descriptor accepts still reaches the inlining path ----

const srcSpellings = [
  `<script src="./setup.ts"></script>`,
  `<script\n  src = "./setup.ts"\n></script>`,
  `<script lang="ts" setup src='./setup.ts'></script>`,
];

for (const source of srcSpellings) {
  const resolved = inlineSfcSrcBlocks(parseSfc, sfcPath, source);
  assert.match(resolved.source, /FromSrc/, `must inline script src for: ${JSON.stringify(source)}`);
  assert.deepEqual(
    resolved.dependencies.map((dependency) => path.basename(dependency)),
    ["setup.ts"],
    "inlined src blocks are reported as dependencies",
  );
}

// --- all three block kinds still inline together -----------------------------

const allBlocks = `<script lang="ts" src="./setup.ts"></script>
<template src="./template.html"></template>
<style scoped src="./styles.css"></style>`;

const resolvedAll = inlineSfcSrcBlocks(parseSfc, sfcPath, allBlocks);
assert.match(resolvedAll.source, /FromSrc/, "script src is inlined");
assert.match(resolvedAll.source, /from src/, "template src is inlined");
assert.match(resolvedAll.source, /seagreen/, "style src is inlined");
assert.deepEqual(
  resolvedAll.dependencies.map((dependency) => path.basename(dependency)).sort(),
  ["setup.ts", "styles.css", "template.html"],
  "all inlined src blocks are reported as dependencies",
);

// --- a lone `<style src>` block still inlines ----------------------------------

const styleOnly = `<template><div /></template>\n<style src="./styles.css"></style>`;
const resolvedStyleOnly = inlineSfcSrcBlocks(parseSfc, sfcPath, styleOnly);
assert.match(resolvedStyleOnly.source, /seagreen/, "lone style src is inlined");
assert.deepEqual(
  resolvedStyleOnly.dependencies.map((dependency) => path.basename(dependency)),
  ["styles.css"],
  "lone style src is reported as a dependency",
);

// --- a template `src` attribute that is not a block still resolves correctly --

const imageSrc = `<template><img src="./logo.png" /></template>`;
const resolvedImage = inlineSfcSrcBlocks(parseSfc, sfcPath, imageSrc);
assert.equal(resolvedImage.source, imageSrc, "asset src attributes are left untouched");
assert.deepEqual(resolvedImage.dependencies, [], "asset src attributes add no dependencies");

// --- #11493 blocks come from the parser, not from text that looks like one ----

const compile = (source: string) =>
  compileFile(sfcPath, new Map(), { sourceMap: false, ssr: false, vapor: false }, source);
const STYLE_IN_SCRIPT = `const s = '<style src="./styles.css"></style>'`;

for (const source of [
  `<script setup>${STYLE_IN_SCRIPT}</script>\n<template><b>{{ s }}</b></template>`,
  `<template><b/></template>\n<!-- <style src="./missing.css"></style> -->`,
]) {
  const compiled = compile(source);
  assert.doesNotMatch(compiled.css ?? "", /seagreen/, source);
  assert.deepEqual(compiled.dependencies, [], source);
}
assert.match(
  compile(`<script setup>${STYLE_IN_SCRIPT}</script>\n<template><b>{{ s }}</b></template>`).code,
  /<style src=\\?"\.\/styles\.css\\?"><\/style>/,
  "the script string stays verbatim",
);

// A self-closing block ends at its own tag and keeps the block after it.
const selfClosing = compile(
  `<template src="./template.html" />\n<style src="./styles.css" />\n<style>.after { color: teal; }</style>`,
);
assert.match(selfClosing.code, /from src/);
assert.match(selfClosing.css ?? "", /seagreen/);
assert.match(selfClosing.css ?? "", /teal/);
assert.match(compile(`<script src="./setup.ts" />\n<template><p>x</p></template>`).code, /FromSrc/);

// Negative control: a real `<style src>` that names no file still fails.
assert.throws(
  () => compile(`<template><b/></template>\n<style src="./missing.css"></style>`),
  /<style src="\.\/missing\.css"> not found/,
);

fs.rmSync(dir, { recursive: true, force: true });

console.log("vite-plugin-vize SFC src import tests passed!");
