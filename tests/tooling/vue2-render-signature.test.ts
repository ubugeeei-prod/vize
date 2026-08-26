import assert from "node:assert/strict";
import { createRequire } from "node:module";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import {
  collectProjectVueFiles,
  resolveGlyphLaunch,
  selectGlyphCorpusProjects,
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
const registry = JSON.parse(
  fs.readFileSync(path.join(root, "tests/_fixtures/vue-ecosystem-fixtures.json"), "utf8"),
) as {
  projects: Array<{
    coverage: string[];
    expectedVueFileCount: number | null;
    id: string;
    sfcDialectRoutes?: SfcDialectRoute[];
  }>;
};

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

test("GoGoCode render signature corpus is optional outside its matrix shard", () => {
  const gogocodeIndex = registry.projects.findIndex((project) => project.id === "gogocode");
  assert.notEqual(gogocodeIndex, -1, "the registry must keep the GoGoCode fixture");
  const shardCount = registry.projects.length;
  const excludedShardIndex = (gogocodeIndex + 1) % shardCount;

  const selected = resolveSelectedGogocodeVue2Route({
    FIXTURE_SHARD_COUNT: String(shardCount),
    FIXTURE_SHARD_INDEX: String(excludedShardIndex),
  });

  assert.equal(selected.sharded, true);
  assert.equal(selected.project, undefined);
  assert.equal(selected.vue2Route, undefined);
});

test("the hydrated GoGoCode Vue 2 partition has stable render signatures for all 97 SFCs", (t) => {
  const selected = resolveSelectedGogocodeVue2Route();
  if (selected.project == null && selected.sharded) {
    t.skip("the GoGoCode fixture is not selected by this real-project matrix shard");
    return;
  }
  assert.ok(selected.project, "the GoGoCode fixture must be selected outside matrix sharding");
  assert.ok(selected.vue2Route, "the GoGoCode fixture must declare its Vue 2 route");

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
    vueGlobs: selected.vue2Route.globs,
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

function resolveSelectedGogocodeVue2Route(environment = process.env): {
  project:
    | {
        id: string;
        sfcDialectRoutes?: SfcDialectRoute[];
      }
    | undefined;
  sharded: boolean;
  vue2Route: SfcDialectRoute | undefined;
} {
  const sharded =
    environment.FIXTURE_SHARD_INDEX != null || environment.FIXTURE_SHARD_COUNT != null;
  const project = (
    selectGlyphCorpusProjects(registry.projects, environment) as Array<{
      id: string;
      sfcDialectRoutes?: SfcDialectRoute[];
    }>
  ).find((candidate) => candidate.id === "gogocode");
  return {
    project,
    sharded,
    vue2Route: project?.sfcDialectRoutes?.find((route) => route.id === "vue2"),
  };
}
