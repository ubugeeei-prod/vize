import assert from "node:assert/strict";
import { it } from "vite-plus/test";
import { getRuleOptions } from "./rule-options.ts";
import { isVOnEventHyphenationOptions } from "./v-on-event-options.ts";

const rule = "vue/v-on-event-hyphenation";

it("projects each explicit event autofix boolean through the actual canonical option carrier", () => {
  for (const autofix of [false, true]) {
    for (const style of ["always", "never"]) {
      const option = { style, ignore: ["update:"], autofix };
      const before = structuredClone(option);
      const result = getRuleOptions(rule, [option]);
      assert.deepEqual(result, { ruleOptions: { [rule]: option } });
      assert.notEqual(result?.ruleOptions?.[rule], option);
      assert.deepEqual(option, before);
    }
  }
  assert.deepEqual(getRuleOptions(rule, ["never"]), { vOnEventHyphenation: "never" });
});

it("refuses malformed event autofix and unknown keys without admitting ignoreTags", () => {
  for (const option of [
    { autofix: null },
    { autofix: "true" },
    { autofix: 1 },
    { autofix: [] },
    { autofix: {} },
    { autofixTypo: true },
    { autofix: true, ignoreTags: [] },
  ]) {
    assert.equal(isVOnEventHyphenationOptions(option), false);
    assert.equal(getRuleOptions(rule, [option]), undefined);
  }
});
