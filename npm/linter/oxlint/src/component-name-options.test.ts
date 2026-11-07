import assert from "node:assert/strict";
import { it } from "vite-plus/test";
import { getRuleOptions } from "./rule-options.ts";
import { createRuleSelection, parseRuleSelection } from "./rule-selection.ts";
import { componentNameCasingSchema } from "./component-name-options.ts";

const rule = "vue/component-name-in-template-casing";

it("preserves the old style projection and carries both tuple fields into the native policy", () => {
  for (const casing of ["PascalCase", "kebab-case"]) {
    assert.deepEqual(getRuleOptions(rule, [casing]), { componentNameInTemplateCasing: casing });
    for (const policy of [
      {},
      { registeredComponentsOnly: false },
      { globals: ["MyWidget"] },
      { registeredComponentsOnly: true, globals: ["my-widget"] },
    ]) {
      const canonical = { casing, ...policy };
      for (const options of [[casing, policy], [canonical]]) {
        const before = structuredClone(options);
        const projected = getRuleOptions(rule, options);
        assert.deepEqual(projected, { ruleOptions: { [rule]: canonical } });
        assert.deepEqual(
          parseRuleSelection({ ["vize/" + rule]: ["warn", ...options] })?.options,
          projected,
        );
        assert.deepEqual(options, before);
        assert.notEqual(projected?.ruleOptions?.[rule], canonical);
        if (policy.globals)
          assert.notEqual(projected?.ruleOptions?.[rule]?.globals, policy.globals);
      }
    }
  }
});

it("retains default/reset and disabled or unselected rule behavior", () => {
  assert.equal(getRuleOptions(rule, []), undefined);
  assert.deepEqual(getRuleOptions(rule, [{}]), { ruleOptions: { [rule]: {} } });
  assert.deepEqual(getRuleOptions(rule, [{ globals: ["/Widget/i"] }]), {
    ruleOptions: { [rule]: { globals: ["/Widget/i"] } },
  });
  assert.deepEqual(createRuleSelection(new Map()).names, []);
  assert.deepEqual(
    parseRuleSelection({
      ["vize/" + rule]: ["off", "PascalCase", { registeredComponentsOnly: false }],
    })?.names,
    [],
  );
  assert.deepEqual(
    parseRuleSelection({ ["vize/" + rule]: [0, { globals: ["MyWidget"] }] })?.options,
    {},
  );
});

it("refuses unknown policies, invalid field types, ambiguous objects and extra tuple entries", () => {
  for (const options of [
    ["camelCase"],
    [null],
    [false],
    [[]],
    [{ casing: "camelCase" }],
    [{ registeredComponentsOnly: null }],
    [{ globals: [42] }],
    [{ ignores: [] }],
    ["PascalCase", { ignores: [] }],
    ["PascalCase", { unknown: true }],
    ["PascalCase", { casing: "kebab-case" }],
    ["PascalCase", { globals: null }],
    ["PascalCase", { registeredComponentsOnly: "false" }],
    ["PascalCase", { globals: ["/^My/i"] }],
    ["PascalCase", null],
    ["PascalCase", undefined],
    ["PascalCase", {}, {}],
    [{}, {}],
    [{ casing: "PascalCase" }, { globals: [] }],
  ])
    assert.equal(getRuleOptions(rule, options), undefined);
  const casing = { enum: ["PascalCase", "kebab-case"] };
  const properties = {
    registeredComponentsOnly: { type: "boolean" },
    globals: { type: "array", items: { type: "string" } },
  };
  assert.deepEqual(componentNameCasingSchema(), {
    type: "array",
    oneOf: [
      {
        items: [
          casing,
          {
            type: "object",
            properties: {
              ...properties,
              globals: {
                type: "array",
                items: { type: "string", not: { pattern: "^/(.+)/(.*)$" } },
              },
            },
            additionalProperties: false,
          },
        ],
        maxItems: 2,
      },
      {
        items: [
          { type: "object", properties: { casing, ...properties }, additionalProperties: false },
        ],
        minItems: 1,
        maxItems: 1,
      },
    ],
  });
});
