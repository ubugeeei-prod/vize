import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import { vueConfigOptions } from "../../crates/vize_carton/src/config/loader/vite-runtime.mjs";

const rule = "script/define-macros-order";
test("the generated canonical graph accepts only boolean defineExposeLast options", () => {
  for (const defineExposeLast of [true, false]) {
    const vue = { linter: { ruleOptions: { [rule]: { defineExposeLast } } } };
    const before = JSON.stringify(vue);
    assert.equal(vueConfigOptions({ vue }), vue);
    assert.equal(JSON.stringify(vue), before);
  }
  for (const defineExposeLast of [null, "true", 1, [], {}]) {
    assert.throws(
      () =>
        vueConfigOptions({ vue: { linter: { ruleOptions: { [rule]: { defineExposeLast } } } } }),
      TypeError,
    );
  }
  assert.throws(
    () =>
      vueConfigOptions({
        vue: { linter: { ruleOptions: { [rule]: { defineExposeLastTypo: true } } } },
      }),
    TypeError,
  );
});

test("public schema and Pkl retain the default-off option alongside the original order", () => {
  const root = new URL("../../", import.meta.url);
  const schema = JSON.parse(
    fs.readFileSync(new URL("npm/cli/schemas/vize.config.schema.json", root), "utf8"),
  ) as {
    definitions: {
      LintRuleOptions: {
        properties: Record<
          string,
          { properties: Record<string, unknown>; additionalProperties: boolean }
        >;
      };
    };
  };
  assert.deepEqual(schema.definitions.LintRuleOptions.properties[rule], {
    type: "object",
    properties: {
      order: {
        default: ["defineProps", "defineEmits"],
        type: "array",
        items: { type: "string", minLength: 1 },
        uniqueItems: true,
      },
      defineExposeLast: { default: false, type: "boolean" },
    },
    additionalProperties: false,
  });
  for (const file of ["LinterConfig.pkl", "vize.pkl"]) {
    const source = fs.readFileSync(new URL("npm/cli/pkl/" + file, root), "utf8");
    const declaration = /^class DefineMacrosOrderOptions \{([\s\S]*?)^\}/mu.exec(source);
    assert.ok(declaration);
    assert.match(declaration[1]!, /defineExposeLast: Boolean = false/u);
  }
});
