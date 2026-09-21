// Precision tiers (P4-6c): every rule's tier comes from its RuleContract row
// in crates/vize_patina/src/rule_contracts/table.rs. A rule without a row, a
// row without a rule, or a row this parser cannot read is a hard failure, so
// the matrix's `tier` column cannot drift from the contract table.

import { readFileSync } from "node:fs";
import path from "node:path";

import { PATINA_SRC } from "./rule-parity-paths.mjs";

export const TIER_TABLE_REL = "crates/vize_patina/src/rule_contracts/table.rs";
export const TIERS = ["exact", "sound", "complete", "heuristic"];

const ROW =
  /^\s*row!\("(?<name>[^"]+)",\s*(?<tier>Exact|Sound|Complete|Heuristic),\s*[A-Z0-9_]+,\s*(?:Error|Warning)\),\s*$/u;

/** name -> tier (lowercase), read from the contract table. */
export function readTierTable() {
  const source = readFileSync(path.join(PATINA_SRC, "rule_contracts", "table.rs"), "utf8");
  const tiers = new Map();
  for (const [index, line] of source.split("\n").entries()) {
    if (!line.trimStart().startsWith("row!(")) continue;
    const match = ROW.exec(line);
    if (!match) throw new Error(`${TIER_TABLE_REL}:${index + 1}: unreadable contract row`);
    const { name, tier } = match.groups;
    if (tiers.has(name)) throw new Error(`${TIER_TABLE_REL}: duplicate contract row "${name}"`);
    tiers.set(name, tier.toLowerCase());
  }
  return tiers;
}

/** Attach `tier` to every rule; fail on a rule without a row or a stray row. */
export function applyTiers(rules, tiers = readTierTable()) {
  for (const rule of rules.values()) {
    const tier = tiers.get(rule.name);
    if (tier === undefined) {
      throw new Error(`rule "${rule.name}" has no tier: add its row to ${TIER_TABLE_REL}`);
    }
    rule.tier = tier;
  }
  for (const name of tiers.keys()) {
    if (!rules.has(name)) throw new Error(`${TIER_TABLE_REL}: row for unknown rule "${name}"`);
  }
}
