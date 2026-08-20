import assert from "node:assert/strict";
import { test } from "node:test";

import { buildVariantSfcSource, compileVariantSfc } from "./art-variant-sfc.js";
import type { ArtFileInfo } from "./types/index.js";

const scriptSetup = [
  "import { ref } from 'vue'",
  "const file = ref(null)",
  "const items = ref([{ isValid: false }])",
].join("\n");

function artFile(overrides: Partial<ArtFileInfo> = {}): ArtFileInfo {
  return {
    path: "/p/src/Example.art.vue",
    metadata: { title: "Example", tags: [], status: "ready" },
    variants: [],
    hasScriptSetup: true,
    hasScript: false,
    styleCount: 0,
    styleBlocks: [],
    isInline: false,
    scriptSetupContent: scriptSetup,
    ...overrides,
  } as ArtFileInfo;
}

void test("a type annotation in a template expression compiles instead of reaching the browser", () => {
  // `@update:value="(f: File | null) => …"` used to be handed to Vue's runtime
  // compiler as a string, where `new Function` threw on the annotation (#3857).
  const result = compileVariantSfc(
    artFile(),
    `<E :value="file" @update:value="(f: File | null) => (file = f)" />`,
    "TypeAnnotation",
    "/p/src/Example.art.vue",
  );

  assert.deepEqual(result.errors, []);
  assert.doesNotMatch(result.code, /File \| null/, "the annotation must be stripped");
  assert.match(result.code, /\(f\) =>/, "the handler must survive as plain JavaScript");
  assert.match(result.code, /\$setup\.file = f/, "the ref must resolve through setup");
  assert.doesNotMatch(
    result.code,
    /\$setup\.file\.value/,
    "module render receives proxy-unwrapped setup state",
  );
});

void test("a non-null assertion compiles and keeps its identifier resolved", () => {
  // The template compiler alone leaves `items[0]!.isValid` unprefixed, which is
  // a ReferenceError at render time; the SFC pipeline resolves it.
  const result = compileVariantSfc(
    artFile(),
    `<E :disabled="items[0]!.isValid" />`,
    "NonNullAssertion",
    "/p/src/Example.art.vue",
  );

  assert.deepEqual(result.errors, []);
  assert.doesNotMatch(result.code, /!\./, "the assertion must be stripped");
  assert.match(
    result.code,
    /\$setup\.items\[0\]\.isValid/,
    "the binding must resolve through setup",
  );
  assert.doesNotMatch(result.code, /\$setup\.items\.value/);
});

void test("plain JavaScript variants keep working", () => {
  const result = compileVariantSfc(
    artFile(),
    `<E :disabled="items[0].isValid" @update:value="f => (file = f)" />`,
    "PlainJs",
    "/p/src/Example.art.vue",
  );

  assert.deepEqual(result.errors, []);
  assert.match(result.code, /\$setup\.items\[0\]\.isValid/);
  assert.doesNotMatch(result.code, /\$setup\.items\.value/);
});

void test("the variant wrapper carries the variant name for gallery queries", () => {
  const source = buildVariantSfcSource(artFile(), `<E />`, 'Some "Quoted" Name');

  assert.match(source, /data-variant="Some &quot;Quoted&quot; Name"/);
  assert.match(source, /<script setup lang="ts">/);
});

void test("an art file without a script block still compiles its template", () => {
  const result = compileVariantSfc(
    artFile({ scriptSetupContent: undefined, hasScriptSetup: false }),
    `<span>static</span>`,
    "Static",
    "/p/src/Example.art.vue",
  );

  assert.deepEqual(result.errors, []);
  assert.match(result.code, /data-variant/);
  // A template-only SFC compiles to a bare `export function render`, which the
  // art module cannot import as a component.
  assert.match(result.code, /export default/, "every variant must export a component");
});

void test("the demonstrated component is imported so <Self> resolves", () => {
  const source = buildVariantSfcSource(artFile(), `<MuseaComponent />`, "Default", {
    componentImportPath: "/p/src/Example.vue",
    componentBindingName: "MuseaComponent",
  });

  assert.match(source, /import MuseaComponent from "\/p\/src\/Example\.vue"/);
});
