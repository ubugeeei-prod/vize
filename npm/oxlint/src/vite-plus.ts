import { getPatinaRules } from "./binding.js";
import { createVizeRuleConfig, type OxlintRuleConfig } from "./configs.js";
import type { PatinaPreset, PatinaRuleMeta, PatinaSettings } from "./model.js";

/**
 * Bare specifier Oxlint uses to load this bridge as a JS plugin.
 *
 * Vite+ hands the `lint` block to Oxlint unchanged, so the same specifier works
 * in `vite.config.ts` and in a hand-written `.oxlintrc.json`.
 */
export const VIZE_JS_PLUGIN_SPECIFIER = "oxlint-plugin-vize";

const VIZE_RULE_PREFIX = "vize/";

/** Built-in plugin names accepted by Vite+'s Oxlint configuration. */
export type VitePlusLintPlugin =
  | "eslint"
  | "import"
  | "jest"
  | "jsdoc"
  | "jsx-a11y"
  | "nextjs"
  | "node"
  | "oxc"
  | "promise"
  | "react"
  | "react-perf"
  | "typescript"
  | "unicorn"
  | "vitest"
  | "vue";

/**
 * Rule bundles `createVizeLintConfig` accepts.
 *
 * `"all"` and `"incremental"` both disable preset gating in the bridge, which is
 * what makes them usable: `"all"` needs every bundle's rules to run at once, and
 * `"incremental"` needs only the rules the caller lists to run.
 */
export type VizeLintPreset = PatinaPreset | "all";

export interface VizeLintConfigOptions {
  /**
   * Include Vize's unstable `vize/type/*` rules in the preset rule map. Off by
   * default, matching `createVizeRuleConfig`.
   */
  includeTypeAware?: boolean;
  /**
   * Built-in Oxlint plugins to keep enabled alongside `vue`.
   *
   * The emitted block always lists `vue`, and narrowing a project's plugin list
   * would silently drop the diagnostics those plugins produce, so anything passed
   * here is merged rather than replaced. A `create-vue` project, for example,
   * needs `["eslint", "typescript", "unicorn", "oxc"]` carried over.
   */
  plugins?: readonly VitePlusLintPlugin[];
  /**
   * Rule bundle to enable. Defaults to `"general-recommended"`, the bridge's own
   * default. `"incremental"` emits no preset rules, so only `rules` run.
   */
  preset?: VizeLintPreset;
  /**
   * Extra Oxlint rules merged after the preset rules. Core Oxlint rule ids pass
   * through untouched; `vize/*` ids are validated against the rules the native
   * bridge registers and against the resolved preset.
   */
  rules?: OxlintRuleConfig;
  /**
   * Patina runtime settings forwarded through `settings.vize`. `preset` is
   * intentionally absent: it is derived from `options.preset` so the rule map and
   * the bridge's runtime gate can never disagree.
   */
  settings?: Omit<PatinaSettings, "preset">;
}

export interface VizeLintConfig {
  jsPlugins: string[];
  plugins: VitePlusLintPlugin[];
  rules: OxlintRuleConfig;
  settings: { vize: PatinaSettings };
}

/**
 * Builds the Oxlint configuration block that runs Patina through this bridge.
 *
 * Vite+ (`vp lint` / `vp check`) reads its Oxlint configuration from the `lint`
 * key of `vite.config.ts` and never reads `.oxlintrc.json`. Wiring the bridge by
 * hand therefore has a silent failure mode: a `.oxlintrc.json` carrying
 * `jsPlugins` and `vize/*` rules looks configured, Vite+ ignores the file, Oxlint
 * never sees a `vize/*` rule id, and `vp lint` reports zero Vize diagnostics
 * while exiting `0`. Spreading this object into `lint` removes the chance to get
 * that wrong:
 *
 * ```ts
 * import { defineConfig } from "vite-plus";
 * import { createVizeLintConfig } from "oxlint-plugin-vize";
 *
 * export default defineConfig({
 *   lint: createVizeLintConfig({ preset: "essential" }),
 * });
 * ```
 *
 * Both validations below exist because the bridge's normal failure modes are
 * silent rather than loud:
 *
 * - An unknown `vize/*` id only produces Oxlint's
 *   `Rule '...' not found in plugin 'vize'` error when Oxlint actually reads the
 *   config. A typo in a config Oxlint never reads reports nothing at all.
 * - A `vize/*` id that is outside the active preset is dropped by the bridge's
 *   runtime preset gate (`plugin.ts`), so it stays listed in `rules` and reports
 *   nothing. `preset: "incremental"` is the supported way to run an arbitrary
 *   subset.
 *
 * @throws {Error} When `options.rules` names a `vize/*` id the native bridge does
 *   not register, or one the resolved preset would silently suppress.
 */
export function createVizeLintConfig(options: VizeLintConfigOptions = {}): VizeLintConfig {
  const preset = options.preset ?? "general-recommended";
  const extraRules = options.rules ?? {};
  assertUsableVizeRules(extraRules, preset);

  return {
    jsPlugins: [VIZE_JS_PLUGIN_SPECIFIER],
    plugins: [...new Set(["vue", ...(options.plugins ?? [])])],
    rules: {
      ...createPresetRules(preset, options.includeTypeAware),
      ...extraRules,
    },
    settings: {
      vize: {
        ...options.settings,
        preset: toRuntimePreset(preset),
      },
    },
  };
}

function createPresetRules(
  preset: VizeLintPreset,
  includeTypeAware: boolean | undefined,
): OxlintRuleConfig {
  // "incremental" means "run exactly what the caller listed", so it contributes
  // no preset rules of its own.
  return preset === "incremental" ? {} : createVizeRuleConfig({ includeTypeAware, preset });
}

/**
 * Maps a rule bundle to the `settings.vize.preset` the bridge must run with.
 *
 * `"all"` becomes `"incremental"` because the bridge gates each rule on preset
 * membership; gating an all-bundles rule map by any single preset would suppress
 * the rules that only belong to the other bundles.
 */
function toRuntimePreset(preset: VizeLintPreset): PatinaPreset {
  return preset === "all" ? "incremental" : preset;
}

function assertUsableVizeRules(rules: OxlintRuleConfig, preset: VizeLintPreset): void {
  const ruleMetaById = new Map(
    getPatinaRules().map((ruleMeta) => [`${VIZE_RULE_PREFIX}${ruleMeta.name}`, ruleMeta]),
  );
  const configuredIds = Object.keys(rules)
    .filter((ruleId) => ruleId.startsWith(VIZE_RULE_PREFIX))
    .sort();

  const unknownIds = configuredIds.filter((ruleId) => !ruleMetaById.has(ruleId));
  if (unknownIds.length > 0) {
    throw new Error(
      `Unknown Vize rule ${pluralizeIds(unknownIds)}: ${unknownIds.join(", ")}. ` +
        "Check the id against the rules oxlint-plugin-vize registers.",
    );
  }

  const runtimePreset = toRuntimePreset(preset);
  const suppressedIds = configuredIds.filter((ruleId) =>
    isSuppressedByPreset(ruleMetaById.get(ruleId) as PatinaRuleMeta, runtimePreset),
  );
  if (suppressedIds.length > 0) {
    throw new Error(
      `Vize rule ${pluralizeIds(suppressedIds)} outside the "${preset}" preset: ${suppressedIds.join(", ")}. ` +
        'Use preset: "incremental" to run an explicit rule subset, or pick the preset that owns these rules.',
    );
  }
}

/**
 * Mirrors the runtime gate in `plugin.ts`: rules with no preset membership are
 * never gated, and `"incremental"` skips gating entirely.
 */
function isSuppressedByPreset(ruleMeta: PatinaRuleMeta, runtimePreset: PatinaPreset): boolean {
  return (
    ruleMeta.presets.length > 0 &&
    runtimePreset !== "incremental" &&
    !ruleMeta.presets.includes(runtimePreset)
  );
}

function pluralizeIds(ids: readonly string[]): string {
  return ids.length === 1 ? "id" : "ids";
}
