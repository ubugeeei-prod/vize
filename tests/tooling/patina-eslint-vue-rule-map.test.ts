import assert from "node:assert/strict";
import { test } from "node:test";
import {
  readRuleMap,
  validateRuleMap,
} from "../../tools/support/compat/fixtures/patina-rule-map.ts";

test("the Patina scorecard exhaustively maps the pinned eslint-plugin-vue rule surface", () => {
  const ruleMap = validateRuleMap(readRuleMap());

  assert.equal(ruleMap.upstream.version, "10.12.0");
  assert.equal(ruleMap.upstream.ruleCount, 256);
  assert.deepEqual(ruleMap.summary, { mapped: 201, unimplemented: 54, intentionalDivergence: 1 });
  // The rule checks definition names as upstream does; the file-name check is
  // vue/component-file-name-casing, which has no upstream counterpart.
  assert.deepEqual(ruleMap.entries["vue/component-definition-name-casing"], {
    status: "mapped",
    patinaRule: "vue/component-definition-name-casing",
    patinaSeverity: "warning",
    patinaPresets: ["ecosystem", "general-recommended", "nuxt", "opinionated"],
  });
  assert.deepEqual(ruleMap.entries["vue/no-unused-properties"], {
    status: "intentional-divergence",
    reason:
      "Patina intentionally checks defineProps declarations only; eslint-plugin-vue also checks Options API props, so the surfaces are not comparable.",
  });
  // Patina's default allows only method references; eslint-plugin-vue's styles
  // are reachable through linter.ruleOptions.
  assert.deepEqual(ruleMap.entries["vue/v-on-handler-style"], {
    status: "mapped",
    patinaRule: "vue/v-on-handler-style",
    patinaSeverity: "warning",
    patinaPresets: ["nuxt", "opinionated"],
    matrixRuleOptions: {
      options: ["method", "inline-function"],
      reason:
        'Patina\'s default policy reports every inline function and statement; eslint-plugin-vue\'s default ["method", "inline-function"] accepts one that a method reference cannot express, such as a handler that reads a v-for or slot variable.',
      documentation: {
        path: "crates/vize_patina/src/rules/opinionated/vue/v_on_handler_style.rs",
        line: 6,
        text: "and inline statements are reported by the default method-handler policy.",
        upstreamOptions: '["method", "inline-function"]',
      },
      defaultGuard: "reports_handler_that_closes_over_v_for_variables",
    },
  });
});
