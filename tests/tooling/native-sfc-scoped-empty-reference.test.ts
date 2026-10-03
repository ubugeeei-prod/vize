import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import { test } from "node:test";

const pack = JSON.parse(
  fs.readFileSync(
    new URL(
      "../../crates/vize_atelier_sfc/tests/fixtures/native_sfc_scoped_empty_vue_3_5_35.json",
      import.meta.url,
    ),
    "utf8",
  ),
);
const fromUi = createRequire(new URL("../../npm/ui/package.json", import.meta.url));
const fromVue = createRequire(fromUi.resolve("vue/package.json"));
const compiler = fromVue("@vue/compiler-sfc");
const { parse } = fromVue("@babel/parser");
const capturedPath = process.env.VIZE_NATIVE_EMPTY_CSS_CAPTURE;
// A promised current-source receipt must exist; it never falls back to fixtures.
const captured = capturedPath ? JSON.parse(fs.readFileSync(capturedPath, "utf8")) : null;
const dataUrl = (source: string) =>
  `data:text/javascript;base64,${Buffer.from(source).toString("base64")}`;

// Dev-only execution changes only actual import source spans, preserving every
// other module byte, including primitive setup strings and authored comments.
function relinkRuntime(code: string, runtimeUrl: string) {
  let linked = "";
  let cursor = 0;
  for (const declaration of parse(code, { sourceType: "module" }).program.body) {
    if (declaration.type !== "ImportDeclaration") continue;
    assert.equal(declaration.source.value, "vue");
    linked += code.slice(cursor, declaration.source.start) + JSON.stringify(runtimeUrl);
    cursor = declaration.source.end;
  }
  return linked + code.slice(cursor);
}

test("runtime import relinking preserves original Vue-like setup strings and comments", () => {
  const original = `import { createApp } from "vue";\nlet label='from "vue"';/* from "vue" */`;
  assert.equal(
    relinkRuntime(original, "data:real-vue-runtime"),
    `import { createApp } from "data:real-vue-runtime";\nlet label='from "vue"';/* from "vue" */`,
  );
});

test("literal empty whole-SFC CSS and complete original-source maps match pinned Vue", () => {
  assert.equal(pack.schema, "vize.native-sfc.scoped-empty-reference");
  assert.equal(pack.runtime, "vue@3.5.35");
  assert.equal(compiler.version, "3.5.35");
  assert.equal(fromVue("vue/package.json").version, "3.5.35");
  assert.equal(pack.fixtures.length, 3);
  assert.equal(new Set(pack.fixtures.map((fixture: any) => fixture.id)).size, 3);
  for (const fixture of pack.fixtures) {
    const parsed = compiler.parse(fixture.source, {
      filename: fixture.filename,
      sourceMap: true,
      ignoreEmpty: false,
    });
    assert.deepEqual(parsed.errors, []);
    assert.equal(parsed.descriptor.template.content, fixture.template);
    assert.equal(parsed.descriptor.scriptSetup?.content ?? null, fixture.script);
    assert.deepEqual(parsed.descriptor.cssVars, []);
    assert.deepEqual(
      parsed.descriptor.styles.map((style: any) => ({
        scoped: !!style.scoped,
        source: style.content,
      })),
      fixture.styles,
    );
    const style = parsed.descriptor.styles[0];
    const compiled = compiler.compileStyle({
      source: style.content,
      filename: fixture.filename,
      id: fixture.scopeId,
      scoped: true,
      trim: false,
      map: style.map,
    });
    assert.deepEqual(compiled.errors, []);
    assert.equal(compiled.code, fixture.css);
    assert.deepEqual(compiled.map, fixture.stockCssMap);
    assert.deepEqual(compiled.map.sourcesContent, [fixture.source]);
    assert(
      fixture.code.includes(
        `_sfc_main.__scopeId = ${JSON.stringify(fixture.scopeId)}\nexport default _sfc_main\n`,
      ),
    );
  }
});

test(
  "fresh Rust empty captures match each entire source/module/CSS result",
  { skip: !captured },
  () => {
    const whole = ({ id, source, code, css, scopeId }: any) => ({ id, source, code, css, scopeId });
    assert.deepEqual(captured.map(whole), pack.fixtures.map(whole));
    for (const item of captured) {
      for (const map of [item.map, item.cssMap]) {
        assert.equal(map.version, 3);
        assert.deepEqual(map.sourcesContent, [item.source]);
        assert(map.mappings.length > 0);
      }
    }
  },
);

test(
  "complete modules in Chromium preserve empty, nonempty and outside scoped behavior",
  { skip: !captured && !process.env.VIZE_NATIVE_EMPTY_REFERENCE_BROWSER },
  async () => {
    const fromTests = createRequire(new URL("../package.json", import.meta.url));
    const { chromium } = fromTests("@playwright/test");
    const browser = await chromium.launch();
    const runtimeUrl = dataUrl(
      fs.readFileSync(fromVue.resolve("vue/dist/vue.esm-browser.prod.js"), "utf8"),
    );
    const executions: unknown[] = [];
    try {
      for (const [index, fixture] of pack.fixtures.entries()) {
        const actual = captured?.[index] ?? fixture;
        const moduleUrl = dataUrl(relinkRuntime(actual.code, runtimeUrl));
        const page = await browser.newPage();
        const errors: string[] = [];
        page.on("pageerror", (error: Error) => errors.push(error.message));
        await page.setContent("<!doctype html><html><head></head><body></body></html>");
        const execution = await page.evaluate(
          async ({ actual, fixture, runtimeUrl, moduleUrl }: any) => {
            const runtime = await import(runtimeUrl);
            const loaded = await import(moduleUrl);
            const style = document.createElement("style");
            style.textContent = actual.css;
            document.head.append(style);
            const mount = document.createElement("section");
            const outside = document.createElement("p");
            outside.className = fixture.classname;
            document.body.append(mount, outside);
            const app = runtime.createApp(loaded.default);
            const instance = app.mount(mount);
            await runtime.nextTick();
            const element = mount.firstElementChild!;
            const state = () => ({
              text: element.textContent,
              classname: element.className,
              scope: element.getAttribute(actual.scopeId),
              empty: element.matches(":empty"),
              scoped: element.matches(fixture.selector),
              background: getComputedStyle(element).backgroundColor,
              outsideEmpty: outside.matches(":empty"),
              outsideScoped: outside.matches(fixture.selector),
              outsideBackground: getComputedStyle(outside).backgroundColor,
            });
            const states = [state()];
            for (const text of ["visible", ""]) {
              if (fixture.script) {
                instance.$.setupState.text = text;
                instance.$forceUpdate();
                await runtime.nextTick();
              } else element.textContent = text;
              states.push(state());
            }
            const html = mount.innerHTML;
            app.unmount();
            return {
              id: fixture.id,
              scopeId: loaded.default.__scopeId,
              runtime: runtime.version,
              label: fixture.script ? instance.$.setupState.label : null,
              states,
              html,
              afterUnmount: mount.childElementCount,
            };
          },
          { actual, fixture, runtimeUrl, moduleUrl },
        );
        assert.deepEqual(errors, []);
        assert.equal(execution.scopeId, fixture.scopeId);
        assert.equal(execution.runtime, "3.5.35");
        assert.equal(execution.label, fixture.script ? 'from "vue"' : null);
        for (const [stateIndex, state] of execution.states.entries()) {
          const empty = stateIndex !== 1;
          assert.equal(state.text, empty ? "" : "visible");
          assert.equal(state.classname, fixture.classname);
          assert.equal(state.scope, "");
          assert.equal(state.empty, empty);
          assert.equal(state.scoped, empty);
          assert.equal(state.background, empty ? "rgb(0, 0, 255)" : "rgba(0, 0, 0, 0)");
          assert.equal(state.outsideEmpty, true);
          assert.equal(state.outsideScoped, false);
          assert.equal(state.outsideBackground, "rgba(0, 0, 0, 0)");
        }
        assert.equal(execution.afterUnmount, 0);
        executions.push(execution);
        await page.close();
      }
      if (process.env.VIZE_NATIVE_EMPTY_CSS_RUNTIME_CAPTURE)
        fs.writeFileSync(
          process.env.VIZE_NATIVE_EMPTY_CSS_RUNTIME_CAPTURE,
          `${JSON.stringify({ runtime: pack.runtime, browser: browser.version(), currentSource: !!captured, executions }, null, 2)}\n`,
        );
    } finally {
      await browser.close();
    }
  },
);
