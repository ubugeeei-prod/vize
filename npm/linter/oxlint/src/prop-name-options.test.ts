import assert from "node:assert/strict";
import { it } from "vite-plus/test";
import { getRuleOptions } from "./rule-options.ts";
import { createRuleSelection, parseRuleSelection } from "./rule-selection.ts";
import { propNameCasingSchema } from "./prop-name-options.ts";

const rule = "vue/prop-name-casing";

it("projects each public prop style into the actual complete native JSON policy", () => {
  for (const casing of ["camelCase", "snake_case"]) {
    for (const option of [casing, { casing }]) {
      const before = structuredClone(option);
      const projected = getRuleOptions(rule, [option]);
      assert.deepEqual(projected, { ruleOptions: { [rule]: { casing } } });
      assert.deepEqual(option, before);
      if (typeof option === "object") assert.notEqual(projected?.ruleOptions?.[rule], option);
      assert.deepEqual(
        parseRuleSelection({ ["vize/" + rule]: ["warn", option] })?.options,
        projected,
      );
    }
  }
});

it("retains the native default/reset and leaves disabled or unselected rules unselected", () => {
  const reset = {};
  const projected = getRuleOptions(rule, [reset]);
  assert.deepEqual(projected, { ruleOptions: { [rule]: {} } });
  assert.notEqual(projected?.ruleOptions?.[rule], reset);
  assert.deepEqual(reset, {});
  assert.equal(getRuleOptions(rule, []), undefined);
  assert.deepEqual(createRuleSelection(new Map()).names, []);
  assert.deepEqual(parseRuleSelection({ ["vize/" + rule]: ["off", "snake_case"] })?.names, []);
  assert.deepEqual(
    parseRuleSelection({ ["vize/" + rule]: [0, { casing: "snake_case" }] })?.options,
    {},
  );
});

it("refuses unsupported tuple entries and preserves the exact strict public schema", () => {
  for (const options of [
    ["snake-case"],
    [false],
    [null],
    [[]],
    [{ casing: null }],
    [{ casing: false }],
    [{ casing: "snake-case" }],
    [{ ignoreProps: [] }],
    [{ casing: "snake_case", unknown: true }],
    ["snake_case", { ignoreProps: [] }],
    [{}, {}],
  ])
    assert.equal(getRuleOptions(rule, options), undefined);
  assert.deepEqual(propNameCasingSchema(), {
    oneOf: [
      { enum: ["camelCase", "snake_case"] },
      {
        type: "object",
        properties: { casing: { enum: ["camelCase", "snake_case"] } },
        additionalProperties: false,
      },
    ],
  });
});
