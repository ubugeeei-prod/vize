import assert from "node:assert/strict";
import { it } from "vite-plus/test";
import {
  blockLangSchema,
  getBlockLangOptions,
  getMaxLinesOptions,
  getMaxLinesPerBlockOptions,
  maxLinesPerBlockSchema,
  maxLinesSchema,
} from "./block-limit-options.ts";
import { getRuleOptions } from "./rule-options.ts";
import { parseRuleSelection } from "./rule-selection.ts";

it("forwards vue/block-lang languages and refuses a shape the native side rejects", () => {
  const option = { script: { lang: "ts" } };
  const before = structuredClone(option);
  const projected = getRuleOptions("vue/block-lang", [option]);
  assert.deepEqual(projected, { ruleOptions: { "vue/block-lang": { script: { lang: "ts" } } } });
  assert.deepEqual(option, before);
  assert.notEqual(projected?.ruleOptions?.["vue/block-lang"], option);
  assert.deepEqual(getBlockLangOptions([{ script: { lang: ["ts", "tsx"], allowNoLang: false } }]), {
    ruleOptions: {
      "vue/block-lang": { script: { lang: ["ts", "tsx"], allowNoLang: false } },
    },
  });
  for (const options of [
    [],
    [{ script: { lang: "" } }],
    [{ script: { lang: [] } }],
    [{ script: { extra: true } }],
    [{ i18n: { lang: "json" } }],
    [{ script: { lang: "ts" } }, {}],
  ])
    assert.equal(getRuleOptions("vue/block-lang", options), undefined);
  assert.deepEqual(blockLangSchema().additionalProperties, false);
  assert.deepEqual(
    parseRuleSelection({ "vize/vue/block-lang": ["warn", { script: { lang: "ts" } }] })?.options,
    projected,
  );
});

it("forwards max-lines as a number or the skip object", () => {
  assert.deepEqual(getRuleOptions("max-lines", [400]), { ruleOptions: { "max-lines": 400 } });
  const object = { max: 400, skipComments: true, skipBlankLines: false };
  assert.deepEqual(getMaxLinesOptions([object]), {
    ruleOptions: { "max-lines": { max: 400, skipComments: true, skipBlankLines: false } },
  });
  assert.notEqual(getMaxLinesOptions([object])?.ruleOptions?.["max-lines"], object);
  for (const options of [
    [1.5],
    [-1],
    [{ max: 1, extra: true }],
    [{ skipComments: "yes" }],
    [],
  ] as const)
    assert.equal(getRuleOptions("max-lines", options), undefined);
  assert.equal(maxLinesSchema().oneOf instanceof Array, true);
  assert.deepEqual(parseRuleSelection({ "vize/max-lines": ["warn", 400] })?.options, {
    ruleOptions: { "max-lines": 400 },
  });
});

it("forwards vue/max-lines-per-block limits", () => {
  const option = { template: 80, script: 120, skipBlankLines: true };
  const projected = getMaxLinesPerBlockOptions([option]);
  assert.deepEqual(projected, {
    ruleOptions: { "vue/max-lines-per-block": { template: 80, script: 120, skipBlankLines: true } },
  });
  assert.notEqual(projected?.ruleOptions?.["vue/max-lines-per-block"], option);
  assert.equal(getRuleOptions("vue/max-lines-per-block", [{ style: -1 }]), undefined);
  assert.equal(getRuleOptions("vue/max-lines-per-block", [{ skipComments: true }]), undefined);
  assert.equal(maxLinesPerBlockSchema().additionalProperties, false);
  assert.deepEqual(
    parseRuleSelection({ "vize/vue/max-lines-per-block": ["warn", option] })?.options,
    projected,
  );
});
