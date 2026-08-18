import { createRequire } from "node:module";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..", "..");

/**
 * The `eslint-plugin-vue` half of a lint divergence run.
 *
 * The plugin, ESLint, and both parsers are pinned in `bench/package.json` — the
 * same pin `tools/fixtures/patina-rule-map.mjs` validates the rule map against —
 * so the baseline can never drift from the map that routes its findings.
 *
 * Only rules the map calls `mapped` are enabled, and only when the preset under
 * test actually activates their patina counterpart. A rule patina has but no
 * preset turns on cannot produce a finding, so leaving it enabled here would
 * report every upstream finding as a false negative and bury the real
 * divergences. Each enabled rule is configured at patina's own default severity,
 * because the comparator classifies on severity: a baseline set to `error` where
 * patina warns splits every agreement into a false-positive/false-negative pair
 * at one span.
 */
export function resolveBaselineRuntime() {
  const requireFromBench = createRequire(join(repoRoot, "bench", "package.json"));
  const manifest = requireFromBench("eslint-plugin-vue/package.json");
  return {
    ESLint: requireFromBench("eslint").ESLint,
    plugin: requireFromBench("eslint-plugin-vue"),
    vueParser: requireFromBench("vue-eslint-parser"),
    scriptParser: requireFromBench("@typescript-eslint/parser"),
    version: manifest.version,
  };
}

/**
 * Rules to compare, and the severity to compare them at.
 *
 * Pass `null` for `preset` to opt out of preset filtering; otherwise pass the
 * preset name the patina run used.
 *
 * With `includeUnimplemented`, rules the map records as a coverage gap are
 * enabled too. Their findings cannot be false negatives — the comparator routes
 * them into `unimplemented` by rule-map status — so this measures how many real
 * upstream findings a project loses by switching, without disturbing the
 * false-positive and false-negative counts. It is off by default because it
 * enables 129 more rules over the whole corpus for a number the parity verdict
 * does not depend on.
 */
export function selectComparableRules(ruleMap, preset, { includeUnimplemented = false } = {}) {
  const rules = {};
  const skippedByPreset = [];
  for (const [upstreamRule, entry] of Object.entries(ruleMap.entries)) {
    if (entry.status === "unimplemented") {
      if (includeUnimplemented) rules[upstreamRule] = "warn";
      continue;
    }
    if (entry.status !== "mapped") continue;
    if (preset != null && !entry.patinaPresets.includes(preset)) {
      skippedByPreset.push({ upstreamRule, patinaRule: entry.patinaRule });
      continue;
    }
    rules[upstreamRule] = entry.patinaSeverity === "error" ? "error" : "warn";
  }
  return { rules, skippedByPreset };
}

export function baselineConfig(runtime, rules) {
  return [
    {
      files: ["**/*.vue"],
      languageOptions: {
        parser: runtime.vueParser,
        parserOptions: {
          parser: runtime.scriptParser,
          ecmaVersion: "latest",
          sourceType: "module",
          ecmaFeatures: { jsx: true },
          extraFileExtensions: [".vue"],
        },
      },
      // Inline `eslint-disable` comments stay in force: patina honors them too
      // (`crates/vize_patina/src/context/eslint_directive.rs`), so switching them
      // off on one side alone would report every suppressed finding as a
      // divergence. Unused-directive reporting is off because it is a lint on the
      // configuration, not a finding about the code.
      linterOptions: { reportUnusedDisableDirectives: "off" },
      plugins: { vue: runtime.plugin },
      rules,
    },
  ];
}

/**
 * Keep only messages from rules this run actually enabled.
 *
 * Corpus sources carry `eslint-disable` comments naming rules from their own
 * toolchains (`@typescript-eslint/*`, `import/*`, `sonarjs/*`). ESLint reports
 * each unresolvable name as a problem whose `ruleId` is that rule, which is a
 * complaint about the fixture's configuration rather than a finding about its
 * code — and the comparator, correctly, treats an unknown baseline rule id as
 * drift between the pin and the installed plugin. Dropping them here keeps that
 * safety net meaningful: everything that survives is a rule the pinned rule map
 * is expected to know.
 *
 * `ruleId: null` messages (parse failures) are kept, because
 * `collectBaselineFindings` counts them as excluded evidence — a file the
 * reference parser cannot read must never look like parity.
 */
export function retainEnabledFindings(results, rules) {
  const enabled = new Set(Object.keys(rules));
  let droppedConfigMessageCount = 0;
  const retained = results.map((result) => {
    const messages = result.messages.filter((message) => {
      if (message.ruleId == null || enabled.has(message.ruleId)) return true;
      droppedConfigMessageCount += 1;
      return false;
    });
    return { ...result, messages };
  });
  return { results: retained, droppedConfigMessageCount };
}

/**
 * Lint `files` (fixture-relative) with the baseline and return ESLint's own
 * result objects, which `collectBaselineFindings` normalizes.
 *
 * `errorOnUnmatchedPattern` stays off because the corpus registry pins a
 * revision, not a working tree: a glob that matches nothing is the fixture's
 * business, and the file-count reconciliation in the caller is what catches a
 * corpus mismatch.
 */
export async function runBaseline(runtime, cwd, files, rules) {
  const eslint = new runtime.ESLint({
    cwd,
    overrideConfigFile: true,
    overrideConfig: baselineConfig(runtime, rules),
    errorOnUnmatchedPattern: false,
  });
  return retainEnabledFindings(await eslint.lintFiles(files), rules);
}
