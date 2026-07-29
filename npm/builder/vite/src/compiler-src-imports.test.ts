/**
 * `resolveSfcSrcImports` must keep inlining `src` blocks, and must not pay for a
 * second whole-SFC descriptor parse when the source cannot carry one.
 */
import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

import { resolveSfcSrcImports } from "./compiler.ts";

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
  const resolved = resolveSfcSrcImports(sfcPath, source);
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
  const resolved = resolveSfcSrcImports(sfcPath, source);
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

const resolvedAll = resolveSfcSrcImports(sfcPath, allBlocks);
assert.match(resolvedAll.source, /FromSrc/, "script src is inlined");
assert.match(resolvedAll.source, /from src/, "template src is inlined");
assert.match(resolvedAll.source, /seagreen/, "style src is inlined");
assert.deepEqual(
  resolvedAll.dependencies.map((dependency) => path.basename(dependency)).sort(),
  ["setup.ts", "styles.css", "template.html"],
  "all inlined src blocks are reported as dependencies",
);

// --- a lone `<style src>` block (handled outside the descriptor) still inlines -

const styleOnly = `<template><div /></template>\n<style src="./styles.css"></style>`;
const resolvedStyleOnly = resolveSfcSrcImports(sfcPath, styleOnly);
assert.match(resolvedStyleOnly.source, /seagreen/, "lone style src is inlined");
assert.deepEqual(
  resolvedStyleOnly.dependencies.map((dependency) => path.basename(dependency)),
  ["styles.css"],
  "lone style src is reported as a dependency",
);

// --- a template `src` attribute that is not a block still resolves correctly --

const imageSrc = `<template><img src="./logo.png" /></template>`;
const resolvedImage = resolveSfcSrcImports(sfcPath, imageSrc);
assert.equal(resolvedImage.source, imageSrc, "asset src attributes are left untouched");
assert.deepEqual(resolvedImage.dependencies, [], "asset src attributes add no dependencies");

fs.rmSync(dir, { recursive: true, force: true });

console.log("vite-plugin-vize SFC src import tests passed!");
