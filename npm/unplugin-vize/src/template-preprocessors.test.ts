import { test } from "node:test";
import "./test/setup.ts";
import { vizeUnplugin } from "./unplugin.ts";
import { packageRoot } from "./test/helpers.ts";

function createPlugin() {
  return vizeUnplugin.raw(
    {
      isProduction: true,
      root: packageRoot,
    },
    {
      framework: "rollup",
    },
  );
}

// Runs the full transform pipeline (compileSfc -> generateOutput -> oxc strip-types)
// against an inline SFC source. The `.vue` path only has to end in `.vue` and stay
// out of `node_modules`; the source is passed directly, so no fixture file is needed.
async function transformSfc(source: string, fileName: string) {
  const plugin = createPlugin();
  const warnings: string[] = [];
  const result = await plugin.transform?.call(
    {
      warn(message: string) {
        warnings.push(message);
      },
    } as never,
    source,
    `${packageRoot}/${fileName}`,
  );
  return { result, warnings };
}

// NOTE on observed behavior: the native compiler accepts `<template lang="pug">`
// without throwing or warning, but it does NOT actually parse the Pug grammar.
// Pug element/attribute markup (e.g. `div.box`, `span(:title="msg")`) leaks through
// verbatim as plain template text, while `{{ }}` interpolation is still resolved.
// These tests therefore assert what the pipeline REALLY produces today: it compiles
// cleanly (no throw, no warnings), strips TypeScript, and emits a Vue render output.
// They intentionally do NOT assert that Pug markup is gone, because it is not.

void test("pug template flows through the transform without warnings or errors", async (t) => {
  const source = `<template lang="pug">
div.box Hello {{ msg }}
  span(:title="msg") world
</template>
<script setup>
const msg = "x"
</script>
`;
  const { result, warnings } = await transformSfc(source, "pug-basic.vue");

  t.assert.ok(result && typeof result === "object", "transform should return a result object");
  t.assert.equal(typeof result.code, "string");
  t.assert.ok(result.code.length > 0, "generated code should be non-empty");

  // No warnings were collected during compilation.
  t.assert.deepStrictEqual(warnings, []);

  // A render output is produced: the SFC descriptor object plus a runtime render
  // helper import. (Vize inlines the render fn inside `setup`; there is no top-level
  // `render` identifier, so we assert on the actual shape.)
  t.assert.match(result.code, /_sfc_main/);
  t.assert.match(result.code, /setup\(__props\)/);
  t.assert.match(result.code, /_createElementBlock/);
  t.assert.match(result.code, /from "vue"/);

  // `{{ msg }}` interpolation was resolved into a real binding reference.
  t.assert.match(result.code, /_toDisplayString\(msg\)/);

  // No leftover TS-only syntax in this JS `<script setup>` case.
  t.assert.doesNotMatch(result.code, /\binterface\b/);
});

void test("pug template with script setup lang=ts strips all TypeScript", async (t) => {
  const source = `<template lang="pug">
div.box Hello {{ msg }}
  span(:title="msg") world
</template>
<script setup lang="ts">
interface Foo { a: number }
const msg: string = "x"
const f: Foo = { a: 1 }
</script>
`;
  const { result, warnings } = await transformSfc(source, "pug-ts.vue");

  t.assert.ok(result && typeof result === "object");
  t.assert.ok(result.code.length > 0, "generated code should be non-empty");
  t.assert.deepStrictEqual(warnings, []);

  // The `interface` declaration and all type annotations must be gone after oxc strip.
  t.assert.doesNotMatch(result.code, /\binterface\b/);
  t.assert.doesNotMatch(result.code, /:\s*string/);
  t.assert.doesNotMatch(result.code, /:\s*Foo/);

  // Runtime values survive the strip.
  t.assert.match(result.code, /const msg = "x"/);
  t.assert.match(result.code, /const f = \{ a: 1 \}/);

  // Still produces a render output.
  t.assert.match(result.code, /_sfc_main/);
  t.assert.match(result.code, /_createElementBlock/);
});

void test("pug template with a scoped style compiles successfully", async (t) => {
  const source = `<template lang="pug">
div.box Hello {{ msg }}
</template>
<script setup>
const msg = "x"
</script>
<style scoped>
.box { color: red; }
</style>
`;
  const { result, warnings } = await transformSfc(source, "pug-scoped.vue");

  t.assert.ok(result && typeof result === "object");
  t.assert.ok(result.code.length > 0, "generated code should be non-empty");
  t.assert.deepStrictEqual(warnings, []);

  // A scope id is wired up for the scoped style; we do not over-assert the CSS itself.
  t.assert.match(result.code, /__scopeId/);
});

void test("the same markup as a standard (non-pug) template also compiles", async (t) => {
  const source = `<template>
<div class="box">Hello {{ msg }}<span :title="msg">world</span></div>
</template>
<script setup>
const msg = "x"
</script>
`;
  const { result, warnings } = await transformSfc(source, "standard.vue");

  t.assert.ok(result && typeof result === "object");
  t.assert.ok(result.code.length > 0, "generated code should be non-empty");
  t.assert.deepStrictEqual(warnings, []);

  // The standard parser produces a real `div` element (contrast with the pug path,
  // where `div.box` is emitted as plain text).
  t.assert.match(result.code, /_createElementBlock\("div"/);
  t.assert.match(result.code, /_toDisplayString\(msg\)/);
});
