import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import {
  extractVueScripts,
  packageNameOfSpecifier,
  scanCssImports,
  scanModuleSpecifiers,
  stripComments,
} from "../../scripts/source-registry-bundle/imports.ts";
import { validateJsonSchemaSubset } from "../../scripts/source-registry-bundle/json-schema-subset.ts";

test("scans static, type-only, re-export, side-effect, and dynamic imports", () => {
  const source = [
    'import { ref } from "vue";',
    'import type { Props } from "./props-types.ts";',
    "import {",
    "  first,",
    "  type Second,",
    '} from "../shared/first.ts";',
    'export { third } from "./third.ts";',
    'export * as all from "./all.ts";',
    'import "./side-effect.css";',
    'const lazy = () => import("./lazy.ts");',
  ].join("\n");

  assert.deepEqual(scanModuleSpecifiers("entry.ts", source), [
    "vue",
    "./props-types.ts",
    "../shared/first.ts",
    "./third.ts",
    "./all.ts",
    "./side-effect.css",
    "./lazy.ts",
  ]);
});

test("ignores imports inside comments and non-import strings", () => {
  const source = [
    "/**",
    " * @example",
    ' * import { Rating } from "@vizejs/ui/rating";',
    " */",
    '// import { gone } from "./gone.ts";',
    'const label = "from ./nowhere";',
    "const pattern = /\"import x from 'y'\"/g;",
    'export const url = "https://example.com/a//b";',
    'import { kept } from "./kept.ts";',
  ].join("\n");

  assert.deepEqual(scanModuleSpecifiers("entry.ts", source), ["./kept.ts"]);
});

test("keeps strings that contain comment markers intact", () => {
  assert.equal(stripComments('const a = "//not"; // gone'), 'const a = "//not"; ');
  assert.equal(stripComments("const b = `/* x */`;"), "const b = `/* x */`;");
});

test("scans only the script blocks of Vue single-file components", () => {
  const source = [
    '<script lang="ts">',
    'export { helper } from "./helper.ts";',
    "</script>",
    '<script setup lang="ts" generic="T">',
    'import Child from "./child.vue";',
    "</script>",
    '<template><div>import("./template-only.ts")</div></template>',
  ].join("\n");

  assert.match(extractVueScripts(source), /helper/);
  assert.deepEqual(scanModuleSpecifiers("component.vue", source), ["./helper.ts", "./child.vue"]);
});

test("scans CSS @import targets", () => {
  assert.deepEqual(scanCssImports('/* @import "./no.css"; */ @import "./tokens.css";'), [
    "./tokens.css",
  ]);
});

test("maps bare specifiers to npm package names", () => {
  assert.equal(packageNameOfSpecifier("vue"), "vue");
  assert.equal(packageNameOfSpecifier("vue/server-renderer"), "vue");
  assert.equal(packageNameOfSpecifier("@scope/name/sub/path"), "@scope/name");
});

test("the schema subset validator rejects shape violations", () => {
  const schema = {
    type: "object",
    additionalProperties: false,
    required: ["name"],
    properties: { name: { type: "string", pattern: "^[a-z]+$" } },
  };
  assert.deepEqual(validateJsonSchemaSubset(schema, { name: "ok" }), []);
  assert.equal(validateJsonSchemaSubset(schema, { name: "NO", extra: 1 }).length, 2);
  assert.equal(validateJsonSchemaSubset({ format: "uri" }, "x").length, 1);
});
