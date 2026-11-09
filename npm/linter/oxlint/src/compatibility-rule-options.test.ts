import assert from "node:assert/strict";
import { it } from "vite-plus/test";

import { getRuleOptions } from "./rule-options.ts";

it("projects eslint-plugin-vue compatibility options into typed Patina options", () => {
  assert.deepEqual(getRuleOptions("vue/enforce-style-attribute", [{ allow: ["plain", "scoped"] }]), {
    ruleOptions: {
      "vue/enforce-style-attribute": { allow: ["plain", "scoped"] },
    },
  });
  assert.deepEqual(getRuleOptions("vue/no-restricted-syntax", ["CallExpression"]), {
    ruleOptions: {
      "vue/no-restricted-syntax": { selectors: ["CallExpression"] },
    },
  });
  assert.deepEqual(getRuleOptions("vue/quote-props", ["as-needed"]), {
    ruleOptions: { "vue/quote-props": "as-needed" },
  });
  assert.deepEqual(
    getRuleOptions("vue/sort-keys", ["desc", { caseSensitive: false, natural: true }]),
    {
      ruleOptions: {
        "vue/sort-keys": { order: "desc", caseSensitive: false, natural: true },
      },
    },
  );
});

it("rejects compatibility options that Patina cannot preserve", () => {
  assert.equal(
    getRuleOptions("vue/no-restricted-syntax", ["CallExpression[callee.name='fetch']"]),
    undefined,
  );
  assert.equal(
    getRuleOptions("vue/sort-keys", ["asc", { ignoreChildrenOf: ["methods"] }]),
    undefined,
  );
  assert.equal(getRuleOptions("vue/quote-props", ["always"]), undefined);
});
