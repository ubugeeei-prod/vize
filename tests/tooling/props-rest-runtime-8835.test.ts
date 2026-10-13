import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { test } from "node:test";
import { compileFunction } from "node:vm";

const require = createRequire(import.meta.url);
const native = require("../../npm/native/index.js");
const fromUi = createRequire(new URL("../../npm/ui/package.json", import.meta.url));
const vue = fromUi("vue");
const ts = fromUi("typescript");
const fixture = new URL(
  "../_fixtures/differential/compiler/props-rest-runtime-8835/",
  import.meta.url,
);

for (const inlineTemplate of [false, true]) {
  test(`reported rest binding and Vue getter semantics, inline=${inlineTemplate}`, () => {
    for (const name of ["Child", "Alias", "RestOnly", "FollowedByCall"]) {
      const source = readFileSync(new URL(`${name}.vue.txt`, fixture), "utf8");
      const compiled = native.compileSfc(source, { filename: `${name}.vue`, inlineTemplate });
      assert.deepEqual(compiled.errors, []);
      const output = ts.transpileModule(compiled.code, {
        compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2022 },
      }).outputText;
      const exports: Record<string, any> = {};
      let observedRest: Record<string, unknown> | undefined;
      const runtime = {
        ...vue,
        createPropsRestProxy(...args: any[]) {
          observedRest = vue.createPropsRestProxy(...args);
          return observedRest;
        },
      };
      compileFunction(output, ["require", "exports"])((id: string) => {
        assert.equal(id, "vue");
        return runtime;
      }, exports);
      const component = exports.default;
      const props = vue.reactive({ label: "Name", "aria-label": "Name", placeholder: "Type here" });
      const setup = component.setup(props, { expose() {} });
      assert(observedRest);
      assert.equal(
        Object.hasOwn(observedRest, name === "Alias" ? "aria-label" : "label"),
        name === "RestOnly",
      );
      assert.equal(observedRest.placeholder, "Type here");
      if (typeof setup === "function") {
        assert.equal(setup({}, []).props.placeholder, "Type here", `${name}: ${compiled.code}`);
        props.placeholder = "Changed";
        assert.equal(setup({}, []).props.placeholder, "Changed");
      } else {
        const rest = setup.rest ?? setup.attrs;
        assert.equal(rest.placeholder, "Type here");
        assert.equal(
          Object.hasOwn(rest, name === "Alias" ? "aria-label" : "label"),
          name === "RestOnly",
        );
        props.placeholder = "Changed";
        assert.equal(rest.placeholder, "Changed");
        assert.equal(typeof component.render, "function", `${name}: ${compiled.code}`);
        const render = () => component.render({}, [], props, setup, {}, {});
        assert.equal(render().props.placeholder, "Changed", `${name}: ${compiled.code}`);
        props.placeholder = "Final";
        assert.equal(render().props.placeholder, "Final", `${name}: ${compiled.code}`);
      }
    }
  });
}
