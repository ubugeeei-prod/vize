import type { PatinaPreset, PatinaSettings } from "./model.js";
import {
  buildVizeLintConfig,
  type VitePlusLintPlugin,
  type VizeLintConfig,
  type VizeLintConfigOptions,
  type VizeLintConfigSettings,
} from "./vite-plus.js";

export interface VizeLintConfigFragment extends Record<string, unknown> {
  extends?: readonly (VizeLintConfigFragment | VizeLintFlatConfig)[];
  ignorePatterns?: readonly string[];
  jsPlugins?: readonly string[];
  overrides?: readonly unknown[];
  plugins?: readonly VitePlusLintPlugin[];
  rules?: VizeLintConfig["rules"];
  settings?: Record<string, unknown> & { vize?: PatinaSettings | Record<string, unknown> };
}

export type VizeLintFlatConfig = readonly VizeLintConfigFragment[];

/**
 * Returns spreadable Oxlint/Vite+ config fragments for Flat Config style use.
 *
 * The fragments intentionally set `settings.vize.preset` to `"incremental"` so
 * multiple fragments can be composed without one preset's runtime gate
 * suppressing rules emitted by another fragment.
 */
export function createVizeLintFlatConfig(options: VizeLintConfigOptions = {}): VizeLintFlatConfig {
  return [buildVizeLintConfig(options, { forceIncrementalRuntime: true })];
}

export const flatConfigs = {
  all: createVizeLintFlatConfig({ preset: "all" }),
  ecosystem: createVizeLintFlatConfig({ preset: "ecosystem" }),
  ecosystemWithTypeAware: createVizeLintFlatConfig({
    includeTypeAware: true,
    preset: "ecosystem",
  }),
  essential: createVizeLintFlatConfig({ preset: "essential" }),
  happyPath: createVizeLintFlatConfig({ preset: "happy-path" }),
  happyPathWithTypeAware: createVizeLintFlatConfig({
    includeTypeAware: true,
    preset: "happy-path",
  }),
  nuxt: createVizeLintFlatConfig({ preset: "nuxt" }),
  opinionated: createVizeLintFlatConfig({ preset: "opinionated" }),
  opinionatedWithTypeAware: createVizeLintFlatConfig({
    includeTypeAware: true,
    preset: "opinionated",
  }),
  recommended: createVizeLintFlatConfig({ preset: "general-recommended" }),
  recommendedWithTypeAware: createVizeLintFlatConfig({
    includeTypeAware: true,
    preset: "general-recommended",
  }),
} satisfies Record<string, VizeLintFlatConfig>;

/**
 * Collapses Flat Config-style fragments into the object shape Vite+ accepts.
 *
 * `vp lint` currently expects a single Oxlint config object under `lint`, so this
 * helper lets `vite.config.ts` keep a flat, spread-friendly authoring style while
 * still handing Vite+ the object it can read.
 */
export function defineVizeLintConfig(
  ...entries: readonly (VizeLintConfigFragment | VizeLintFlatConfig)[]
): VizeLintConfig {
  const config: VizeLintConfig = {
    jsPlugins: [],
    plugins: [],
    rules: {},
    settings: { vize: {} },
  };
  const runtimePresets: PatinaPreset[] = [];

  for (const entry of entries) {
    mergeFlatConfigEntry(config, runtimePresets, entry);
  }

  config.jsPlugins = [...new Set(config.jsPlugins)];
  config.plugins = [...new Set(config.plugins)];
  if (runtimePresets.includes("incremental")) {
    config.settings.vize.preset = "incremental";
  } else {
    const uniquePresets = [...new Set(runtimePresets)];
    if (uniquePresets.length === 1) {
      config.settings.vize.preset = uniquePresets[0];
    } else if (uniquePresets.length > 1) {
      config.settings.vize.preset = "incremental";
    }
  }

  return config;
}

function mergeFlatConfigEntry(
  target: VizeLintConfig,
  runtimePresets: PatinaPreset[],
  entry: VizeLintConfigFragment | VizeLintFlatConfig,
): void {
  if (Array.isArray(entry)) {
    for (const item of entry) {
      mergeFlatConfigEntry(target, runtimePresets, item);
    }
    return;
  }

  for (const extended of entry.extends ?? []) {
    mergeFlatConfigEntry(target, runtimePresets, extended);
  }

  mergeFlatConfigItem(target, runtimePresets, entry);
}

function mergeFlatConfigItem(
  target: VizeLintConfig,
  runtimePresets: PatinaPreset[],
  item: VizeLintConfigFragment,
): void {
  if (item.jsPlugins !== undefined) {
    target.jsPlugins.push(...item.jsPlugins);
  }
  if (item.plugins !== undefined) {
    target.plugins.push(...item.plugins);
  }
  if (item.rules !== undefined) {
    target.rules = { ...target.rules, ...item.rules };
  }
  if (item.ignorePatterns !== undefined) {
    target.ignorePatterns = [
      ...new Set([...(readStringArray(target.ignorePatterns) ?? []), ...item.ignorePatterns]),
    ];
  }
  if (item.overrides !== undefined) {
    target.overrides = [...(readUnknownArray(target.overrides) ?? []), ...item.overrides];
  }
  if (item.settings !== undefined) {
    mergeSettings(target.settings, runtimePresets, item.settings);
  }

  for (const [key, value] of Object.entries(item)) {
    if (
      value === undefined ||
      key === "extends" ||
      key === "ignorePatterns" ||
      key === "jsPlugins" ||
      key === "overrides" ||
      key === "plugins" ||
      key === "rules" ||
      key === "settings"
    ) {
      continue;
    }

    target[key] = value;
  }
}

function mergeSettings(
  target: VizeLintConfigSettings,
  runtimePresets: PatinaPreset[],
  settings: Record<string, unknown> & { vize?: PatinaSettings | Record<string, unknown> },
): void {
  for (const [key, value] of Object.entries(settings)) {
    if (key !== "vize") {
      target[key] = value;
    }
  }

  if (isRecord(settings.vize)) {
    const runtimePreset = normalizeRuntimePresetSetting(settings.vize.preset);
    if (runtimePreset !== null) {
      runtimePresets.push(runtimePreset);
    }
    target.vize = { ...target.vize, ...settings.vize };
  }
}

function normalizeRuntimePresetSetting(value: unknown): PatinaPreset | null {
  if (typeof value !== "string") {
    return null;
  }

  const normalized = value.replaceAll(/[-_\s]/gu, "").toLowerCase();
  switch (normalized) {
    case "generalrecommended":
    case "happypath":
    case "happy":
    case "default":
    case "recommended":
      return "general-recommended";
    case "essential":
      return "essential";
    case "ecosystem":
    case "eco":
      return "ecosystem";
    case "incremental":
    case "all":
      return "incremental";
    case "opinionated":
    case "opnionated":
    case "strict":
      return "opinionated";
    case "nuxt":
      return "nuxt";
    default:
      return null;
  }
}

function readStringArray(value: unknown): string[] | null {
  if (!Array.isArray(value) || !value.every((entry) => typeof entry === "string")) {
    return null;
  }

  return value;
}

function readUnknownArray(value: unknown): unknown[] | null {
  return Array.isArray(value) ? value : null;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}
