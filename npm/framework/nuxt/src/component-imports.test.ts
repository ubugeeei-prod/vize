import assert from "node:assert/strict";
import { test } from "node:test";

import { injectNuxtComponentImports, type NuxtComponentImport } from "./components.ts";

const button: NuxtComponentImport = {
  filePath: "/app/components/Button.vue",
  exportName: "default",
};
const resolve = (name: string) => (name === "BaseButton" ? button : null);

void test("a compiler resolution of a Nuxt component becomes a direct import", () => {
  assert.equal(
    injectNuxtComponentImports('const _component = _resolveComponent("BaseButton");', resolve),
    'import __vize_nuxt_component_0 from "/app/components/Button.vue";\n' +
      "const _component = __vize_nuxt_component_0;",
  );
});

void test("a template call of the auto-imported resolveComponent is left to Nuxt", () => {
  // `<component :is="resolveComponent('BaseButton')" />` compiles to a member
  // call on the render context. Rewriting it named a property that does not exist.
  const code = 'createVNode(resolveDynamicComponent(_ctx.resolveComponent("BaseButton")), null);';
  assert.equal(injectNuxtComponentImports(code, resolve), code);
});

void test("an identifier that only ends in resolveComponent is not a resolution", () => {
  const code =
    'const c = myResolveComponent("BaseButton"); const d = $resolveComponent("BaseButton");';
  assert.equal(injectNuxtComponentImports(code, resolve), code);
});
