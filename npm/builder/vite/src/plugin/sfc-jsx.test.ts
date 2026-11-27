import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire, SourceMap } from "node:module";
import os from "node:os";
import path from "node:path";
import { after, test } from "node:test";

import type { VizePluginState } from "./state.ts";

// `compiler.ts` takes `compileJsx` off the binding when it is first evaluated,
// so the spy is installed before any plugin module is imported.
const native = createRequire(import.meta.url)("@vizejs/native") as Record<string, unknown>;
const jsxCalls: {
  filename?: string;
  jsxMode?: string;
  jsxCompat?: string;
  vapor?: boolean;
  lang?: string;
}[] = [];
const compileJsx = native.compileJsx as (source: string, options?: object) => unknown;
native.compileJsx = (source: string, options?: (typeof jsxCalls)[number]) => {
  jsxCalls.push({ ...options });
  return compileJsx(source, options);
};
const { loadSfcModule } = await import("../test/sfc-module.ts");
const { VizeSfcCompileError } = await import("../compiler.ts");
const { findSfcJsxScript } = await import("./sfc-jsx.ts");
const { parseSync } = await import("oxc-parser");

const TSX_SETUP = `<script setup lang="tsx">
import { ref } from "vue";
const n = ref<number>(2);
const formatCount = (value: number) => \`#\${value}\`;
const Render = () => <span class="tsx">{formatCount(n.value)}</span>;
</script>

<template><div><p>static</p><Render /></div></template>
`;

const roots: string[] = [];
after(() => {
  for (const root of roots) fs.rmSync(root, { recursive: true, force: true });
});

function project(files: Record<string, string>) {
  const root = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-sfc-jsx-")));
  roots.push(root);
  for (const [name, source] of Object.entries(files)) {
    fs.writeFileSync(path.join(root, name), source);
  }
  return { root, file: (name: string) => path.join(root, name) };
}

function state(root: string, overrides: Partial<VizePluginState> = {}): VizePluginState {
  return {
    cache: new Map(),
    ssrCache: new Map(),
    collectedCss: new Map(),
    precompileMetadata: new Map(),
    pendingHmrUpdateTypes: new Map(),
    isProduction: false,
    root,
    clientViteBase: "/",
    serverViteBase: "/",
    server: null,
    filter: () => true,
    scanPatterns: ["**/*.vue"],
    precompileBatchSize: 128,
    ignorePatterns: [],
    mergedOptions: {},
    initialized: true,
    dynamicImportAliasRules: [],
    cssAliasRules: [],
    extractCss: false,
    componentsCssFileName: "assets/vize-components.css",
    clientViteDefine: {},
    serverViteDefine: {},
    logger: { log() {}, info() {}, warn() {}, error() {} } as never,
    ...overrides,
  } as VizePluginState;
}

/** The module parses as JavaScript, and binds each import name once. */
function assertLoweredModule(code: string, label: string): void {
  assert.doesNotMatch(code, /<span|<b>|<\/b>/, `${label}: no JSX is left for the bundler`);
  const parsed = parseSync("module.js", code, { sourceType: "module" });
  assert.deepEqual(parsed.errors, [], `${label}: the output parses as JavaScript`);
  const locals = parsed.program.body.flatMap((statement) =>
    statement.type === "ImportDeclaration"
      ? statement.specifiers.map((specifier) => specifier.local.name)
      : [],
  );
  assert.deepEqual(locals, [...new Set(locals)], `${label}: no import binding is declared twice`);
}

void test("JSX in <script setup lang=tsx> is lowered for the client, SSR and the dev server", async () => {
  const { root, file } = project({ "Va.vue": TSX_SETUP });
  const cases = [
    { label: "client build", ssr: false, server: null },
    { label: "SSR build", ssr: true, server: null },
    { label: "dev server", ssr: false, server: { watcher: { add() {} } } as never },
  ];
  for (const { label, ssr, server } of cases) {
    jsxCalls.length = 0;
    const result = await loadSfcModule(state(root, { server }), file("Va.vue"), { ssr });
    assert.ok(result, label);
    assertLoweredModule(result.code, label);
    assert.match(
      result.code,
      /_createElementVNode\("span", \{ class: "tsx" \}, \[\s+formatCount\(n\.value\)\s+\]\)/,
      `${label}: the JSX element becomes a VNode, which client and server renderers both render`,
    );
    assert.match(result.code, ssr ? /ssrRender/ : /_sfc_render/, label);
    if (server) assert.match(result.code, /import\.meta\.hot/, `${label}: HMR code stays`);
    assert.equal(jsxCalls.length, 1, `${label}: one JSX lowering per module`);
    assert.equal(jsxCalls[0].lang, "tsx", label);
  }
});

void test("a position inside the lowered JSX maps back to its line in the .vue file", async () => {
  const { root, file } = project({ "Va.vue": TSX_SETUP });
  for (const ssr of [false, true]) {
    const result = await loadSfcModule(state(root), file("Va.vue"), { ssr });
    assert.ok(result?.map, `ssr=${ssr}: the module keeps a map`);
    const offset = result.code.indexOf("formatCount(n.value)");
    const before = result.code.slice(0, offset);
    const line = before.split("\n").length - 1;
    const column = offset - (before.lastIndexOf("\n") + 1);
    const entry = new SourceMap(result.map as never).findEntry(line, column) as {
      originalLine?: number;
      originalSource?: string;
    };
    assert.equal(
      entry.originalLine,
      TSX_SETUP.split("\n").findIndex((text) => text.includes("<span")),
      `ssr=${ssr}: the JSX child traces to its authored line`,
    );
    assert.equal(entry.originalSource, file("Va.vue"), `ssr=${ssr}`);
  }
});

void test("lang=jsx and a normal <script lang=tsx> beside <script setup> are lowered", async () => {
  const { root, file } = project({
    "Jsx.vue": `<script setup lang="jsx">
const label = "jsx";
const Render = () => <b>{label}</b>;
</script>
<template><Render /></template>
`,
    "Both.vue": `<script lang="tsx">
export const Badge = (props) => <b class="badge">{props.text}</b>;
</script>
<script setup lang="tsx">
const Render = () => <span><Badge text="both" /></span>;
</script>
<template><Render /></template>
`,
  });
  for (const ssr of [false, true]) {
    jsxCalls.length = 0;
    const jsx = await loadSfcModule(state(root), file("Jsx.vue"), { ssr });
    assert.ok(jsx);
    assertLoweredModule(jsx.code, `lang=jsx ssr=${ssr}`);
    assert.match(jsx.code, /_createElementVNode\("b", null, \[\s+label\s+\]\)/);
    assert.equal(jsxCalls[0]?.lang, "jsx");

    const both = await loadSfcModule(state(root), file("Both.vue"), { ssr });
    assert.ok(both);
    assertLoweredModule(both.code, `normal and setup scripts ssr=${ssr}`);
    assert.match(both.code, /_createElementVNode\("b", \{ class: "badge" \}/);
    assert.match(both.code, /_resolveDynamicComponent\(Badge\), \{ text: "both" \}/);
  }
});

void test("an SFC without a JSX script block is left to the SFC compiler alone", async () => {
  const source = `<script setup lang="ts">
const markup: string = "<span>not jsx</span>";
</script>
<template><p>{{ markup }}</p></template>
`;
  const { root, file } = project({ "Plain.vue": source });
  for (const ssr of [false, true]) {
    jsxCalls.length = 0;
    const result = await loadSfcModule(state(root), file("Plain.vue"), { ssr });
    assert.ok(result);
    assert.equal(jsxCalls.length, 0, `ssr=${ssr}: no JSX lowering call`);
    assert.match(result.code, /"<span>not jsx<\/span>"/, `ssr=${ssr}`);
  }
});

void test("the JSX options of the plugin reach the lowering, and its errors fail the compile", async () => {
  const { root, file } = project({ "Va.vue": TSX_SETUP });
  jsxCalls.length = 0;
  await loadSfcModule(state(root, { mergedOptions: { jsxCompat: "babel" } }), file("Va.vue"));
  assert.equal(jsxCalls[0]?.jsxCompat, "babel");

  // A setup binding that escapes into the returned bindings cannot be a Vapor
  // JSX component, so this mode reports a diagnostic instead of emitting code.
  const vapor = state(root, { mergedOptions: { jsxMode: "vapor" } });
  await assert.rejects(loadSfcModule(vapor, file("Va.vue")), (error: unknown) => {
    assert.ok(error instanceof VizeSfcCompileError);
    assert.equal(error.filePath, file("Va.vue"));
    assert.match(error.message, /Va\.vue/);
    assert.match(error.message, /in <script lang="tsx"> \(line 1\)/);
    return true;
  });
  assert.equal(jsxCalls.at(-1)?.jsxMode, "vapor");
});

void test("a > or a lang inside a quoted attribute value does not end or name the script tag", async () => {
  assert.deepEqual(findSfcJsxScript(`<template />\n<script setup data-note=">" lang="tsx">`), {
    lang: "tsx",
    line: 2,
  });
  assert.deepEqual(
    findSfcJsxScript(`<script setup generic="T extends Array<string>" lang='jsx'>`),
    { lang: "jsx", line: 1 },
  );
  assert.equal(findSfcJsxScript(`<script setup lang="ts" data-x='lang="tsx"'>`), null);
  assert.equal(findSfcJsxScript(`<script setup lang="tsx-like">`), null);
  assert.deepEqual(
    findSfcJsxScript(`<script lang="ts">\n</script>\n<script setup lang=tsx>`),
    { lang: "tsx", line: 3 },
    "a later script tag is still read",
  );

  const { root, file } = project({
    "Quoted.vue": TSX_SETUP.replace("<script setup", `<script setup data-note=">"`),
  });
  const result = await loadSfcModule(state(root), file("Quoted.vue"));
  assert.ok(result);
  assertLoweredModule(result.code, "quoted > before lang");
});

void test("SSR lowers JSX to VNodes whatever jsxMode or vapor ask of the client", async () => {
  const { root, file } = project({ "Va.vue": TSX_SETUP });
  for (const mergedOptions of [{ jsxMode: "vapor" as const }, { vapor: true }]) {
    jsxCalls.length = 0;
    const result = await loadSfcModule(state(root, { mergedOptions }), file("Va.vue"), {
      ssr: true,
    });
    assert.ok(result);
    assertLoweredModule(result.code, `ssr ${JSON.stringify(mergedOptions)}`);
    assert.match(result.code, /_createElementVNode\("span"/);
    assert.equal(jsxCalls[0]?.jsxMode, "vdom");
    assert.equal(jsxCalls[0]?.vapor, false);
  }
});

void test("an authored vue import that binds a JSX helper's local name fails the compile", async () => {
  const { root, file } = project({
    "Clash.vue": `<template><Render /></template>
<script setup lang="tsx">
import { ref as _createElementVNode } from "vue";
const n = _createElementVNode(2);
const Render = () => <span>{n.value}</span>;
</script>
`,
  });
  for (const ssr of [false, true]) {
    await assert.rejects(
      loadSfcModule(state(root), file("Clash.vue"), { ssr }),
      (error: unknown) => {
        assert.ok(error instanceof VizeSfcCompileError);
        assert.equal(error.filePath, file("Clash.vue"));
        assert.match(error.message, /`ref as _createElementVNode`/);
        assert.match(error.message, /in <script lang="tsx"> \(line 2\)/);
        return true;
      },
    );
  }
});
