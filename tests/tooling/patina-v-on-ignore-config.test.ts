import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import { test } from "node:test";
import type { LintRuleOptions } from "../../npm/cli/src/types/generated.ts";
import {
  vueConfigOptions,
  mergeVueConfigOptions,
  projectViteConfigSource,
} from "../../crates/vize_carton/src/config/loader/vite-runtime.mjs";

type Equal<A, B> =
  (<T>() => T extends A ? 1 : 2) extends <T>() => T extends B ? 1 : 2 ? true : false;
const declarationContract: Equal<
  NonNullable<LintRuleOptions["vue/v-on-event-hyphenation"]>,
  "always" | "never" | { style?: "always" | "never"; ignore?: string[] }
> = true;
const root = new URL("../../", import.meta.url);
const rule = "vue/v-on-event-hyphenation";
const schema = JSON.parse(
  fs.readFileSync(new URL("npm/cli/schemas/vize.config.schema.json", root), "utf8"),
) as {
  definitions: { LintRuleOptions: { properties: Record<string, unknown> } };
};
const optionSchema = schema.definitions.LintRuleOptions.properties[rule];
const expectedSchema = {
  oneOf: [
    { type: "string", enum: ["always", "never"] },
    {
      type: "object",
      properties: {
        style: { default: "always", type: "string", enum: ["always", "never"] },
        ignore: {
          default: [],
          type: "array",
          items: { type: "string", pattern: "^(?!\\s*$)(?![\\s\\S]*:exit$)" },
          uniqueItems: true,
        },
      },
      additionalProperties: false,
    },
  ],
};
const require = createRequire(new URL("../../package.json", import.meta.url));
const Ajv = require("ajv") as new (options: {
  strict: boolean;
  allErrors: boolean;
  useDefaults: boolean;
  coerceTypes: boolean;
}) => {
  validateSchema(schema: unknown): boolean;
  compile(schema: unknown): (value: unknown) => boolean;
};
const validator = new Ajv({
  strict: true,
  allErrors: true,
  useDefaults: false,
  coerceTypes: false,
});
const validate = validator.compile(optionSchema);
const selected = (option: unknown) => {
  const vue = {
    compiler: { sourceMap: true },
    formatter: { singleQuote: false },
    linter: { rules: { [rule]: "off" }, ruleOptions: { [rule]: option } },
  };
  const before = JSON.stringify(vue);
  assert.equal(vueConfigOptions({ vue }), vue, "keep the original canonical config identity");
  assert.equal(JSON.stringify(vue), before, "do not inject defaults or alter foreign values");
  assert.equal(vue.linter.rules[rule], "off", "config validation does not enable the rule");
  return vue;
};

test("public declarations, schema and Pkl retain strings plus only the documented ignore object", () => {
  assert.equal(declarationContract, true);
  assert.equal((require("ajv/package.json") as { version: string }).version, "8.20.0");
  assert.deepEqual(optionSchema, expectedSchema);
  assert.equal(validator.validateSchema(optionSchema), true);
  assert.throws(() => validator.compile({ type: "string", unsupportedIgnoreKeyword: true }), Error);
  for (const name of ["LinterConfig.pkl", "vize.pkl"]) {
    const source = fs.readFileSync(new URL(`npm/cli/pkl/${name}`, root), "utf8");
    const declaration = /^class VOnEventHyphenationOptions \{([\s\S]*?)^\}/mu.exec(source);
    assert(declaration);
    assert.deepEqual(
      declaration[1]!
        .trim()
        .split("\n")
        .map((line) => line.trim()),
      ['style: HyphenationStyle = "always"', "ignore: Listing<String> = new Listing {}"],
    );
    assert.match(
      source,
      /`vue\/v-on-event-hyphenation`: \(HyphenationStyle\|VOnEventHyphenationOptions\)\? = null/u,
    );
  }
});

test("independent strict JSON Schema and canonical selection agree on complete valid boundary inputs", () => {
  for (const option of [
    "always",
    "never",
    {},
    { style: "always" },
    { style: "never", ignore: [] },
    { ignore: ["camelCase", "update:", "雪🌸"] },
    { ignore: ["Case", "case"] },
    { ignore: ["\u0085"] },
    { ignore: [":exit\n", "a\n:exit\n", "not:exit:tail"] },
  ]) {
    assert.equal(validate(option), true, JSON.stringify(option));
    selected(option);
  }
  const empty = {};
  selected(empty);
  assert.deepEqual(
    Reflect.ownKeys(empty),
    [],
    "defaults remain native/Pkl policy, not JS mutation",
  );
});

test("both validators refuse unknown keys, bad types, duplicate strings, blank and exact-exit suffixes", () => {
  const invalid: unknown[] = [
    null,
    true,
    1,
    [],
    "invalid",
    { style: "invalid" },
    { style: null },
    { style: true },
    { ignore: null },
    { ignore: "camelCase" },
    { ignore: [1] },
    { ignore: [null] },
    { ignore: ["same", "same"] },
    { autofix: true },
    { ignoreTags: ["Foo"] },
    { style: "always", extra: false },
    ...["", " ", "\t\r\n", "\ufeff", "a:exit", ":exit", "a\n:exit"].map((value) => ({
      ignore: [value],
    })),
  ];
  for (const option of invalid) {
    assert.equal(validate(option), false, JSON.stringify(option));
    assert.throws(
      () => vueConfigOptions({ vue: { linter: { ruleOptions: { [rule]: option } } } }),
      TypeError,
    );
  }
});

test("canonical overlays replace whole policies while preserving peer rules and other sections", async () => {
  const base = {
    compiler: { compatibility: { vueVersion: "2.7" }, sourceMap: true },
    formatter: { singleQuote: true, semi: true },
    linter: {
      rules: { [rule]: "warn", "vue/v-bind-style": "error" },
      ruleOptions: {
        [rule]: { style: "never", ignore: ["before"] },
        "vue/v-bind-style": "longform",
      },
    },
  };
  const before = structuredClone(base);
  for (const option of [{ ignore: [] }, {}, { style: "always" }]) {
    const own = {
      compiler: { sourceMap: false },
      formatter: { singleQuote: false, semi: undefined },
      linter: { ruleOptions: { [rule]: option, "vue/v-bind-style": undefined } },
    };
    const ownBefore = structuredClone(own);
    const merged = mergeVueConfigOptions<unknown>(base, own);
    const expected = {
      compiler: { compatibility: { vueVersion: "2.7" }, sourceMap: false },
      formatter: { singleQuote: false, semi: true },
      linter: { ...base.linter, ruleOptions: { [rule]: option, "vue/v-bind-style": "longform" } },
    };
    assert.deepEqual(merged, expected);
    assert.equal(vueConfigOptions({ vue: merged }), merged);
    assert.deepEqual(await projectViteConfigSource({ vue: merged }), {
      typeChecker: { jsxTypecheck: true },
      ...expected,
    });
    assert.deepEqual(own, ownBefore, "composition must not mutate the later config");
  }
  for (const own of [
    undefined,
    {},
    { linter: { ruleOptions: {} } },
    { linter: { ruleOptions: { [rule]: undefined } } },
  ]) {
    assert.deepEqual(mergeVueConfigOptions<unknown>(base, own), base);
  }
  assert.deepEqual(base, before, "composition must not mutate inherited policy or peers");
});

test("canonical overlays retain existing strict uniqueness refusals", () => {
  assert.throws(
    () =>
      vueConfigOptions({
        vue: {
          linter: {
            ruleOptions: {
              "script/define-macros-order": { order: ["defineProps", "defineProps"] },
            },
          },
        },
      }),
    TypeError,
  );
});
