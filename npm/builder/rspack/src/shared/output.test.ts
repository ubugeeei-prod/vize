import { test } from "node:test";
import assert from "node:assert/strict";
import type { CompiledModule, TemplateAssetUrl } from "../types/index.ts";
import { generateOutput } from "./output.ts";

function compiledModule(
  code: string,
  overrides: Partial<Pick<CompiledModule, "templateAssetUrls">> = {},
): CompiledModule {
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
