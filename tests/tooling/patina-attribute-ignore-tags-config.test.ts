import assert from "node:assert/strict";
import { test } from "node:test";
import { vueConfigOptions } from "../../crates/vize_carton/src/config/loader/vite-runtime.mjs";

const rule = "vue/attribute-hyphenation";
test("canonical literal tag options retain nonmatching strings and refuse regex policy evaluation", () => {
  for (const ignoreTags of [
    [],
    ["Foo.Bar", "ns:Widget"],
    ["", " ", ":exit", "//", "/name", "/x/\n"],
  ]) {
    const vue = { linter: { ruleOptions: { [rule]: { ignoreTags } } } };
    assert.deepEqual(vueConfigOptions({ vue }), vue);
  }
  for (const ignoreTags of [["/^My/"], ["/x/y"], ["/[/"]])
    assert.throws(
      () => vueConfigOptions({ vue: { linter: { ruleOptions: { [rule]: { ignoreTags } } } } }),
      /ignoreTags supports literal tag names only; regular-expression policies are unsupported/,
    );
});

test("literal tag validation is owned by the actual attribute rule schema", () => {
  const vue = { experimentals: { plugin: { ruleOptions: { [rule]: { ignoreTags: ["/^My/"] } } } } };
  assert.deepEqual(vueConfigOptions({ vue }), vue);
});
