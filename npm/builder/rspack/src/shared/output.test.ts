import { test } from "node:test";
import assert from "node:assert/strict";
import { encodedMap, originalPositionFor, TraceMap } from "@jridgewell/trace-mapping";
import type { CompiledModule, TemplateAssetUrl } from "../types/index.ts";
import { generateOutput, generateOutputWithMap } from "./output.ts";

function compiledModule(code: string, overrides: Partial<CompiledModule> = {}): CompiledModule {
  return {
    code,
    errors: [],
    warnings: [],
    scopeId: "test1234",
    hasScoped: false,
    styles: [],
    customBlocks: [],
    isCustomElement: false,
    templateAssetUrls: [{ url: "./logo.png", varName: "_imports_0" }] satisfies TemplateAssetUrl[],
    macroArtifacts: [],
    ...overrides,
  };
}

void test("template asset rewrite leaves same-valued script strings intact", () => {
  const output = generateOutput(
    compiledModule(`
const same = "./logo.png";
const _sfc_main = {};
const _hoisted_1 = { src: "./logo.png" };
function _sfc_render() {
  return _createElementVNode("img", { src: "./logo.png" });
}
_sfc_main.render = _sfc_render;
export default _sfc_main;
`),
    { requestPath: "./App.vue" },
  );

  assert.match(output, /import _imports_0 from "\.\/logo\.png";/);
  assert.match(output, /const same = "\.\/logo\.png";/);
  assert.match(output, /const _hoisted_1 = \{ src: _imports_0 \};/);
  assert.match(output, /_createElementVNode\("img", \{ src: _imports_0 \}\)/);
});

void test("template asset rewrite converts SSR template literals to concatenations", () => {
  const output = generateOutput(
    compiledModule(`
const same = "./logo.png";
const _sfc_main = {};
function ssrRender(_ctx, _push) {
  _push(\`<img src="./logo.png">\`);
}
_sfc_main.ssrRender = ssrRender;
export default _sfc_main;
`),
    { requestPath: "./App.vue" },
  );

  assert.match(output, /const same = "\.\/logo\.png";/);
  assert.ok(output.includes('_push("<img src=\\"" + _imports_0 + "\\">")'));
});

void test("output export rewrite ignores export default text inside template literals", () => {
  const output = generateOutput(
    compiledModule(
      ["const message = `", "export default fake", "`;", "export default { name: 'Real' };"].join(
        "\n",
      ),
      { templateAssetUrls: [] },
    ),
    { requestPath: "./App.vue" },
  );

  assert.match(output, /export default fake/);
  assert.doesNotMatch(output, /const _sfc_main = fake/);
  assert.match(output, /const _sfc_main = \{ name: 'Real' \};/);
  assert.match(output, /export default _sfc_main;/);
});

void test("output export rewrite preserves pure annotations on default exports", () => {
  const output = generateOutput(
    compiledModule(
      [
        'import { defineComponent } from "vue";',
        "export default /*#__PURE__*/ defineComponent({ name: 'Annotated' });",
      ].join("\n"),
      { templateAssetUrls: [] },
    ),
    { requestPath: "./App.vue" },
  );

  assert.match(output, /const _sfc_main = \/\*#__PURE__\*\/ defineComponent/);
  assert.match(output, /export default _sfc_main;/);
});

void test("output source maps follow inserted SFC lines", () => {
  const output = generateOutputWithMap(
    compiledModule(
      ["const _sfc_main = {};", "_sfc_main.render = () => null;", "export default _sfc_main;"].join(
        "\n",
      ),
      {
        map: JSON.stringify({
          version: 3,
          sources: ["App.vue"],
          names: [],
          mappings: "AAAA;AACA;AACA",
        }),
        templateAssetUrls: [],
        styles: [
          {
            content: ".app { color: red; }",
            lang: "css",
            scoped: false,
            module: false,
            index: 0,
          },
        ],
      },
    ),
    {
      requestPath: "./App.vue",
      filePath: "/workspace/src/App.vue",
      rootContext: "/workspace",
      hmr: true,
    },
  );

  assert.ok(output.map);
  assert.match(output.code, /__hmrId/);
  assert.ok(output.map!.mappings.split(";").length > 3);
  assert.deepEqual(output.map!.sources, ["App.vue"]);
  const lines = output.code.split("\n");
  const mappingLines = output.map!.mappings.split(";");
  const renderLine = lines.findIndex((line) => line.includes("_sfc_main.render"));
  const styleImportLine = lines.findIndex((line) => line.includes("type=style"));
  assert.notEqual(renderLine, -1);
  assert.notEqual(styleImportLine, -1);
  assert.notEqual(mappingLines[renderLine], "", "original render code keeps its mapping");
  assert.equal(mappingLines[styleImportLine], "", "injected style imports stay unmapped");
});

void test("drops malformed source maps at the output boundary", () => {
  const output = generateOutputWithMap(
    compiledModule("export default {};", {
      map: "{not-json}",
      templateAssetUrls: [],
    }),
    { requestPath: "./App.vue" },
  );
  assert.equal(output.map, null);
});

void test("uses CSS module namespace interop for native and extracted CSS", () => {
  const output = generateOutput(
    compiledModule("export default {};", {
      templateAssetUrls: [],
      styles: [
        {
          content: ".button { color: red; }",
          lang: "css",
          scoped: false,
          module: true,
          index: 0,
        },
      ],
    }),
    { requestPath: "./App.vue", nativeCss: false },
  );

  assert.match(output, /import \* as _cssModule_0/);
  assert.match(output, /typeof value\.default === "object"/);
  assert.match(output, /__vize_resolve_css_module__\(_cssModule_0\)/);

  const nativeOutput = generateOutput(
    compiledModule("export default {};", {
      templateAssetUrls: [],
      styles: [
        {
          content: ".button { color: red; }",
          lang: "css",
          scoped: false,
          module: true,
          index: 0,
        },
      ],
    }),
    { requestPath: "./App.vue", nativeCss: true },
  );
  assert.doesNotMatch(nativeOutput, /__vize_resolve_css_module__/);
  assert.match(nativeOutput, /__cssModules\["\$style"\] = _cssModule_0/);
});

for (const ssr of [false, true]) {
  void test(`source maps preserve columns between multiple ${ssr ? "SSR" : "VDOM"} asset rewrites`, () => {
    const code = [
      'const note = "🐱"; const _sfc_main = {};',
      ssr
        ? 'function ssrRender(_ctx, _push) { _push(`<img src="./logo.png">${_ctx.middle}<img src="./wide-logo.png">${_ctx.tail}`); }'
        : 'function _sfc_render(_ctx) { return [_createElementVNode("img", { src: "./logo.png", title: _ctx.middle }), _createElementVNode("img", { src: "./wide-logo.png", title: _ctx.tail })]; }',
      "export default _sfc_main;",
    ].join("\n");
    const nativeMap = encodedMap(
      new TraceMap({
        version: 3,
        sources: ["App.vue"],
        sourcesContent: [code],
        names: [],
        mappings: code
          .split("\n")
          .map((line, row) =>
            Array.from({ length: line.length }, (_, column): [number, number, number, number] => [
              column,
              0,
              row,
              column,
            ]),
          ),
      }),
    );
    const output = generateOutputWithMap(
      compiledModule(code, {
        map: JSON.stringify(nativeMap),
        templateAssetUrls: [
          { url: "./logo.png", varName: "_imports_0" },
          { url: "./wide-logo.png", varName: "_imports_1" },
        ],
      }),
      { requestPath: "./App.vue", filePath: "/src/App.vue" },
    );
    assert.ok(output.map);
    const map = new TraceMap(JSON.stringify(output.map));
    for (const marker of ["_ctx.middle", "_ctx.tail"]) {
      const offset = output.code.indexOf(marker);
      assert.notEqual(offset, -1);
      const prefix = output.code.slice(0, offset);
      const original = originalPositionFor(map, {
        line: prefix.split("\n").length,
        column: offset - prefix.lastIndexOf("\n") - 1,
      });
      assert.equal(original.source, "App.vue");
      assert.equal(original.line, 2);
      assert.equal(original.column, code.split("\n")[1].indexOf(marker));
    }
    assert.equal(originalPositionFor(map, { line: 1, column: 0 }).source, null);
  });
}
