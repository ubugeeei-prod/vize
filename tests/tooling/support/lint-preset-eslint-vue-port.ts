import assert from "node:assert/strict";

import { canonical, identity, jsonBytes } from "./lint-preset-source-successors.ts";

const eslintVueRuleMapPath = "tests/_fixtures/patina-eslint-vue-rule-map.json";
const eslintVueMembershipPath =
  "crates/vize_patina/src/preset/snapshots/vize_patina__preset__tests__lint_preset_rule_membership.snap";

const portedEslintVueRules = [
  "vue/camelcase",
  "vue/dot-notation",
  "vue/enforce-style-attribute",
  "vue/eqeqeq",
  "vue/next-tick-style",
  "vue/no-console",
  "vue/no-constant-condition",
  "vue/no-duplicate-class-names",
  "vue/no-empty-pattern",
  "vue/no-implicit-coercion",
  "vue/no-irregular-whitespace",
  "vue/no-literals-in-template",
  "vue/no-loss-of-precision",
  "vue/no-negated-condition",
  "vue/no-parsing-error",
  "vue/no-restricted-syntax",
  "vue/no-restricted-v-bind",
  "vue/no-sparse-arrays",
  "vue/no-undef-components",
  "vue/no-useless-concat",
  "vue/object-shorthand",
  "vue/prefer-separate-static-class",
  "vue/prefer-template",
  "vue/prefer-v-model",
  "vue/quote-props",
  "vue/require-macro-variable-name",
  "vue/sort-keys",
  "vue/v-if-else-key",
  "vue/block-lang",
  "vue/require-default-export",
  "vue/require-prop-comment",
] as const;

const newlyTrackedEslintVueRules = [
  "vue/no-content-outside-block",
  "vue/no-define-model-default",
  "vue/no-props-shadow",
  "vue/no-shadow-native-events",
] as const;

const optInEslintVueRules = [
  "vue/enforce-style-attribute",
  "vue/next-tick-style",
  ...newlyTrackedEslintVueRules,
  "vue/require-macro-variable-name",
  "vue/sort-keys",
  "vue/camelcase",
  "vue/dot-notation",
  "vue/eqeqeq",
  "vue/no-console",
  "vue/no-constant-condition",
  "vue/no-empty-pattern",
  "vue/no-implicit-coercion",
  "vue/no-irregular-whitespace",
  "vue/no-loss-of-precision",
  "vue/no-negated-condition",
  "vue/no-sparse-arrays",
  "vue/no-useless-concat",
  "vue/object-shorthand",
  "vue/prefer-template",
  "vue/quote-props",
  "vue/no-restricted-syntax",
  "vue/no-literals-in-template",
  "vue/no-restricted-v-bind",
  "vue/prefer-v-model",
  "vue/v-if-else-key",
  "vue/block-lang",
  "vue/require-default-export",
  "vue/require-prop-comment",
] as const;

// Remove only the exact eslint-plugin-vue compatibility port before the
// frozen preset-history ledgers replay. Both source identities are pinned so
// a later rule-map or membership edit must receive its own finite successor.
export function inverseEslintVueRulePort(name: string, body: Buffer): Buffer {
  if (name === eslintVueRuleMapPath) {
    identity(body, {
      bytes: 54125,
      sha256: "e468375d0bf07d9f465c0ef439d16dea64ec228206e36110558af4679e07adc8",
    });
    const value = canonical(body);
    assert.deepEqual(value.upstream, {
      package: "eslint-plugin-vue",
      ruleCount: 256,
      version: "10.12.0",
    });
    assert.deepEqual(value.summary, {
      intentionalDivergence: 3,
      mapped: 184,
      unimplemented: 69,
    });
    for (const rule of [...portedEslintVueRules, ...newlyTrackedEslintVueRules]) {
      assert.equal(value.entries[rule].status, "mapped", rule);
      assert.equal(value.entries[rule].patinaRule.length > 0, true, rule);
    }
    for (const rule of portedEslintVueRules)
      value.entries[rule] = { issue: 3223, status: "unimplemented" };
    for (const rule of newlyTrackedEslintVueRules) delete value.entries[rule];
    assert.deepEqual(value.entries["vue/no-shared-component-data"], {
      issue: 3223,
      status: "unimplemented",
    });
    value.entries["vue/no-shared-component-data"] = {
      patinaPresets: ["ecosystem", "essential", "general-recommended", "nuxt", "opinionated"],
      patinaRule: "script/no-deprecated-data-object-declaration",
      patinaSeverity: "error",
      status: "mapped",
    };
    value.summary = { intentionalDivergence: 3, mapped: 150, unimplemented: 99 };
    value.upstream = { package: "eslint-plugin-vue", ruleCount: 252, version: "10.9.2" };
    const predecessor = jsonBytes(value);
    identity(predecessor, {
      bytes: 51010,
      sha256: "d35cd69eae6004dd2fd57b52bab1fe928b6df5c7f5c6a90b4fcff087395ccb17",
    });
    return predecessor;
  }
  if (name === eslintVueMembershipPath) {
    identity(body, {
      bytes: 36309,
      sha256: "abe7325c43538cc239a226a539dcf2a1c8619ba79758799de2c97f92e96bfe03",
    });
    const text = body.toString();
    const at = text.indexOf("{");
    const table = canonical(Buffer.from(text.slice(at)));
    for (const rule of optInEslintVueRules) {
      assert.equal(table.opt_in.filter((name: string) => name === rule).length, 1, rule);
      table.opt_in.splice(table.opt_in.indexOf(rule), 1);
    }
    const predecessor = Buffer.concat([Buffer.from(text.slice(0, at)), jsonBytes(table)]);
    identity(predecessor, {
      bytes: 35404,
      sha256: "596d5f6e263d7a9e57164b699d5f12419a7956b706e45446dc2002f195018668",
    });
    return predecessor;
  }
  return body;
}
