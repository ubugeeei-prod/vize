import { test } from "node:test";
import {
  extractStyleBlocks,
  generateOutput,
  generateScopeId,
  hasDelegatedStyles,
  toStyleBlockInfo,
} from "./style.ts";
import type { CompiledModule, StyleBlockInfo, StyleBlockNapi } from "./types.ts";

function makeBlock(over: Partial<StyleBlockInfo> = {}): StyleBlockInfo {
  return {
    content: "",
    src: null,
    lang: null,
    scoped: false,
    module: false,
    index: 0,
    ...over,
  };
}

function makeCompiled(over: Partial<CompiledModule> = {}): CompiledModule {
  return {
    code: "export default {}",
    scopeId: "abc123",
    hasScoped: false,
    styles: [],
    ...over,
  };
}

void test("extractStyleBlocks returns ordered StyleBlockInfo for scoped scss + css module", (t) => {
  const source = [
    `<template><div class="a">hi</div></template>`,
    `<style scoped lang="scss">`,
    `.a { color: red; }`,
    `</style>`,
    `<style module>`,
    `.b { color: blue; }`,
    `</style>`,
  ].join("\n");

  const blocks = extractStyleBlocks(source);

  t.assert.equal(blocks.length, 2);

  // first block: scoped scss
  t.assert.equal(blocks[0].index, 0);
  t.assert.equal(blocks[0].lang, "scss");
  t.assert.equal(blocks[0].scoped, true);
  t.assert.equal(blocks[0].module, false);
  t.assert.match(blocks[0].content, /\.a \{ color: red; \}/);

  // second block: unnamed css module (module === true), plain css lang (null)
  t.assert.equal(blocks[1].index, 1);
  t.assert.equal(blocks[1].lang, null);
  t.assert.equal(blocks[1].scoped, false);
  t.assert.equal(blocks[1].module, true);
  t.assert.match(blocks[1].content, /\.b \{ color: blue; \}/);
});

void test("extractStyleBlocks surfaces a named css module as the module name string", (t) => {
  const source = [
    `<template><div class="b">hi</div></template>`,
    `<style module="cls">`,
    `.b { color: blue; }`,
    `</style>`,
  ].join("\n");

  const blocks = extractStyleBlocks(source);

  t.assert.equal(blocks.length, 1);
  t.assert.equal(blocks[0].module, "cls");
  t.assert.equal(blocks[0].scoped, false);
  t.assert.equal(blocks[0].index, 0);
});

void test("toStyleBlockInfo maps napi block module flag to boolean|string", (t) => {
  const base: StyleBlockNapi = {
    content: ".x{}",
    scoped: true,
    module: false,
    index: 3,
  };

  // plain block
  const plain = toStyleBlockInfo(base);
  t.assert.equal(plain.module, false);
  t.assert.equal(plain.lang, null);
  t.assert.equal(plain.src, null);
  t.assert.equal(plain.scoped, true);
  t.assert.equal(plain.index, 3);

  // unnamed module -> true
  const unnamed = toStyleBlockInfo({ ...base, module: true });
  t.assert.equal(unnamed.module, true);

  // named module -> name string
  const named = toStyleBlockInfo({ ...base, module: true, moduleName: "foo" });
  t.assert.equal(named.module, "foo");

  // src + lang preserved
  const withSrc = toStyleBlockInfo({ ...base, src: "./a.css", lang: "scss" });
  t.assert.equal(withSrc.src, "./a.css");
  t.assert.equal(withSrc.lang, "scss");
});

void test("generateScopeId is deterministic and returns an 8-char hex id", (t) => {
  const source = `<template><div>x</div></template><style scoped>.a{}</style>`;
  const a = generateScopeId("/project/src/App.vue", "/project", false, source);
  const b = generateScopeId("/project/src/App.vue", "/project", false, source);

  t.assert.equal(a, b);
  t.assert.match(a, /^[0-9a-f]{8}$/);
});

void test("hasDelegatedStyles is false for a single plain non-module css block", (t) => {
  t.assert.equal(hasDelegatedStyles(makeCompiled({ styles: [makeBlock()] })), false);
  // explicit lang "css" is also plain
  t.assert.equal(hasDelegatedStyles(makeCompiled({ styles: [makeBlock({ lang: "css" })] })), false);
  // no styles at all
  t.assert.equal(hasDelegatedStyles(makeCompiled({ styles: [] })), false);
});

void test("hasDelegatedStyles is true when any block needs a preprocessor", (t) => {
  for (const lang of ["scss", "sass", "less", "stylus", "styl"]) {
    t.assert.equal(
      hasDelegatedStyles(makeCompiled({ styles: [makeBlock({ lang })] })),
      true,
      `lang=${lang} should delegate`,
    );
  }
});

void test("hasDelegatedStyles is true for css-module blocks (unnamed or named)", (t) => {
  t.assert.equal(hasDelegatedStyles(makeCompiled({ styles: [makeBlock({ module: true })] })), true);
  t.assert.equal(
    hasDelegatedStyles(makeCompiled({ styles: [makeBlock({ module: "cls" })] })),
    true,
  );
});

void test("hasDelegatedStyles is true when one of several blocks delegates", (t) => {
  const compiled = makeCompiled({
    styles: [makeBlock({ index: 0 }), makeBlock({ index: 1, lang: "less" })],
  });
  t.assert.equal(hasDelegatedStyles(compiled), true);
});

void test("generateOutput (a): export default with hasScoped and no _sfc_main rewrites and appends scopeId", (t) => {
  const out = generateOutput(
    makeCompiled({ code: "export default { name: 'A' }", scopeId: "abc123", hasScoped: true }),
    { isProduction: true, isDev: false },
  );

  // rewritten declaration
  t.assert.match(out, /const _sfc_main = \{ name: 'A' \}/);
  // no leftover original export-default-of-object
  t.assert.equal(/^export default \{/m.test(out), false);
  // appended scopeId assignment
  t.assert.match(out, /_sfc_main\.__scopeId = "data-v-abc123";/);
  // re-exported default
  t.assert.match(out, /export default _sfc_main;/);
  // no style injection IIFE (no css)
  t.assert.equal(out.includes("document.createElement"), false);
});

void test("generateOutput (a'): export default WITHOUT hasScoped does not append __scopeId", (t) => {
  const out = generateOutput(makeCompiled({ code: "export default {}", hasScoped: false }), {
    isProduction: true,
    isDev: false,
  });

  t.assert.match(out, /const _sfc_main = \{\}/);
  t.assert.match(out, /export default _sfc_main;/);
  t.assert.equal(out.includes("__scopeId"), false);
});

void test("generateOutput (b): plain css in non-production injects the __vize_css__ style IIFE", (t) => {
  const out = generateOutput(
    makeCompiled({
      code: "export default {}",
      scopeId: "abc123",
      css: ".x{color:red}",
      styles: [makeBlock()],
    }),
    { isProduction: false, isDev: false },
  );

  t.assert.match(out, /export const __vize_css__ = ".x\{color:red\}";/);
  t.assert.match(out, /const __vize_css_id__ = "vize-style-abc123";/);
  t.assert.match(out, /document\.createElement\("style"\)/);
  t.assert.match(out, /document\.head\.appendChild\(style\)/);
  // the original module body still follows the IIFE
  t.assert.match(out, /export default _sfc_main;/);
});

void test("generateOutput (b'): plain css is also injected in production when extractCss is falsy", (t) => {
  const out = generateOutput(makeCompiled({ css: ".y{}", styles: [makeBlock()] }), {
    isProduction: true,
    isDev: false,
    extractCss: false,
  });
  t.assert.match(out, /export const __vize_css__ = ".y\{\}";/);
  t.assert.match(out, /document\.createElement/);
});

void test("generateOutput (b''): plain css is NOT injected in production when extractCss is true", (t) => {
  const out = generateOutput(makeCompiled({ css: ".z{}", styles: [makeBlock()] }), {
    isProduction: true,
    isDev: false,
    extractCss: true,
  });
  t.assert.equal(out.includes("__vize_css__"), false);
  t.assert.equal(out.includes("document.createElement"), false);
});

void test("generateOutput (c): delegated scss (scoped) + css module emits style imports + __cssModules setup", (t) => {
  const out = generateOutput(
    makeCompiled({
      code: "export default {}",
      scopeId: "abc123",
      hasScoped: true,
      styles: [
        makeBlock({ lang: "scss", scoped: true, index: 0 }),
        makeBlock({ module: true, index: 1 }),
      ],
    }),
    { isProduction: true, isDev: false, filePath: "/proj/App.vue" },
  );

  // scoped scss block -> side-effect style import carrying scoped=data-v-<scopeId>
  t.assert.match(
    out,
    /import "\/proj\/App\.vue\?vue=&type=style&index=0&lang=scss&scoped=data-v-abc123";/,
  );
  // unnamed css module -> default-binding import using $style, lang defaults to css, module= (empty)
  t.assert.match(
    out,
    /import \$style from "\/proj\/App\.vue\?vue=&type=style&index=1&lang=css&module=";/,
  );
  // __cssModules wiring
  t.assert.match(out, /_sfc_main\.__cssModules = _sfc_main\.__cssModules \|\| \{\};/);
  t.assert.match(out, /_sfc_main\.__cssModules\["\$style"\] = \$style;/);
  // setup is placed before export default _sfc_main
  t.assert.match(out, /__cssModules\["\$style"\] = \$style;\nexport default _sfc_main;/);
  // delegated path means the inline css IIFE is NOT used
  t.assert.equal(out.includes("document.createElement"), false);
  // scoped scopeId still applied to the component
  t.assert.match(out, /_sfc_main\.__scopeId = "data-v-abc123";/);
});

void test("generateOutput (c'): named css module uses its name as binding and url module value", (t) => {
  const out = generateOutput(
    makeCompiled({
      code: "export default {}",
      scopeId: "zzz",
      hasScoped: false,
      styles: [makeBlock({ module: "cls", index: 0 })],
    }),
    { isProduction: true, isDev: false, filePath: "/proj/App.vue" },
  );

  t.assert.match(
    out,
    /import cls from "\/proj\/App\.vue\?vue=&type=style&index=0&lang=css&module=cls";/,
  );
  t.assert.match(out, /_sfc_main\.__cssModules\["cls"\] = cls;/);
});

void test("generateOutput (c''): delegated styles require filePath; without it falls back (no style imports)", (t) => {
  const out = generateOutput(
    makeCompiled({
      code: "export default {}",
      scopeId: "abc123",
      styles: [makeBlock({ lang: "scss", scoped: true, index: 0 })],
    }),
    { isProduction: true, isDev: false },
  );

  // no filePath => no delegated style import is emitted
  t.assert.equal(out.includes("?vue=&type=style"), false);
  t.assert.equal(out.includes("__cssModules"), false);
});

void test("generateOutput (d): named render export with no export default and no _sfc_main appends component shell", (t) => {
  const out = generateOutput(
    makeCompiled({
      code: "export function render() { return 1; }",
      scopeId: "rrr",
      hasScoped: true,
      styles: [],
    }),
    { isProduction: true, isDev: false },
  );

  // original named export preserved
  t.assert.match(out, /export function render\(\) \{ return 1; \}/);
  // appended component shell
  t.assert.match(out, /const _sfc_main = \{\};/);
  t.assert.match(out, /_sfc_main\.__scopeId = "data-v-rrr";/);
  t.assert.match(out, /_sfc_main\.render = render;/);
  t.assert.match(out, /export default _sfc_main;/);
});

void test("generateOutput (d'): named render export without scoping skips __scopeId", (t) => {
  const out = generateOutput(
    makeCompiled({
      code: "export function render() { return 1; }",
      scopeId: "rrr",
      hasScoped: false,
      styles: [],
    }),
    { isProduction: true, isDev: false },
  );

  t.assert.match(out, /const _sfc_main = \{\};/);
  t.assert.match(out, /_sfc_main\.render = render;/);
  t.assert.equal(out.includes("__scopeId"), false);
});
