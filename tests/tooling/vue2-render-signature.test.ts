import assert from "node:assert/strict";
import { createRequire } from "node:module";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import {
  collectProjectVueFiles,
  loadGlyphCorpusProjects,
  resolveGlyphLaunch,
  withFormattedWorkspace,
} from "../../tools/fixtures/glyph-corpus.mjs";
import type { SfcDialectRoute } from "./support/sfc-baseline-routes.ts";
import {
  vue27RenderCodeSignature,
  vue2RenderFunctionSignature,
  vue2RenderSignature,
} from "./support/vue2-render-signature.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const gogocodeRoot = path.join(root, "tests/_fixtures/_git/gogocode");
const require = createRequire(import.meta.url);

const gogocodeProject = (
  loadGlyphCorpusProjects() as Array<{ id: string; sfcDialectRoutes?: SfcDialectRoute[] }>
).find((project) => project.id === "gogocode");
const vue2Route = gogocodeProject?.sfcDialectRoutes?.find((route) => route.id === "vue2");
assert.ok(vue2Route, "the GoGoCode fixture must declare its Vue 2 route");
const vue2Globs = vue2Route.globs;

test("Vue 2 render signatures normalize only pure attribute layout", () => {
  const before = `with(this){return _c('p',{attrs:{"name":"slide","appear":""},on:{"focus":function(){return focus()},"click":function(){return click()}}},[_v("Vue版本："+_s(version))])}`;
  const after = `with(this){return _c("p",{attrs:{appear:"",name:"slide"},on:{click:function(){return click()},focus:function(){return focus()}}},[_v("Vue版本："+_s(version))])}`;

  assert.deepEqual(vue2RenderFunctionSignature(before), vue2RenderFunctionSignature(after));
  assert.notDeepEqual(
    vue2RenderFunctionSignature(before),
    vue2RenderFunctionSignature(
      `with(this){return _c("p",{attrs:{appear:"",name:"slide"},on:{click:function(){return click()},focus:function(){return focus()}}},[_v(" Vue版本："+_s(version)+" ")])}`,
    ),
  );
});

test("Vue 2 render signatures preserve direct binding evaluation order", () => {
  const before = `with(this){return _c('Widget',{attrs:{"a":first(),"b":second()}})}`;
  const after = `with(this){return _c('Widget',{attrs:{"b":second(),"a":first()}})}`;
  const getterBefore = `with(this){return _c('Widget',{attrs:{"a":first,"b":second}})}`;
  const getterAfter = `with(this){return _c('Widget',{attrs:{"b":second,"a":first}})}`;

  assert.notDeepEqual(vue2RenderFunctionSignature(before), vue2RenderFunctionSignature(after));
  assert.notDeepEqual(
    vue2RenderFunctionSignature(getterBefore),
    vue2RenderFunctionSignature(getterAfter),
  );
});

test("Vue 2.7 module render signatures use the same scoped normalization", () => {
  const before = `var render = function render(){var _vm=this,_c=_vm._self._c;return _c('p',{attrs:{"name":"slide","appear":""}},[_vm._v("Vue版本："+_vm._s(_vm.version))])}\nvar staticRenderFns = []`;
  const after = `var render = function render(){var _vm=this,_c=_vm._self._c;return _c("p",{attrs:{appear:"",name:"slide"}},[_vm._v("Vue版本："+_vm._s(_vm.version))])}\nvar staticRenderFns = []`;
  assert.deepEqual(vue27RenderCodeSignature(before), vue27RenderCodeSignature(after));
  assert.notDeepEqual(
    vue27RenderCodeSignature(before),
    vue27RenderCodeSignature(
      `var render = function render(){var _vm=this,_c=_vm._self._c;return _c("p",{attrs:{appear:"",name:"slide"}},[_vm._v(" Vue版本："+_vm._s(_vm.version)+" ")])}\nvar staticRenderFns = []`,
    ),
  );

  assert.notDeepEqual(
    vue27RenderCodeSignature(
      `var render = function render(){var _vm=this,_c=_vm._self._c;return _c('pre',[_vm._v("  keep   me  ")])}\nvar staticRenderFns = []`,
    ),
    vue27RenderCodeSignature(
      `var render = function render(){var _vm=this,_c=_vm._self._c;return _c('pre',[_vm._v(" keep me ")])}\nvar staticRenderFns = []`,
    ),
  );
});

test("Vue 2 render signatures preserve user object evaluation order", () => {
  const before = `with(this){return _c('Widget',{attrs:{"value":{a:first(),b:second()}},on:{"click":function(){return consume({a:first(),b:second()})}}})}`;
  const afterValue = `with(this){return _c('Widget',{attrs:{"value":{b:second(),a:first()}},on:{"click":function(){return consume({a:first(),b:second()})}}})}`;
  const afterHandler = `with(this){return _c('Widget',{attrs:{"value":{a:first(),b:second()}},on:{"click":function(){return consume({b:second(),a:first()})}}})}`;

  assert.notDeepEqual(vue2RenderFunctionSignature(before), vue2RenderFunctionSignature(afterValue));
  assert.notDeepEqual(
    vue2RenderFunctionSignature(before),
    vue2RenderFunctionSignature(afterHandler),
  );
});

test("Vue 2 render signatures preserve spread helper order", () => {
  const before = `with(this){return _c('Widget',_g(_b({attrs:{"id":"x"}},'Widget',props,false),listeners))}`;
  const after = `with(this){return _c('Widget',_b(_g({attrs:{"id":"x"}},listeners),'Widget',props,false))}`;

  assert.notDeepEqual(vue2RenderFunctionSignature(before), vue2RenderFunctionSignature(after));
});

test("Vue 2 render signatures preserve text inside pre-like ancestors", () => {
  for (const tag of ["pre", "textarea", "listing"]) {
    const before = `with(this){return _c('${tag}',[_c('span',[_v("  keep   me  ")])])}`;
    const after = `with(this){return _c('${tag}',[_c('span',[_v(" keep me ")])])}`;
    assert.notDeepEqual(
      vue2RenderFunctionSignature(before),
      vue2RenderFunctionSignature(after),
      tag,
    );
  }

  assert.deepEqual(
    vue2RenderFunctionSignature(`with(this){return _c('span',[_v("  keep   me  ")])}`),
    vue2RenderFunctionSignature(`with(this){return _c('span',[_v(" keep me ")])}`),
  );
  assert.deepEqual(
    vue2RenderFunctionSignature(`with(this){return _c('Pre',[_v("  layout   text  ")])}`),
    vue2RenderFunctionSignature(`with(this){return _c('Pre',[_v(" layout text ")])}`),
  );
  assert.notDeepEqual(
    vue2RenderFunctionSignature(
      `with(this){return _c('div',{pre:true},[_v("{{  raw   text  }}")])}`,
    ),
    vue2RenderFunctionSignature(`with(this){return _c('div',{pre:true},[_v("{{ raw text }}")])}`),
  );
  assert.notDeepEqual(
    vue2RenderFunctionSignature(
      `with(this){return _c('pre',[_c('span',[_v("  keep   me  ")])],2)}`,
    ),
    vue2RenderFunctionSignature(`with(this){return _c('pre',[_c('span',[_v(" keep me ")])],2)}`),
    "a data-less createElement call must not mistake normalization type 2 for children",
  );
});

test("Vue 2 render signatures preserve interpolation boundary whitespace", () => {
  const spaced = `with(this){return _c('p',[_v("Hello "+_s(name)+" world")])}`;
  const missingBefore = `with(this){return _c('p',[_v("Hello"+_s(name)+" world")])}`;
  const missingAfter = `with(this){return _c('p',[_v("Hello "+_s(name)+"world")])}`;

  assert.notDeepEqual(
    vue2RenderFunctionSignature(spaced),
    vue2RenderFunctionSignature(missingBefore),
  );
  assert.notDeepEqual(
    vue2RenderFunctionSignature(spaced),
    vue2RenderFunctionSignature(missingAfter),
  );
  assert.notDeepEqual(
    vue2RenderFunctionSignature(spaced),
    vue2RenderFunctionSignature(`with(this){return _c('p',[_v(" Hello "+_s(name)+" world ")])}`),
  );
  assert.deepEqual(
    vue2RenderFunctionSignature(`with(this){return _c('p',[_v("Hello  "+_s(name)+"  world")])}`),
    vue2RenderFunctionSignature(spaced),
  );
});

test("Vue 2 render signatures retain semantic handler and static-render changes", () => {
  const numeric32 = `with(this){return _c('input',{on:{"keyup":function($event){if($event.keyCode!==32)return null;return keys()}}})}`;
  const numeric113 = `with(this){return _c('input',{on:{"keyup":function($event){if($event.keyCode!==113)return null;return keys()}}})}`;
  assert.notDeepEqual(
    vue2RenderFunctionSignature(numeric32),
    vue2RenderFunctionSignature(numeric113),
  );

  assert.notDeepEqual(
    vue2RenderSignature("with(this){return _m(0)}", ["with(this){return _c('p',[_v(\"a\")])}"]),
    vue2RenderSignature("with(this){return _m(0)}", ["with(this){return _c('p',[_v(\"b\")])}"]),
  );
});

test("the hydrated GoGoCode Vue 2 partition has stable render signatures for all 97 SFCs", (t) => {
  if (!fs.existsSync(gogocodeRoot) || fs.readdirSync(gogocodeRoot).length === 0) {
    t.skip("the GoGoCode fixture submodule is not hydrated");
    return;
  }

  const compiler = require("vue-sfc-compiler-2-6/build.js") as {
    compile(
      source: string,
      options: object,
    ): {
      errors: unknown[];
      render: string;
      staticRenderFns: string[];
    };
    parseComponent(
      source: string,
      options: object,
    ): {
      template: { content: string } | null;
    };
  };
  const project = {
    id: "gogocode-vue2-render-signature",
    fixtureDir: gogocodeRoot,
    hydrated: true,
    vueGlobs: vue2Globs,
  };
  const files = collectProjectVueFiles(project) as string[];
  assert.equal(files.length, 97, "the exact Vue 2 route partition must remain complete");

  let compared = 0;
  withFormattedWorkspace(project, files, resolveGlyphLaunch(), ({ workspaceDir }) => {
    for (const file of files) {
      const before = compileTemplate(path.join(gogocodeRoot, file), compiler);
      const after = compileTemplate(path.join(workspaceDir, file), compiler);
      assert.deepEqual(after, before, file);
      compared += 1;
    }
  });
  assert.equal(compared, 97);
});

function compileTemplate(
  file: string,
  compiler: {
    compile(
      source: string,
      options: object,
    ): {
      errors: unknown[];
      render: string;
      staticRenderFns: string[];
    };
    parseComponent(
      source: string,
      options: object,
    ): {
      template: { content: string } | null;
    };
  },
): unknown {
  const descriptor = compiler.parseComponent(fs.readFileSync(file, "utf8"), { pad: false });
  if (descriptor.template == null) return null;
  const result = compiler.compile(descriptor.template.content, {
    comments: true,
    outputSourceRange: true,
    whitespace: "preserve",
  });
  assert.deepEqual(result.errors, [], `${file}: official Vue 2.6 baseline rejected the template`);
  return vue2RenderSignature(result.render, result.staticRenderFns);
}
