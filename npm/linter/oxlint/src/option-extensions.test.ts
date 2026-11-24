// The Oxlint and ESLint bridges forward the same option shapes `vize lint`
// reads from `linter.ruleOptions` (#13535, #13536, #13534).
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { it } from "vite-plus/test";
import { getRuleOptions } from "./rule-options.ts";
import { parseRuleSelection } from "./rule-selection.ts";
import { restrictedHtmlElementsSchema, restrictedVOnSchema } from "./restricted-entry-options.ts";

const require = createRequire(import.meta.url);
const Ajv: typeof import("ajv").default = require("ajv");

it("carries the v-bind-style tuple and keeps the lone style a string", () => {
  const rule = "vue/v-bind-style";
  for (const style of ["shorthand", "longform"]) {
    assert.deepEqual(getRuleOptions(rule, [style]), { ruleOptions: { [rule]: style } });
    for (const options of [
      {},
      { sameNameShorthand: "always" },
      { sameNameShorthand: "never" },
      { sameNameShorthand: "ignore" },
    ]) {
      const tuple = [style, options];
      const before = structuredClone(tuple);
      const projected = getRuleOptions(rule, tuple);
      assert.deepEqual(projected, { ruleOptions: { [rule]: [style, options] } });
      assert.deepEqual(
        parseRuleSelection({ ["vize/" + rule]: ["warn", ...tuple] })?.options,
        projected,
      );
      assert.deepEqual(tuple, before);
      const forwarded = projected?.ruleOptions?.[rule];
      assert.notEqual(Array.isArray(forwarded) ? forwarded[1] : undefined, options);
    }
  }
  assert.equal(getRuleOptions(rule, []), undefined);
  for (const options of [
    ["sometimes"],
    [{ sameNameShorthand: "always" }],
    ["shorthand", "longform"],
    ["shorthand", null],
    ["shorthand", ["always"]],
    ["shorthand", { sameNameShorthand: "sometimes" }],
    ["shorthand", { sameName: "always" }],
    ["shorthand", { sameNameShorthand: "always" }, {}],
  ])
    assert.equal(getRuleOptions(rule, options), undefined, JSON.stringify(options));
});

it("carries restricted element entries with their messages", () => {
  const rule = "vue/no-restricted-html-elements";
  const validate = new Ajv().compile(restrictedHtmlElementsSchema());
  for (const options of [
    ["a"],
    [{ element: ["a"], message: "Use Bar." }],
    [{ element: ["a"] }],
    [{ element: "a", message: "Use Bar." }],
    ["marquee", { element: ["a", "button"], message: "Use the design system." }, "blink"],
  ]) {
    const before = structuredClone(options);
    const projected = getRuleOptions(rule, options);
    assert.deepEqual(projected, { ruleOptions: { [rule]: options } }, JSON.stringify(options));
    assert.notEqual(projected?.ruleOptions?.[rule], options);
    assert.deepEqual(options, before);
    assert(validate(options), JSON.stringify(options));
  }
  assert.equal(getRuleOptions(rule, []), undefined);
  assert(validate([]));
  for (const options of [
    [{ message: "Use Bar." }],
    [{ element: 1 }],
    [{ element: ["a", 1] }],
    [{ element: "a", message: 1 }],
    [{ element: "a", msg: "Use Bar." }],
    [["a"]],
    [null],
    ["a", { message: "Use Bar." }],
  ]) {
    assert.equal(getRuleOptions(rule, options), undefined, JSON.stringify(options));
    assert(!validate(options), JSON.stringify(options));
  }
});

it("carries restricted listener entries with modifiers, element and message", () => {
  const rule = "vue/no-restricted-v-on";
  const validate = new Ajv().compile(restrictedVOnSchema());
  for (const options of [
    ["click"],
    [{ argument: "click", modifiers: ["stop"] }],
    [{ argument: "click", modifiers: ["stop", "prevent"], message: "Handle it in the component." }],
    ["legacy-click", { argument: "focus", element: "MyInput" }],
  ]) {
    const before = structuredClone(options);
    const projected = getRuleOptions(rule, options);
    assert.deepEqual(projected, { ruleOptions: { [rule]: options } }, JSON.stringify(options));
    assert.notEqual(projected?.ruleOptions?.[rule], options);
    assert.deepEqual(options, before);
    assert(validate(options), JSON.stringify(options));
  }
  assert.equal(getRuleOptions(rule, []), undefined);
  for (const options of [
    [null],
    [{ argument: null }],
    [{ modifiers: ["stop"] }],
    [{ argument: "click", modifiers: "stop" }],
    [{ argument: "click", modifiers: [1] }],
    [{ argument: "click", element: 1 }],
    [{ argument: "click", modifier: ["stop"] }],
    [["click"]],
  ]) {
    assert.equal(getRuleOptions(rule, options), undefined, JSON.stringify(options));
    assert(!validate(options), JSON.stringify(options));
  }
});
