import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { vueConfigOptions } from "../../crates/vize_carton/src/config/loader/vite-runtime.mjs";
import { corpus, root, rule } from "./support/patina-attribute-ignore-contract.ts";

test("canonical attribute style and custom-ignore policies roundtrip without mutation", () => {
  for (const policy of [
    "always",
    "never",
    {},
    ...corpus.valid.map((row) => row.policy),
    { ignore: ["\u0085", "\u200b", "name:exit\n"] },
  ]) {
    const config = { vue: { linter: { ruleOptions: { [rule]: policy } } } };
    const before = JSON.stringify(config);
    assert.deepEqual(vueConfigOptions(config), config.vue);
    assert.equal(JSON.stringify(config), before);
  }
  for (const policy of [
    ...corpus.invalid.map((row) => ({ ignore: row.ignore })),
    { style: null },
    { style: "sometimes" },
    { ignore: null },
    { ignore: "active" },
    { ignoreTags: [] },
  ])
    assert.throws(
      () => vueConfigOptions({ vue: { linter: { ruleOptions: { [rule]: policy } } } }),
      TypeError,
    );
});

test("published schema/types/PKL declare the attribute option while keeping the original style form", () => {
  const schema = JSON.parse(
    fs.readFileSync(path.join(root, "npm/cli/schemas/vize.config.schema.json"), "utf8"),
  );
  const option = schema.definitions.LintRuleOptions.properties[rule];
  assert.deepEqual(option.oneOf[0], { type: "string", enum: ["always", "never"] });
  assert.deepEqual(Object.keys(option.oneOf[1].properties).sort(), ["ignore", "style"]);
  assert.equal(option.oneOf[1].additionalProperties, false);
  assert.equal(option.oneOf[1].properties.ignore.uniqueItems, true);
  for (const file of ["npm/cli/pkl/LinterConfig.pkl", "npm/cli/pkl/vize.pkl"]) {
    const value = fs.readFileSync(path.join(root, file), "utf8");
    assert(value.includes("class AttributeHyphenationOptions {"));
    assert(
      value.includes(
        "`vue/attribute-hyphenation`: (HyphenationStyle|AttributeHyphenationOptions)? = null",
      ),
    );
  }
});
