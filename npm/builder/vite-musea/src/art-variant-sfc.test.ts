import assert from "node:assert/strict";
import { test } from "node:test";

import { buildVariantSfcSource, compileVariantSfc } from "./art-variant-sfc.js";
import { loadNative } from "./native-loader.ts";
import type { ArtFileInfo } from "./types/index.js";

void test("native inline Art retains the imported host component name", () => {
  const filename = "/project/Button.vue";
  const source = `<template><button /></template><art title="Button"><variant name="Default"><Button /></variant></art>`;
  const parsed = loadNative().parseArt(source, { filename });
  assert.equal(parsed.variants[0].template, "<Button />");
  const art = artFile({
    path: filename,
    componentPath: filename,
    isInline: true,
    scriptSetupContent: undefined,
    hasScriptSetup: false,
    variants: parsed.variants,
  });
  const result = compileVariantSfc(art, "<Button />", "Default", filename);
  assert.deepEqual(result.errors, []);
  assert.match(result.code, /import Button from ['"]\/project\/Button\.vue['"]/u);
  assert.doesNotMatch(result.code, /_resolveComponent\("Button"\)/u);
});

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

void test("an inline variant play is contextually typed and registered from script-setup scope", () => {
  const source = buildVariantSfcSource(
    artFile({ scriptSetupContent: "const expectedLabel = 'Save'" }),
    `<button>Save</button>`,
    "Primary",
    {
      play: "async ({ canvas, expect }) => expect(canvas.getByText(expectedLabel)).toBeVisible()",
    },
  );

  assert.match(source, /const __museaVariantPlay: MuseaPlay = \(async \(\{ canvas, expect \}\)/);
  assert.match(source, /canvas\.getByText\(expectedLabel\)/);
  assert.match(source, /window as any\)\.__museaPlay = __museaVariantPlay/);
  assert.throws(
    () => buildVariantSfcSource(artFile(), `<button />`, "Unsafe", { play: "run); alert(1)" }),
    /Invalid Musea play expression/,
  );
});

void test("variant props and attrs keep generated component contracts", () => {
  const source = buildVariantSfcSource(artFile(), `<MuseaComponent label="Override" />`, "Typed", {
    componentImportPath: "/p/src/Example.vue",
    componentBindingName: "MuseaComponent",
    props: { label: "Save", disabled: false },
    attrs: { "aria-label": "Save button", "data-state": "ready" },
  });

  assert.match(
    source,
    /__museaVariantProps = \{"label":"Save","disabled":false\} satisfies Partial<MuseaComponentProps<typeof MuseaComponent>>/,
  );
  assert.match(
    source,
    /__museaVariantAttrs = .* satisfies MuseaComponentAttrs<typeof MuseaComponent>/,
  );
  assert.match(source, /<MuseaComponent v-bind="__museaVariantBindings" label="Override" \/>/);
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

void test("the inferred demonstrated component is imported for the authored tag", () => {
  const source = buildVariantSfcSource(artFile(), `<MuseaComponent />`, "Default", {
    componentImportPath: "/p/src/Example.vue",
    componentBindingName: "MuseaComponent",
  });

  assert.match(source, /import MuseaComponent from "\/p\/src\/Example\.vue"/);
});
