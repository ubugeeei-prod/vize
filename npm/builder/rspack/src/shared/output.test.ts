import { test } from "node:test";
import assert from "node:assert/strict";
import { encodedMap, originalPositionFor, TraceMap } from "@jridgewell/trace-mapping";
import { compileSfc } from "@vizejs/native";
import type { CompiledModule, TemplateAssetUrl } from "../types/index.ts";
import { generateOutput, generateOutputWithMap } from "./output.ts";

/** The native shape of `code`: a script-only SFC is emitted verbatim. */
function nativeShape(code: string) {
  const result = compileSfc(`<script>${code}</script>`, { filename: "/src/App.vue" });
  assert.equal(result.code, code, "the fixture is emitted verbatim");
  return result.moduleShape;
}

function hmrModuleId(code: string): string {
  const match = code.match(/"module":"([0-9a-f]{16})"/);
  assert.ok(match, "HMR metadata did not include a module id");
  return match[1];
}

function compiledModule(code: string, overrides: Partial<CompiledModule> = {}): CompiledModule {
  return {
    code,
    moduleShape: overrides.moduleShape ?? nativeShape(code),
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

void test("HMR snapshots combine native and host-generated import facts", () => {
  const code =
    'import child from "./Child.vue";\nconst _sfc_main = { child };\nexport default _sfc_main;';
  const shape = nativeShape(code);
  assert.ok(shape);
  const output = generateOutput(
    compiledModule(code, {
      moduleShape: {
        ...shape,
        imports: [{ source: "./Child.vue", local: "child", namespace: false }],
        hasSideEffectImport: false,
        hasReexport: false,
        hasDynamicImport: false,
      },
      customBlocks: [{ type: "i18n", content: "{}", attrs: {}, index: 0 }],
    }),
    { requestPath: "./App.vue", hmr: true },
  );
  assert.ok(
    output.includes(
      '[["./Child.vue:child", child], ["./App.vue?vue&type=i18n&index=0:block0", block0], ["./logo.png:_imports_0", _imports_0]]',
    ),
  );
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

void test("template-only render output gains a default component export", () => {
  const output = generateOutput(
    compiledModule(`export function render(_ctx, _cache) {\n  return null;\n}`, {
      hasScoped: true,
      templateAssetUrls: [],
      // A template-only client compile's native shape.
      moduleShape: {
        hasDefaultExport: false,
        hasSfcMainDefined: false,
        hasNamedRenderExport: true,
        hasNamedSsrRenderExport: false,
        defaultExportIsSfcMain: false,
        imports: [],
        hasSideEffectImport: false,
        hasReexport: false,
        hasDynamicImport: false,
      },
    }),
    { requestPath: "./App.vue" },
  );

  assert.equal(
    output,
    `export function render(_ctx, _cache) {\n  return null;\n}\nconst _sfc_main = { render, __scopeId: "data-v-test1234" };\nexport default _sfc_main;`,
  );
});

void test("CSS module class maps do not change HMR identity when script and template are unchanged", () => {
  const hmr = {
    source: "source-a",
    script: "script-a",
    template: "template-a",
    options: "options-a",
    canRerender: true,
  };
  const styles = [
    {
      content: ".button { composes: variantOne; padding: 7px; }",
      lang: "css",
      scoped: false,
      module: true,
      index: 1,
    },
    {
      content: ".label { composes: thin; }",
      lang: "css",
      scoped: false,
      module: "theme",
      index: 2,
    },
  ];
  const before = generateOutput(
    compiledModule('export default { __cssModules: { theme: { label: "offset-a" } } };', {
      templateAssetUrls: [],
      hmr,
      styles,
    }),
    { requestPath: "./App.vue", hmr: true },
  );
  const shiftedClasses = generateOutput(
    compiledModule('export default { __cssModules: { theme: { label: "offset-b" } } };', {
      templateAssetUrls: [],
      hmr: { ...hmr, source: "source-b" },
      styles,
    }),
    { requestPath: "./App.vue", hmr: true },
  );
  assert.notEqual(before, shiftedClasses);
  assert.equal(hmrModuleId(before), hmrModuleId(shiftedClasses));

  const templateEdit = generateOutput(
    compiledModule('export default { __cssModules: { theme: { label: "offset-b" } } };', {
      templateAssetUrls: [],
      hmr: { ...hmr, source: "source-c", template: "template-b" },
      styles,
    }),
    { requestPath: "./App.vue", hmr: true },
  );
  assert.notEqual(hmrModuleId(before), hmrModuleId(templateEdit));

  const plain = generateOutput(
    compiledModule("export default { name: 'plain-a' };", {
      templateAssetUrls: [],
      hmr,
    }),
    { requestPath: "./App.vue", hmr: true },
  );
  const plainEdit = generateOutput(
    compiledModule("export default { name: 'plain-b' };", {
      templateAssetUrls: [],
      hmr: { ...hmr, source: "source-b" },
    }),
    { requestPath: "./App.vue", hmr: true },
  );
  assert.notEqual(hmrModuleId(plain), hmrModuleId(plainEdit));
});

void test("a hand-built module without moduleShape fails with the compileFile contract", () => {
  const withoutShape = compiledModule("const _sfc_main = {};\nexport default _sfc_main;\n");
  delete withoutShape.moduleShape;
  assert.throws(() => generateOutput(withoutShape, { requestPath: "./App.vue" }), {
    message:
      "[vize] generateOutput requires moduleShape; pass the CompiledModule returned by compileFile",
  });
});
