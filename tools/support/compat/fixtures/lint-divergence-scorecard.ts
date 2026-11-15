import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
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

type RuleSetting = string | [string, unknown];
type Rules = Readonly<Record<string, RuleSetting>>;

interface LintResult {
  readonly filePath: string;
  readonly messages: readonly unknown[];
  readonly [key: string]: unknown;
}

interface EslintInstance {
  calculateConfigForFile(file: string): Promise<unknown>;
  lintFiles(files: readonly string[]): Promise<LintResult[]>;
}

interface ScorecardRuntime {
  readonly ESLint: new (options: Readonly<Record<string, unknown>>) => EslintInstance;
}

interface BaselineGroup {
  readonly files: string[];
  readonly rules: Rules;
}

export interface ScorecardBaselineResult {
  readonly rawResults: LintResult[];
  readonly results: LintResult[];
  readonly droppedConfigMessageCount: number;
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

/** Apply the native rule's reviewed option semantics without mutating the historical oracle. */
export function configureScorecardRules(rules: Rules): Rules {
  const quoteProps = rules["vue/quote-props"];
  if (quoteProps === undefined) return rules;
  const severity = Array.isArray(quoteProps) ? quoteProps[0] : quoteProps;
  return { ...rules, "vue/quote-props": [severity, "as-needed"] };
}

/**
 * Run the current scorecard with per-file project context.
 *
 * The native CLI treats components installed by vue-router and Nuxt as globals
 * only when their package resolves from the linted file. Partitioning by those
 * exact boundaries keeps one parse per file and at most four ESLint instances
 * per bounded producer shard.
 */
export async function runScorecardBaseline(
  runtime: ScorecardRuntime,
  baselineConfig: (runtime: ScorecardRuntime, rules: Rules) => unknown,
  cwd: string,
  files: readonly string[],
  requestedRules: Rules,
): Promise<ScorecardBaselineResult> {
  const rules = requestedRules,
    groups = baselineGroups(cwd, files, requestedRules),
    rawResults: LintResult[] = [];
  for (const group of groups) {
    const eslint = new runtime.ESLint({
      cwd,
      passOnNoPatterns: group.files.length === 0,
      overrideConfigFile: true,
      overrideConfig: baselineConfig(runtime, group.rules),
      errorOnUnmatchedPattern: false,
    });
    if (group.files.length === 0) {
      await eslint.calculateConfigForFile("__vize_empty_vue_config__.vue");
    }
    rawResults.push(...(await eslint.lintFiles(group.files)));
  }

  const order = new Map(files.map((file, index) => [resolve(cwd, file), index]));
  rawResults.sort(
    (left, right) =>
      (order.get(resolve(left.filePath)) ?? Number.MAX_SAFE_INTEGER) -
      (order.get(resolve(right.filePath)) ?? Number.MAX_SAFE_INTEGER),
  );
  const enabled = new Set(Object.keys(rules));
  let droppedConfigMessageCount = 0;
  const results = rawResults.map((result) => {
    const messages = result.messages.filter((message) => {
      const ruleId = messageRuleId(message);
      if (ruleId === null || (typeof ruleId === "string" && enabled.has(ruleId))) return true;
      droppedConfigMessageCount++;
      return false;
    });
    return messages.length === result.messages.length ? result : { ...result, messages };
  });
  return { rawResults, results, droppedConfigMessageCount };
}

const routerComponentPattern = "^(?:RouterLink|RouterView)$";
const nuxtComponentPattern =
  "^(?:Nuxt|NuxtChild|NuxtPage|NuxtLayout|NuxtLink|NuxtLoadingIndicator|NuxtErrorBoundary|NuxtWelcome|NuxtIsland|NuxtRouteAnnouncer|ClientOnly|DevOnly|NoSsr)$";

function baselineGroups(cwd: string, files: readonly string[], rules: Rules): BaselineGroup[] {
  if (files.length === 0) return [{ files: [], rules }];
  const groups = new Map<string, BaselineGroup>(),
    routerResolutionCache = new Map<string, boolean>(),
    nuxtResolutionCache = new Map<string, boolean>();
  for (const file of files) {
    const contexts: string[] = [];
    let contextual = rules;
    if (packageIsResolvableFrom(cwd, file, "vue-router", routerResolutionCache)) {
      contextual = withIgnoredComponents(contextual, routerComponentPattern);
      contexts.push("vue-router");
    }
    if (packageIsResolvableFrom(cwd, file, "nuxt", nuxtResolutionCache)) {
      contextual = withIgnoredComponents(contextual, nuxtComponentPattern);
      contexts.push("nuxt");
    }
    const key = contexts.join("+") || "default",
      group = groups.get(key) ?? { files: [], rules: contextual };
    group.files.push(file);
    groups.set(key, group);
  }
  return [...groups.values()];
}

function withIgnoredComponents(rules: Rules, componentPattern: string): Rules {
  const setting = rules["vue/no-undef-components"];
  if (setting === undefined) return rules;
  const severity = Array.isArray(setting) ? setting[0] : setting,
    supplied = Array.isArray(setting) && isPlainObject(setting[1]) ? setting[1] : {},
    ignorePatterns = Array.isArray(supplied.ignorePatterns) ? supplied.ignorePatterns : [];
  if (ignorePatterns.includes(componentPattern)) return rules;
  return {
    ...rules,
    "vue/no-undef-components": [
      severity,
      { ...supplied, ignorePatterns: [...ignorePatterns, componentPattern] },
    ],
  };
}

function packageIsResolvableFrom(
  cwd: string,
  file: string,
  packageName: string,
  cache: Map<string, boolean>,
): boolean {
  let current = dirname(resolve(cwd, file));
  const visited: string[] = [];
  let available = false;
  for (;;) {
    const cached = cache.get(current);
    if (cached !== undefined || cache.has(current)) {
      available = cached ?? false;
      break;
    }
    visited.push(current);
    if (existsSync(join(current, "node_modules", packageName, "package.json"))) {
      available = true;
      break;
    }
    const parent = dirname(current);
    if (parent === current) break;
    current = parent;
  }
  for (const directory of visited) cache.set(directory, available);
  return available;
}

function isPlainObject(value: unknown): value is Record<string, unknown> {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

function messageRuleId(message: unknown): unknown {
  return isPlainObject(message) ? message.ruleId : undefined;
}
