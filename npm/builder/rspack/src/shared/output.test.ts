import { test } from "node:test";
import assert from "node:assert/strict";
import type { CompiledModule } from "../types/index.ts";
import { generateOutput } from "./output.ts";

function compiledModule(code: string): CompiledModule {
  return {
    code,
    errors: [],
    warnings: [],
    scopeId: "test1234",
    hasScoped: false,
    styles: [],
    customBlocks: [],
    isCustomElement: false,
    templateAssetUrls: [{ url: "./logo.png", varName: "_imports_0" }],
    macroArtifacts: [],
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
