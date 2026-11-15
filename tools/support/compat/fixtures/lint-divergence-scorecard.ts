import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

interface BenchmarkManifest {
  readonly devDependencies: Readonly<Record<string, string>>;
}

interface ProviderManifest {
  readonly name: string;
  readonly version: string;
}

interface RuleMap {
  readonly upstream: {
    readonly package: string;
    readonly version: string;
  };
}

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..", "..", "..", "..");
const upstreamPackage = "eslint-plugin-vue";
export const baselineProviderPackage = "eslint-plugin-vue-scorecard";

export function assertBaselineIdentity(
  benchManifest: BenchmarkManifest,
  providerManifest: ProviderManifest,
  ruleMap: RuleMap,
): void {
  const declared = benchManifest.devDependencies[baselineProviderPackage];
  assert.equal(ruleMap.upstream.package, upstreamPackage, "lint rule-map package drifted");
  assert.equal(providerManifest.name, upstreamPackage, "lint baseline package name drifted");
  assert.equal(
    declared,
    `npm:${upstreamPackage}@${providerManifest.version}`,
    `${baselineProviderPackage} must be an exact npm alias pin`,
  );
  assert.equal(
    providerManifest.version,
    ruleMap.upstream.version,
    `${baselineProviderPackage} version must match the lint rule map`,
  );
}

export function resolveScorecardRuntime() {
  const benchPackageJson = join(repoRoot, "tools", "benchmarks", "scripts", "package.json"),
    requireFromBench = createRequire(benchPackageJson),
    benchManifest = JSON.parse(readFileSync(benchPackageJson, "utf8")) as BenchmarkManifest,
    ruleMap = JSON.parse(
      readFileSync(join(repoRoot, "tests", "_fixtures", "patina-eslint-vue-rule-map.json"), "utf8"),
    ) as RuleMap,
    manifest = requireFromBench(`${baselineProviderPackage}/package.json`) as ProviderManifest;
  assertBaselineIdentity(benchManifest, manifest, ruleMap);
  return {
    ESLint: requireFromBench("eslint").ESLint,
    plugin: requireFromBench(baselineProviderPackage),
    vueParser: requireFromBench("vue-eslint-parser"),
    scriptParser: requireFromBench("@typescript-eslint/parser"),
    version: manifest.version,
    runtimeVersions: {
      eslint: (requireFromBench("eslint/package.json") as ProviderManifest).version,
      "eslint-plugin-vue": manifest.version,
      "vue-eslint-parser": (requireFromBench("vue-eslint-parser/package.json") as ProviderManifest)
        .version,
      "@typescript-eslint/parser": (
        requireFromBench("@typescript-eslint/parser/package.json") as ProviderManifest
      ).version,
    },
  };
}
