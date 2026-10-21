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

test("the current eslint-plugin-vue configurable surface stays explicit", () => {
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
  assert.equal(audit.configurableRules.length, audit.upstream.configurableRuleCount);
  assert.deepEqual(
    audit.configurableRules,
    [...new Set(audit.configurableRules)].sort((left, right) => left.localeCompare(right)),
  );

  const statuses = { mapped: 0, unimplemented: 0, intentionalDivergence: 0 };
  for (const rule of audit.configurableRules) {
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

test("the current release delta does not silently move the frozen parity provider", () => {
  assert.equal(ruleMap.upstream.version, audit.pinnedCompatibilityProvider.version);
  assert.equal(ruleMap.upstream.ruleCount, audit.pinnedCompatibilityProvider.ruleCount);
  assert.equal(
    ruleMap.upstream.ruleCount + audit.pinnedCompatibilityProvider.currentAdditions.length,
    audit.upstream.ruleCount,
  );
  assert.deepEqual(audit.pinnedCompatibilityProvider.currentAdditions, [
    {
      rule: "vue/no-shadow-native-events",
      configurable: false,
      status: "unimplemented",
    },
  ]);
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
