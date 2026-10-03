import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import { test } from "node:test";
import { pathToFileURL } from "node:url";

const pack = JSON.parse(
  fs.readFileSync(
    new URL(
      "../../crates/vize_atelier_sfc/tests/fixtures/native_sfc_scoped_class_list_vue_3_5_35.json",
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
const capturedPath = process.env.VIZE_NATIVE_CLASS_LIST_CSS_CAPTURE;
const captured = capturedPath ? JSON.parse(fs.readFileSync(capturedPath, "utf8")) : null;
const executions: unknown[] = [];

test("class-list whole-SFC references keep pinned real CSS and complete native module contracts", () => {
  assert.equal(pack.schema, "vize.native-sfc.scoped-class-list-reference");
  assert.equal(pack.runtime, "vue@3.5.35");
  assert.equal(compiler.version, "3.5.35");
  assert.equal(fromVue("vue/package.json").version, "3.5.35");
  assert.equal(pack.fixtures.length, 3);
  assert.equal(new Set(pack.fixtures.map((fixture: any) => fixture.id)).size, 3);
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

test(
  "fresh Rust class-list captures match each entire committed source/module/CSS result",
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

test("real Vue class-list styles apply every selector independently only inside the component", async () => {
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
    assert.equal(element.className, fixture.classes.join(" "));
    assert.equal(element.getAttribute(fixture.scopeId), "");
    const outside = window.document.createElement(element.tagName);
    window.document.body.append(outside);
    const scopedRules = [...style.sheet.cssRules].filter((rule: any) =>
      rule.selectorText?.includes(`[${fixture.scopeId}]`),
    );
    assert.equal(scopedRules.length, 1);
    const alternatives: unknown[] = [];
    for (const [classIndex, classname] of fixture.classes.entries()) {
      element.className = classname;
      outside.className = classname;
      const selector = fixture.selectors[classIndex];
      assert.equal(element.matches(selector), true, `${fixture.id}: ${selector}`);
      assert.equal(outside.matches(selector), false, `${fixture.id}: ${selector}`);
      assert.equal(element.matches(scopedRules[0].selectorText), true, fixture.id);
      assert.equal(outside.matches(scopedRules[0].selectorText), false, fixture.id);
      assert.equal(window.getComputedStyle(element).backgroundColor, "blue", fixture.id);
      assert.notEqual(window.getComputedStyle(outside).backgroundColor, "blue", fixture.id);
      alternatives.push({
        classname,
        selector,
        inside: element.outerHTML,
        excludedOutside: outside.outerHTML,
      });
    }
    element.className = fixture.classes.join(" ");
    if (fixture.script) {
      assert.equal(element.textContent, fixture.text);
      instance.$.setupState.count = 4;
      instance.$forceUpdate();
      await runtime.nextTick();
      assert.equal(element.textContent, "4");
      assert.equal(element.getAttribute(fixture.scopeId), "");
      assert.equal(element.className, fixture.classes.join(" "));
    }
    executions.push({
      id: fixture.id,
      scopeId: loaded.default.__scopeId,
      alternatives,
      html: mount.innerHTML,
    });
    app.unmount();
    assert.equal(mount.childElementCount, 0);
  }
  if (process.env.VIZE_NATIVE_CLASS_LIST_CSS_RUNTIME_CAPTURE)
    fs.writeFileSync(
      process.env.VIZE_NATIVE_CLASS_LIST_CSS_RUNTIME_CAPTURE,
      `${JSON.stringify({ runtime: pack.runtime, executions }, null, 2)}\n`,
    );
});
