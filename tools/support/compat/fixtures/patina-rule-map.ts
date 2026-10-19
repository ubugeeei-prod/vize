import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";
import { fileURLToPath, pathToFileURL } from "node:url";

export type RuleMapEntry =
  | {
      readonly status: "mapped";
      readonly patinaRule: string;
      readonly patinaSeverity: string;
      readonly patinaPresets: readonly string[];
    }
  | { readonly status: "unimplemented"; readonly issue: number }
  | { readonly status: "intentional-divergence"; readonly reason: string };

export interface RuleMap {
  readonly schemaVersion: number;
  readonly upstream: {
    readonly package: string;
    readonly version: string;
    readonly ruleCount: number;
  };
  readonly summary: {
    readonly mapped: number;
    readonly unimplemented: number;
    readonly intentionalDivergence: number;
  };
  readonly entries: Readonly<Record<string, RuleMapEntry>>;
}

interface PatinaRuleMeta {
  readonly defaultSeverity: string;
  readonly presets: string[];
}

function field(value: unknown, key: string): unknown {
  return value != null && (typeof value === "object" || typeof value === "function")
    ? (value as Readonly<Record<string, unknown>>)[key]
    : undefined;
}

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../../..");
export const ruleMapPath = path.join(
  repoRoot,
  "tests",
  "_fixtures",
  "patina-eslint-vue-rule-map.json",
);

const upstreamPackage = "eslint-plugin-vue";
const trackingIssue = 3223;
// Code-point ordering, not localeCompare: the rule map is a checked-in artifact
// that must be byte-identical across runners regardless of the host's ICU locale
// data, and the Rust-side drift test reads it as written.
const byCodePoint = (left: string, right: string): number =>
  left < right ? -1 : left > right ? 1 : 0;
const explicitPatinaAliases = new Map([
  ["vue/attributes-order", "vue/attribute-order"],
  ["vue/block-order", "vue/sfc-element-order"],
  ["vue/no-async-in-computed-properties", "script/no-async-in-computed"],
]);
const intentionalDivergenceOverrides = new Map([
  [
    "vue/component-definition-name-casing",
    "Patina's rule checks SFC file-name casing; eslint-plugin-vue checks the component definition name, so their findings are not comparable.",
  ],
  [
    "vue/no-unused-properties",
    "Patina intentionally checks defineProps declarations only; eslint-plugin-vue also checks Options API props, so the surfaces are not comparable.",
  ],
]);

function loadUpstream(): { ruleIds: string[]; version: string } {
  const requireFromBench = createRequire(
    path.join(repoRoot, "tools", "benchmarks", "scripts", "package.json"),
  );
  const plugin: unknown = requireFromBench(upstreamPackage);
  const version = field(requireFromBench(`${upstreamPackage}/package.json`), "version");
  const rules = field(plugin, "rules");
  assert.ok(typeof version === "string" && rules != null && typeof rules === "object");
  const benchManifest: unknown = JSON.parse(
    fs.readFileSync(path.join(repoRoot, "tools", "benchmarks", "scripts", "package.json"), "utf8"),
  );

  assert.equal(
    field(field(benchManifest, "devDependencies"), upstreamPackage),
    version,
    `${upstreamPackage} must stay exactly pinned in tools/benchmarks/scripts/package.json`,
  );

  return {
    ruleIds: Object.keys(rules)
      .map((name) => `vue/${name}`)
      .sort(),
    version,
  };
}

/**
 * Registered patina rules, keyed by name.
 *
 * The recorded `defaultSeverity` and `presets` are what lets a consumer decide,
 * from the checked-in map alone, which rules are comparable against upstream
 * under a given preset and at which severity — `tools/fixtures/lint-divergence.ts`
 * classifies on severity, so a baseline configured at the wrong one turns every
 * agreement into a false-positive/false-negative pair. Reading them here keeps
 * the native binding a `--write`-time dependency instead of a run-time one.
 */
async function loadPatinaRules(): Promise<Map<string, PatinaRuleMeta>> {
  const nativeEntry = process.env.VIZE_PATINA_NATIVE_MODULE
    ? path.resolve(process.env.VIZE_PATINA_NATIVE_MODULE)
    : path.join(repoRoot, "npm", "native", "index.js");
  const nativeModule: unknown = await import(pathToFileURL(nativeEntry).href);
  const binding = field(nativeModule, "default") ?? nativeModule;
  const getPatinaRules = field(binding, "getPatinaRules");
  assert.ok(
    typeof getPatinaRules === "function",
    "building the rule map requires a local @vizejs/native binding",
  );
  const listed: unknown = getPatinaRules.call(binding);
  assert.ok(Array.isArray(listed), "getPatinaRules must return a list");
  const rules = new Map<string, PatinaRuleMeta>();
  for (const rule of listed as readonly unknown[]) {
    const name = String(field(rule, "name"));
    const defaultSeverity = field(rule, "defaultSeverity");
    const presets = field(rule, "presets");
    assert.ok(typeof defaultSeverity === "string", `${name} has an unsupported default severity`);
    assert.match(
      defaultSeverity,
      /^(?:error|warning)$/u,
      `${name} has an unsupported default severity`,
    );
    assert.ok(Array.isArray(presets), `${name} must record its presets`);
    rules.set(name, {
      defaultSeverity,
      presets: (presets as readonly unknown[]).map(String).sort(byCodePoint),
    });
  }
  return rules;
}

function patinaTargetFor(
  upstreamRule: string,
  patinaRules: ReadonlyMap<string, PatinaRuleMeta>,
): string | null {
  if (patinaRules.has(upstreamRule)) {
    return upstreamRule;
  }

  const scriptRule = `script/${upstreamRule.slice("vue/".length)}`;
  if (patinaRules.has(scriptRule)) {
    return scriptRule;
  }

  const alias = explicitPatinaAliases.get(upstreamRule);
  return alias != null && patinaRules.has(alias) ? alias : null;
}

export async function generateRuleMap(): Promise<RuleMap> {
  const upstream = loadUpstream();
  const patinaRules = await loadPatinaRules();
  const entries: Record<string, RuleMapEntry> = {};
  let mapped = 0;
  let intentionalDivergence = 0;

  for (const ruleId of upstream.ruleIds) {
    const divergenceReason = intentionalDivergenceOverrides.get(ruleId);
    if (divergenceReason != null) {
      entries[ruleId] = {
        status: "intentional-divergence",
        reason: divergenceReason,
      };
      intentionalDivergence += 1;
      continue;
    }

    const patinaRule = patinaTargetFor(ruleId, patinaRules);
    if (patinaRule == null) {
      entries[ruleId] = { status: "unimplemented", issue: trackingIssue };
      continue;
    }
    const meta = patinaRules.get(patinaRule);
    if (meta === undefined) throw new Error(`${patinaRule} is not a registered patina rule`);
    entries[ruleId] = {
      status: "mapped",
      patinaRule,
      patinaSeverity: meta.defaultSeverity,
      patinaPresets: meta.presets,
    };
    mapped += 1;
  }

  return {
    schemaVersion: 1,
    upstream: {
      package: upstreamPackage,
      version: upstream.version,
      ruleCount: upstream.ruleIds.length,
    },
    summary: {
      mapped,
      unimplemented: upstream.ruleIds.length - mapped - intentionalDivergence,
      intentionalDivergence,
    },
    entries,
  };
}

export function readRuleMap(): RuleMap {
  // The checked-in map; validateRuleMap() checks every field against upstream.
  return JSON.parse(fs.readFileSync(ruleMapPath, "utf8")) as RuleMap;
}

export function validateRuleMap(ruleMap: RuleMap = readRuleMap()): RuleMap {
  const upstream = loadUpstream();

  assert.equal(ruleMap.schemaVersion, 1);
  assert.deepEqual(ruleMap.upstream, {
    package: upstreamPackage,
    version: upstream.version,
    ruleCount: upstream.ruleIds.length,
  });
  assert.deepEqual(
    Object.keys(ruleMap.entries),
    upstream.ruleIds,
    "every pinned eslint-plugin-vue rule must have exactly one sorted map entry",
  );

  let mapped = 0;
  let unimplemented = 0;
  let intentionalDivergence = 0;
  for (const [ruleId, entry] of Object.entries(ruleMap.entries)) {
    assert.equal(typeof entry, "object", `${ruleId} must have a structured map entry`);
    if (entry.status === "mapped") {
      assert.match(entry.patinaRule, /^(?:script|vue)\/[a-z0-9-]+$/u);
      assert.deepEqual(Object.keys(entry).sort(), [
        "patinaPresets",
        "patinaRule",
        "patinaSeverity",
        "status",
      ]);
      // Shape only. The recorded values are enforced against the live registry
      // by `eslint_vue_rule_map_matches_registered_patina_rules` in
      // `crates/vize_patina/src/preset/tests.rs`, which needs no native binding and so
      // can run on every PR.
      assert.match(entry.patinaSeverity, /^(?:error|warning)$/u, `${ruleId} needs a severity`);
      assert.ok(
        Array.isArray(entry.patinaPresets) &&
          entry.patinaPresets.every((preset) => /\S/u.test(preset)),
        `${ruleId} needs its patina preset membership`,
      );
      assert.deepEqual(
        entry.patinaPresets,
        [...entry.patinaPresets].sort(byCodePoint),
        `${ruleId} must list presets in sorted order`,
      );
      mapped += 1;
      continue;
    }
    if (entry.status === "unimplemented") {
      assert.equal(entry.issue, trackingIssue, `${ruleId} must link the scorecard issue`);
      assert.deepEqual(Object.keys(entry).sort(), ["issue", "status"]);
      unimplemented += 1;
      continue;
    }
    if (entry.status === "intentional-divergence") {
      assert.match(entry.reason, /\S/u, `${ruleId} needs a non-empty divergence reason`);
      assert.deepEqual(Object.keys(entry).sort(), ["reason", "status"]);
      intentionalDivergence += 1;
      continue;
    }
    assert.fail(`${ruleId} has unsupported status ${JSON.stringify(field(entry, "status"))}`);
  }

  assert.deepEqual(ruleMap.summary, { mapped, unimplemented, intentionalDivergence });
  assert.equal(
    mapped + unimplemented + intentionalDivergence,
    upstream.ruleIds.length,
    "the initial scorecard may not hide rules behind an uncounted status",
  );
  return ruleMap;
}

async function main(): Promise<void> {
  if (process.argv[1] == null || path.resolve(process.argv[1]) !== fileURLToPath(import.meta.url)) {
    return;
  }
  if (process.argv[2] === "--write") {
    const ruleMap = await generateRuleMap();
    fs.writeFileSync(ruleMapPath, `${JSON.stringify(ruleMap, null, 2)}\n`);
    return;
  }
  if (process.argv.length === 2 || process.argv[2] === "--check") {
    validateRuleMap();
    return;
  }
  throw new Error(
    "Usage: rust-script tools/commands/fixtures/patina-rule-map.rs [--check|--write]",
  );
}

await main();
