import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";

import { repoRoot } from "./_helpers/tool-commands.ts";

const audit = JSON.parse(
  fs.readFileSync(
    path.join(repoRoot, "tests/_fixtures/patina-eslint-vue-configurable-audit.json"),
    "utf8",
  ),
);
const ruleMap = JSON.parse(
  fs.readFileSync(path.join(repoRoot, "tests/_fixtures/patina-eslint-vue-rule-map.json"), "utf8"),
);
const schema = JSON.parse(
  fs.readFileSync(path.join(repoRoot, "npm/cli/schemas/vize.config.schema.json"), "utf8"),
);

function isStringArray(value: unknown): value is string[] {
  return Array.isArray(value) && value.every((entry: unknown) => typeof entry === "string");
}

test("the current eslint-plugin-vue configurable surface stays explicit", () => {
  const configurableRules: unknown = audit.configurableRules;

  assert.equal(audit.schemaVersion, 1);
  assert.deepEqual(audit.upstream, {
    package: "eslint-plugin-vue",
    version: "10.11.1",
    tagCommit: "ec9c53a6fbed637d7b0fbcd6893b0206ae32839f",
    publishedAt: "2026-09-24T01:08:57Z",
    source: "https://github.com/vuejs/eslint-plugin-vue/releases/tag/v10.11.1",
    ruleCount: 253,
    configurableRuleCount: 112,
  });
  assert.ok(isStringArray(configurableRules));
  assert.equal(configurableRules.length, audit.upstream.configurableRuleCount);
  assert.deepEqual(
    configurableRules,
    [...new Set(configurableRules)].sort((left, right) => left.localeCompare(right)),
  );

  const statuses = { mapped: 0, unimplemented: 0, intentionalDivergence: 0 };
  for (const rule of configurableRules) {
    const entry = ruleMap.entries[rule];
    assert.ok(entry, `${rule} must remain in the pinned compatibility map`);
    if (entry.status === "mapped") statuses.mapped += 1;
    else if (entry.status === "unimplemented") statuses.unimplemented += 1;
    else if (entry.status === "intentional-divergence") statuses.intentionalDivergence += 1;
    else assert.fail(`${rule} has unsupported status ${entry.status}`);
  }
  assert.deepEqual(statuses, {
    mapped: audit.summary.mapped,
    unimplemented: audit.summary.unimplemented,
    intentionalDivergence: audit.summary.intentionalDivergence,
  });
});

test("the scorecard can advance without silently moving the frozen parity provider", () => {
  assert.deepEqual(audit.pinnedCompatibilityProvider, {
    version: "10.9.2",
    ruleCount: 252,
    currentAdditions: [
      {
        rule: "vue/no-shadow-native-events",
        configurable: false,
        status: "unimplemented",
      },
    ],
  });
  assert.deepEqual(ruleMap.upstream, {
    package: "eslint-plugin-vue",
    version: "10.12.0",
    ruleCount: 256,
  });
  assert.equal(
    audit.pinnedCompatibilityProvider.ruleCount +
      audit.pinnedCompatibilityProvider.currentAdditions.length,
    audit.upstream.ruleCount,
  );
});

test("new configurable mappings retain their exact presets without claiming typed options", () => {
  const additions = {
    "vue/comment-directive": {
      patinaPresets: ["ecosystem", "essential", "general-recommended", "nuxt", "opinionated"],
      patinaRule: "vue/comment-directive",
      patinaSeverity: "error",
      status: "mapped",
    },
    "vue/match-component-file-name": {
      patinaPresets: [],
      patinaRule: "vue/match-component-file-name",
      patinaSeverity: "warning",
      status: "mapped",
    },
  };
  for (const [upstreamRule, entry] of Object.entries(additions)) {
    assert.deepEqual(ruleMap.entries[upstreamRule], entry, upstreamRule);
    assert.ok(audit.configurableRules.includes(upstreamRule), upstreamRule);
    assert.equal(
      audit.mappedWithTypedOptions.some(([rule]: [string, string]) => rule === upstreamRule),
      false,
      `${upstreamRule} has no typed-option parity claim`,
    );
    assert.equal(
      Object.hasOwn(schema.definitions.LintRuleOptions.properties, entry.patinaRule),
      false,
      `${entry.patinaRule} must not acquire an unaudited typed-option claim`,
    );
  }
});

test("every claimed mapped option is present in the public typed config schema", () => {
  const properties = schema.definitions.LintRuleOptions.properties;
  assert.equal(audit.mappedWithTypedOptions.length, audit.summary.mappedWithTypedOptions);
  assert.equal(
    audit.summary.mapped - audit.summary.mappedWithTypedOptions,
    audit.summary.mappedWithoutTypedOptions,
  );

  for (const [upstreamRule, patinaRule] of audit.mappedWithTypedOptions) {
    assert.equal(ruleMap.entries[upstreamRule]?.status, "mapped", upstreamRule);
    assert.equal(ruleMap.entries[upstreamRule]?.patinaRule, patinaRule, upstreamRule);
    assert.ok(properties[patinaRule], `${patinaRule} must expose typed options`);
  }
});

test("real-project allowance arrays accept the same duplicate values as native config", () => {
  const properties = schema.definitions.LintRuleOptions.properties;
  for (const [rule, field] of [
    ["vue/warn-custom-directive", "ignore"],
    ["vue/no-bare-strings-in-template", "allowlist"],
    ["a11y/anchor-has-content", "accessibleChildren"],
    ["a11y/label-has-for", "controlComponents"],
  ]) {
    assert.equal(properties[rule].properties[field].uniqueItems, undefined, `${rule}.${field}`);
  }
});
