import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import {
  vueConfigOptions,
  mergeVueConfigOptions,
} from "../../crates/vize_carton/src/config/loader/vite-runtime.mjs";
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
    const declaration = /^class\s+AttributeHyphenationOptions\s*\{([\s\S]*?)^\}/mu.exec(value);
    assert(declaration);
    const tokens = (source: string) =>
      source.match(/"(?:\\.|[^"\\])*"|`[^`]*`|[A-Za-z_][\w-]*|[^\s]/gu);
    assert.deepEqual(
      tokens(declaration[1]!),
      tokens('style: HyphenationStyle = "always" ignore: Listing<String> = new Listing {}'),
    );
    const entry = /`vue\/attribute-hyphenation`\s*:\s*([\s\S]*?=\s*null)/u.exec(value);
    assert(entry);
    assert.deepEqual(
      tokens(entry[1]!),
      tokens("(HyphenationStyle|AttributeHyphenationOptions)? = null"),
    );
  }
});

test("attribute policy overlays reset the entire inherited policy while preserving peer configuration", () => {
  const base = {
    compiler: { sourceMap: true },
    linter: {
      rules: { [rule]: "warn" },
      ruleOptions: {
        [rule]: { style: "never", ignore: ["inherited"] },
        "vue/v-bind-style": "longform",
      },
    },
  };
  const before = structuredClone(base);
  for (const policy of [{}, { ignore: [] }, "always"]) {
    const own = { linter: { ruleOptions: { [rule]: policy } } };
    const ownBefore = structuredClone(own);
    assert.deepEqual(mergeVueConfigOptions<unknown>(base, own), {
      compiler: base.compiler,
      linter: {
        rules: base.linter.rules,
        ruleOptions: { [rule]: policy, "vue/v-bind-style": "longform" },
      },
    });
    assert.deepEqual(own, ownBefore);
  }
  assert.deepEqual(
    mergeVueConfigOptions<unknown>(base, { linter: { ruleOptions: { [rule]: undefined } } }),
    base,
  );
  assert.deepEqual(base, before);
});
