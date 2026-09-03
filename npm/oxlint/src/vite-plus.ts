import { getPatinaRules } from "./binding.js";
import {
  createVizeRuleConfig,
  type OxlintRuleConfig,
  type VizeRuleConfigPreset,
} from "./configs.js";
import type { PatinaPreset, PatinaRuleMeta, PatinaSettings } from "./model.js";

/**
 * Bare specifier Oxlint uses to load this bridge as a JS plugin.
 *
 * Vite+ hands the `lint` block to Oxlint unchanged, so the same specifier works
 * in `vite.config.ts` and in a hand-written `.oxlintrc.json`.
 */
export const VIZE_JS_PLUGIN_SPECIFIER = "oxlint-plugin-vize";

const VIZE_RULE_PREFIX = "vize/";
const DEFAULT_VIZE_LINT_PRESET = "general-recommended" satisfies PatinaPreset;

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
 * `"all"`, `"incremental"`, and multiple-preset selections disable preset
 * gating in the bridge. That is what makes them usable: the runtime only accepts
 * one preset, while those shapes need exactly the emitted rule map to run.
 */
export type VizeLintPreset = PatinaPreset | "happy-path" | "all";
export type VizeLintPresetInput = VizeLintPreset | readonly VizeLintPreset[];
type NormalizedVizeLintPreset = PatinaPreset | "all";

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
   * Rule bundle or bundles to enable. Defaults to `"general-recommended"`, the
   * bridge's own default. `"incremental"` emits no preset rules, so only `rules`
   * run.
   */
  preset?: VizeLintPresetInput;
  /**
   * Additional rule bundles to enable alongside `preset`. This is equivalent to
   * passing an array to `preset`, but reads better when the base preset is kept
   * separate from project-specific add-ons.
   */
  presets?: readonly VizeLintPreset[];
  /**
   * Extra Oxlint rules merged after the preset rules. Core Oxlint rule ids pass
   * through untouched; `vize/*` ids are validated against the rules the native
   * bridge registers and, for a single gated preset, against the resolved preset.
   */
  rules?: OxlintRuleConfig;
  /**
   * Patina runtime settings forwarded through `settings.vize`. `preset` is
   * intentionally absent: it is derived from `options.preset`/`options.presets`
   * so the rule map and the bridge's runtime gate can never disagree.
   */
  settings?: Omit<PatinaSettings, "preset">;
}

export interface VizeLintConfigSettings extends Record<string, unknown> {
  vize: PatinaSettings;
}

export interface VizeLintConfig extends Record<string, unknown> {
  jsPlugins: string[];
  plugins: VitePlusLintPlugin[];
  rules: OxlintRuleConfig;
  settings: VizeLintConfigSettings;
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
 * - A `vize/*` id that is outside a single active preset is dropped by the
 *   bridge's runtime preset gate (`plugin.ts`), so it stays listed in `rules` and
 *   reports nothing. `preset: "incremental"` is the supported way to run an
 *   arbitrary subset.
 *
 * @throws {Error} When `options.rules` names a `vize/*` id the native bridge does
 *   not register, or one a single resolved preset would silently suppress.
 */
export function createVizeLintConfig(options: VizeLintConfigOptions = {}): VizeLintConfig {
  return buildVizeLintConfig(options, { forceIncrementalRuntime: false });
}

export function buildVizeLintConfig(
  options: VizeLintConfigOptions,
  buildOptions: { forceIncrementalRuntime: boolean },
): VizeLintConfig {
  const presets = resolveLintPresets(options);
  const runtimePreset = buildOptions.forceIncrementalRuntime
    ? "incremental"
    : toRuntimePreset(presets);
  const extraRules = options.rules ?? {};
  assertUsableVizeRules(extraRules, presets, runtimePreset);

  return {
    jsPlugins: [VIZE_JS_PLUGIN_SPECIFIER],
    plugins: [...new Set(["vue", ...(options.plugins ?? [])])],
    rules: {
      ...createPresetRules(presets, options.includeTypeAware),
      ...extraRules,
    },
    settings: {
      vize: {
        ...options.settings,
        preset: runtimePreset,
      },
    },
  };
}

function createPresetRules(
  presets: readonly NormalizedVizeLintPreset[],
  includeTypeAware: boolean | undefined,
): OxlintRuleConfig {
  const ruleConfigPresets = presets.filter(isRuleConfigPreset);
  if (ruleConfigPresets.length === 0) {
    return {};
  }

  return createVizeRuleConfig({ includeTypeAware, presets: ruleConfigPresets });
}

function resolveLintPresets(
  options: Pick<VizeLintConfigOptions, "preset" | "presets">,
): NormalizedVizeLintPreset[] {
  const presetInput =
    options.preset === undefined && options.presets === undefined
      ? DEFAULT_VIZE_LINT_PRESET
      : options.preset;
  const requestedPresets = [...toPresetArray(presetInput ?? []), ...(options.presets ?? [])].map(
    normalizeLintPreset,
  );

  return [...new Set(requestedPresets)];
}

function toPresetArray(preset: VizeLintPresetInput | readonly []): readonly VizeLintPreset[] {
  return Array.isArray(preset) ? preset : [preset];
}

function normalizeLintPreset(preset: VizeLintPreset): NormalizedVizeLintPreset {
  return preset === "happy-path" ? DEFAULT_VIZE_LINT_PRESET : preset;
}

function isRuleConfigPreset(preset: NormalizedVizeLintPreset): preset is VizeRuleConfigPreset {
  return preset !== "incremental";
}

/**
 * Maps rule bundles to the `settings.vize.preset` the bridge must run with.
 *
 * `"all"` and multiple concrete presets become `"incremental"` because the
 * bridge gates each rule on preset membership; gating a combined rule map by any
 * single preset would suppress the rules that only belong to the other bundles.
 */
function toRuntimePreset(presets: readonly NormalizedVizeLintPreset[]): PatinaPreset {
  if (presets.length === 1 && presets[0] !== "all") {
    return presets[0];
  }

  return "incremental";
}

function assertUsableVizeRules(
  rules: OxlintRuleConfig,
  presets: readonly NormalizedVizeLintPreset[],
  runtimePreset: PatinaPreset,
): void {
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

  const suppressedIds = configuredIds.filter((ruleId) =>
    isSuppressedByPreset(ruleMetaById.get(ruleId) as PatinaRuleMeta, runtimePreset),
  );
  if (suppressedIds.length > 0) {
    throw new Error(
      `Vize rule ${pluralizeIds(suppressedIds)} outside ${describePresetSelection(presets)}: ${suppressedIds.join(", ")}. ` +
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

function describePresetSelection(presets: readonly NormalizedVizeLintPreset[]): string {
  const names = presets.map((preset) =>
    preset === DEFAULT_VIZE_LINT_PRESET ? "happy-path" : preset,
  );
  const label = names.length === 0 ? "empty" : names.join(", ");
  return `the "${label}" ${names.length === 1 ? "preset" : "presets"}`;
}

function pluralizeIds(ids: readonly string[]): string {
  return ids.length === 1 ? "id" : "ids";
}
