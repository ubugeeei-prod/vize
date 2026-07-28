import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";
import { fileURLToPath, pathToFileURL } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
export const ruleMapPath = path.join(
  repoRoot,
  "tests",
  "_fixtures",
  "patina-eslint-vue-rule-map.json",
);

const upstreamPackage = "eslint-plugin-vue";
const trackingIssue = 3223;
const explicitPatinaAliases = new Map([
  ["vue/attributes-order", "vue/attribute-order"],
  ["vue/block-order", "vue/sfc-element-order"],
  ["vue/no-async-in-computed-properties", "script/no-async-in-computed"],
]);

function loadUpstream() {
  const requireFromBench = createRequire(path.join(repoRoot, "bench", "package.json"));
  const plugin = requireFromBench(upstreamPackage);
  const manifest = requireFromBench(`${upstreamPackage}/package.json`);
  const benchManifest = JSON.parse(
    fs.readFileSync(path.join(repoRoot, "bench", "package.json"), "utf8"),
  );

  assert.equal(
    benchManifest.devDependencies?.[upstreamPackage],
    manifest.version,
    `${upstreamPackage} must stay exactly pinned in bench/package.json`,
  );

  return {
    ruleIds: Object.keys(plugin.rules)
      .map((name) => `vue/${name}`)
      .sort(),
    version: manifest.version,
  };
}

async function loadPatinaRuleNames() {
  const nativeEntry = process.env.VIZE_PATINA_NATIVE_MODULE
    ? path.resolve(process.env.VIZE_PATINA_NATIVE_MODULE)
    : path.join(repoRoot, "npm", "native", "index.js");
  const nativeModule = await import(pathToFileURL(nativeEntry));
  const binding = nativeModule.default ?? nativeModule;
  assert.equal(
    typeof binding.getPatinaRules,
    "function",
    "building the rule map requires a local @vizejs/native binding",
  );
  return new Set(binding.getPatinaRules().map((rule) => rule.name));
}

function patinaTargetFor(upstreamRule, patinaRules) {
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

export async function generateRuleMap() {
  const upstream = loadUpstream();
  const patinaRules = await loadPatinaRuleNames();
  const entries = {};
  let mapped = 0;

  for (const ruleId of upstream.ruleIds) {
    const patinaRule = patinaTargetFor(ruleId, patinaRules);
    if (patinaRule == null) {
      entries[ruleId] = { status: "unimplemented", issue: trackingIssue };
      continue;
    }
    entries[ruleId] = { status: "mapped", patinaRule };
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
      unimplemented: upstream.ruleIds.length - mapped,
    },
    entries,
  };
}

export function readRuleMap() {
  return JSON.parse(fs.readFileSync(ruleMapPath, "utf8"));
}

export function validateRuleMap(ruleMap = readRuleMap()) {
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
  for (const [ruleId, entry] of Object.entries(ruleMap.entries)) {
    assert.equal(typeof entry, "object", `${ruleId} must have a structured map entry`);
    if (entry.status === "mapped") {
      assert.match(entry.patinaRule, /^(?:script|vue)\/[a-z0-9-]+$/u);
      assert.deepEqual(Object.keys(entry).sort(), ["patinaRule", "status"]);
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
      continue;
    }
    assert.fail(`${ruleId} has unsupported status ${JSON.stringify(entry.status)}`);
  }

  assert.deepEqual(ruleMap.summary, { mapped, unimplemented });
  assert.equal(
    mapped + unimplemented,
    upstream.ruleIds.length,
    "the initial scorecard may not hide rules behind an uncounted status",
  );
  return ruleMap;
}

async function main() {
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
  throw new Error("Usage: node tools/fixtures/patina-rule-map.mjs [--check|--write]");
}

await main();
