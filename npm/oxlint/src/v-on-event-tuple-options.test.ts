import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { it } from "vite-plus/test";
import { getRuleOptions } from "./rule-options.ts";
import { parseRuleSelection } from "./rule-selection.ts";
import { vOnEventHyphenationRuleSchema, vOnEventHyphenationSchema } from "./v-on-event-options.ts";
import {
  cases,
  policy,
  rule,
} from "../../../tests/tooling/support/patina-v-on-autofix-contract.ts";

const require = createRequire(import.meta.url);
const Ajv: typeof import("ajv").default = require("ajv");
const validate = new Ajv().compile(vOnEventHyphenationRuleSchema());
const canonical = new Ajv().compile(vOnEventHyphenationSchema());

it("projects all 22 frozen official option vectors without mutating their policies", () => {
  const observations = cases.map((row) => {
    const before = structuredClone(row.options);
    const projected = getRuleOptions(rule, row.options);
    const selection = parseRuleSelection({ ["vize/" + rule]: ["warn", ...row.options] });
    return { id: row.id, before, options: row.options, projected, selection };
  });
  console.log("vize-v-on-tuple-projections " + JSON.stringify(observations));
  for (const [index, actual] of observations.entries()) {
    const row = cases[index]!;
    const option = policy(row);
    const expected =
      option === undefined
        ? undefined
        : typeof option === "string"
          ? { vOnEventHyphenation: option }
          : { ruleOptions: { [rule]: option } };
    if (row.options.length === 2) {
      const projected = actual.projected?.ruleOptions?.[rule];
      assert.notEqual(projected, row.options[1], row.id);
      if (typeof projected === "object" && projected !== null && "ignore" in projected)
        assert.notEqual(projected.ignore, (row.options[1] as { ignore: unknown }).ignore);
      assert(canonical(option), row.id);
      assert.deepEqual(getRuleOptions(rule, [option]), expected, row.id);
    }
    assert(validate(actual.options), row.id);
    assert.deepEqual(actual.projected, expected, row.id);
    assert.deepEqual(actual.selection?.options, expected ?? {}, row.id);
    assert.deepEqual(actual.options, actual.before, row.id);
  }
});

it("keeps default, canonical empty reset, off and unrelated selections unchanged", () => {
  assert(validate([]));
  assert.equal(getRuleOptions(rule, []), undefined);
  assert.deepEqual(getRuleOptions(rule, [{}]), { ruleOptions: { [rule]: {} } });
  assert.deepEqual(getRuleOptions(rule, ["always", {}]), {
    ruleOptions: { [rule]: { style: "always" } },
  });
  assert.deepEqual(parseRuleSelection({ ["vize/" + rule]: "warn" })?.names, [rule]);
  assert.deepEqual(
    parseRuleSelection({ ["vize/" + rule]: ["off", "always", { autofix: true }] })?.names,
    [],
  );
  assert.deepEqual(parseRuleSelection({ "vize/vue/no-v-html": "warn" })?.options, {});
  const source = { style: "never" as const, autofix: true, ignore: ["update:"] };
  const projected = getRuleOptions(rule, [source]);
  assert.deepEqual(projected, { ruleOptions: { [rule]: source } });
  assert.notEqual(projected?.ruleOptions?.[rule], source);
});

it("refuses ambiguous tuples, unsupported ignoreTags and malformed policies at both boundaries", () => {
  const invalid = [
    ["sometimes"],
    [null],
    [false],
    [[]],
    ["always", null],
    ["always", undefined],
    ["always", { autofix: null }],
    ["always", { autofix: "true" }],
    ["always", { autofix: 1 }],
    ["always", { ignore: null }],
    ["always", { ignore: [0] }],
    ["always", { ignore: [""] }],
    ["always", { ignore: ["row:exit"] }],
    ["always", { ignore: ["row", "row"] }],
    ["always", { ignoreTags: [] }],
    ["always", { unknown: true }],
    ["always", { style: "never" }],
    ["always", { style: undefined }],
    [{ style: "always" }, { autofix: true }],
    [{}, {}],
    ["always", {}, {}],
  ];
  const observations = invalid.map((options) => ({
    options,
    projected: getRuleOptions(rule, options),
    valid: validate(options),
    errors: structuredClone(validate.errors),
  }));
  console.log("vize-v-on-tuple-refusals " + JSON.stringify(observations));
  for (const row of observations) {
    assert.equal(row.projected, undefined);
    assert.equal(row.valid, false);
  }
});
