import assert from "node:assert/strict";
import { test } from "node:test";

import {
  TIER_TABLE_REL,
  applyTiers,
  readTierTable,
} from "../../tools/support/compat/davinci/lib/rule-parity-tiers.mjs";

// P4-6c: the rule-parity matrix's `tier` column comes from the RuleContract
// table, and generation fails on a rule without a row or a row without a
// rule — proven here by injection, beside the real table's census.

test("the contract table declares a tier for 249 rules", () => {
  const tiers: Map<string, string> = readTierTable();
  const census = new Map<string, number>();
  for (const tier of tiers.values()) census.set(tier, (census.get(tier) ?? 0) + 1);
  assert.equal(tiers.size, 249);
  assert.deepEqual(
    [...census].sort(([a], [b]) => (a < b ? -1 : 1)),
    [
      ["complete", 32],
      ["exact", 200],
      ["heuristic", 17],
    ],
  );
});

test("a rule without a row fails generation", () => {
  const rules = new Map([
    ["vue/a", { name: "vue/a" }],
    ["vue/b", { name: "vue/b" }],
  ]);
  assert.throws(() => applyTiers(rules, new Map([["vue/a", "exact"]])), {
    message: `rule "vue/b" has no tier: add its row to ${TIER_TABLE_REL}`,
  });
});

test("a row without a rule fails generation", () => {
  const rules = new Map([["vue/a", { name: "vue/a" }]]);
  const tiers = new Map([
    ["vue/a", "exact"],
    ["vue/gone", "heuristic"],
  ]);
  assert.throws(() => applyTiers(rules, tiers), {
    message: `${TIER_TABLE_REL}: row for unknown rule "vue/gone"`,
  });
});

test("every rule receives exactly its row's tier", () => {
  const rules = new Map([
    ["vue/a", { name: "vue/a" } as { name: string; tier?: string }],
    ["vue/b", { name: "vue/b" } as { name: string; tier?: string }],
  ]);
  applyTiers(
    rules,
    new Map([
      ["vue/a", "exact"],
      ["vue/b", "complete"],
    ]),
  );
  assert.deepEqual(
    [...rules.values()].map((rule) => [rule.name, rule.tier]),
    [
      ["vue/a", "exact"],
      ["vue/b", "complete"],
    ],
  );
});
