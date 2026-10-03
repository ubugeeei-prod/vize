import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import { test } from "node:test";
import { pathToFileURL } from "node:url";

const pack = JSON.parse(
  fs.readFileSync(
    new URL(
      "../../crates/vize_atelier_sfc/tests/fixtures/native_sfc_scoped_css_vue_3_5_35.json",
      import.meta.url,
    ),
    "utf8",
  ),
);
const fromUi = createRequire(new URL("../../npm/ui/package.json", import.meta.url));
const fromVue = createRequire(fromUi.resolve("vue/package.json"));
const { Window } = await import(pathToFileURL(fromUi.resolve("happy-dom")).href);
const window = new Window();
Object.assign(globalThis, {
  window,
  document: window.document,
  Element: window.Element,
  HTMLElement: window.HTMLElement,
  SVGElement: window.SVGElement,
  Node: window.Node,
});
const runtime = fromVue("vue");
const compiler = fromVue("@vue/compiler-sfc");
const dataUrl = (source: string) =>
  `data:text/javascript;base64,${Buffer.from(source).toString("base64")}`;
const runtimeUrl = dataUrl(
  `import runtime from ${JSON.stringify(pathToFileURL(fromVue.resolve("vue")).href)};\n` +
    [
      "toDisplayString",
      "openBlock",
      "createElementBlock",
      "createElementVNode",
      "createTextVNode",
      "createCommentVNode",
      "Fragment",
      "normalizeClass",
      "normalizeStyle",
    ]
      .map((name) => `export const ${name} = runtime.${name};`)
      .join("\n"),
);
const capturedPath = process.env.VIZE_NATIVE_SCOPED_CSS_CAPTURE;
const captured = capturedPath ? JSON.parse(fs.readFileSync(capturedPath, "utf8")) : null;
const executions: unknown[] = [];

test("scoped whole-SFC references keep pinned real CSS and complete native module contracts", () => {
  assert.equal(pack.schema, "vize.native-sfc.scoped-css-reference");
  assert.equal(pack.runtime, "vue@3.5.35");
  assert.equal(compiler.version, "3.5.35");
  assert.equal(fromVue("vue/package.json").version, "3.5.35");
  assert.equal(pack.fixtures.length, 4);
  assert.equal(new Set(pack.fixtures.map((fixture: any) => fixture.id)).size, 4);
  for (const fixture of pack.fixtures) {
    const parsed = compiler.parse(fixture.source, {
      filename: fixture.filename,
      ignoreEmpty: false,
    });
    assert.deepEqual(parsed.errors, []);
    assert.equal(parsed.descriptor.template.content, fixture.template);
    assert.equal(parsed.descriptor.scriptSetup?.content ?? null, fixture.script);
    assert.deepEqual(
      parsed.descriptor.styles.map((style: any) => ({
        scoped: !!style.scoped,
        source: style.content,
      })),
      fixture.styles,
    );
    let css = "";
    for (const style of parsed.descriptor.styles) {
      const compiled = compiler.compileStyle({
        source: style.content,
        filename: fixture.filename,
        id: fixture.scopeId,
        scoped: style.scoped,
        trim: false,
      });
      assert.deepEqual(compiled.errors, []);
      if (css) css += "\n";
      css += compiled.code;
    }
    assert.equal(css, fixture.css);
    assert(
      fixture.code.includes(
        `_sfc_main.__scopeId = ${JSON.stringify(fixture.scopeId)}\nexport default _sfc_main\n`,
      ),
    );
  }
});

test("quoted binding spellings change real Vue CSS or descriptor variables and require refusal", () => {
  for (const [value, expected] of [
    ["'v-bind(color)'", "'var(--quoted-color)'"],
    ['"v-bind (color)"', '"var(--quoted-color)"'],
    ["'v/**/-bind(color)'", "'v/**/-bind(color)'"],
    ["'v-/* x */bind(color)'", "'v-/* x */bind(color)'"],
  ]) {
    const css = `.a{content:${value}}`;
    const parsed = compiler.parse(`<template><p/></template><style scoped>${css}</style>`);
    assert.deepEqual(parsed.errors, []);
    assert.deepEqual(parsed.descriptor.cssVars, ["color"]);
    const result = compiler.compileStyle({
      source: css,
      filename: "Quoted.vue",
      id: "data-v-quoted",
      scoped: true,
      trim: false,
    });
    assert.deepEqual(result.errors, []);
    assert.equal(result.code, `.a[data-v-quoted]{content:${expected}}`);
  }
});

test(
  "fresh Rust scoped captures match each entire committed source/module/CSS result",
  { skip: !captured },
  () => {
    assert.deepEqual(
      captured.map(({ id, source, code, css, scopeId }: any) => ({
        id,
        source,
        code,
        css,
        scopeId,
      })),
      pack.fixtures.map(({ id, source, code, css, scopeId }: any) => ({
        id,
        source,
        code,
        css,
        scopeId,
      })),
    );
    for (const item of captured) {
      for (const map of [item.map, item.cssMap]) {
        assert.equal(map.version, 3);
        assert.deepEqual(map.sourcesContent, [item.source]);
        assert(map.mappings.length > 0);
      }
    }
  },
);

test("real Vue mounting applies the same scoped CSS selector to actual DOM only inside the component", async () => {
  for (const [index, expected] of pack.fixtures.entries()) {
    const fixture = { ...expected, ...captured?.[index] };
    window.document.body.replaceChildren();
    const style = window.document.createElement("style");
    style.textContent = fixture.css;
    window.document.head.replaceChildren(style);
    const mount = window.document.createElement("section");
    window.document.body.append(mount);
    const loaded = await import(
      dataUrl(fixture.code.replaceAll('from "vue"', `from ${JSON.stringify(runtimeUrl)}`))
    );
    assert.equal(loaded.default.__scopeId, fixture.scopeId);
    const app = runtime.createApp(loaded.default);
    const instance = app.mount(mount);
    await runtime.nextTick();
    const element = mount.firstElementChild;
    assert(element);
    assert.equal(element.getAttribute(fixture.scopeId), "");
    const outside = window.document.createElement(element.tagName);
    outside.className = element.className;
    window.document.body.append(outside);
    const scopedRules = [...style.sheet.cssRules].filter((rule: any) =>
      rule.selectorText?.includes(`[${fixture.scopeId}]`),
    );
    assert.equal(scopedRules.length, fixture.styles.filter((style: any) => style.scoped).length);
    for (const rule of scopedRules) {
      assert.equal(element.matches(rule.selectorText), true, fixture.id);
      assert.equal(outside.matches(rule.selectorText), false, fixture.id);
    }
    if (fixture.script) {
      assert.equal(element.textContent, fixture.text);
      instance.$.setupState.count = 4;
      instance.$forceUpdate();
      await runtime.nextTick();
      assert.equal(element.textContent, "4");
      assert.equal(element.getAttribute(fixture.scopeId), "");
    }
    executions.push({
      id: fixture.id,
      scopeId: loaded.default.__scopeId,
      selectors: scopedRules.map((rule: any) => rule.selectorText),
      html: mount.innerHTML,
      excludedOutside: outside.outerHTML,
    });
    app.unmount();
    assert.equal(mount.childElementCount, 0);
  }
  if (process.env.VIZE_NATIVE_SCOPED_CSS_RUNTIME_CAPTURE)
    fs.writeFileSync(
      process.env.VIZE_NATIVE_SCOPED_CSS_RUNTIME_CAPTURE,
      `${JSON.stringify({ runtime: pack.runtime, executions }, null, 2)}\n`,
    );
});
