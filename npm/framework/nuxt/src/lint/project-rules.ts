/**
 * Project lint rules layered under the generated Nuxt oxlint config.
 *
 * The artifact keeps `settings.vize.preset` at `incremental` so Nuxt-only rules
 * stay enabled. A preset from the shared project config is therefore expanded into
 * explicit rule entries instead of replacing that runtime preset.
 */
import type { NuxtLintRuleConfig, NuxtLintSeverity } from "@vizejs/nuxt-lint-config";
import type { VizeRuleConfigPreset } from "oxlint-plugin-vize";
import * as fs from "node:fs";
import * as path from "node:path";
import { importConfigModule } from "../../../../cli/src/config/module-runtime.mjs";
import {
  VITE_CONFIG_FILE_NAMES,
  resolveViteConfigSource,
} from "../../../../cli/src/config/vite-runtime.mjs";

export type ProjectLintRules = Record<string, NuxtLintRuleConfig>;

export interface ProjectLintRuleEntry {
  files: string[];
  ignores?: string[];
  name?: string;
  rules: ProjectLintRules;
}

export interface ProjectLintRuleSets {
  entries?: ProjectLintRuleEntry[];
  vize?: ProjectLintRules;
  oxlint?: ProjectLintRules;
  warnings?: string[];
}

export type ProjectPresetLoader = (
  preset: VizeRuleConfigPreset,
  typeAware: boolean,
) => Promise<ProjectLintRules>;

const CONFIG_PRESETS = [
  "happy-path",
  "opinionated",
  "essential",
  "incremental",
  "ecosystem",
  "nuxt",
] as const;

const RETIRED_CONFIG = /^vize\.config\.(?:pkl|[cm]?[jt]s|json)$/;
const CONFIG_MIGRATION =
  "vize.config.* is no longer supported; move Vue settings to vue: in vite.config.* and TypeScript settings to tsconfig.json";

type ConfigPreset = (typeof CONFIG_PRESETS)[number];

interface ProjectLinterConfig {
  preset?: ConfigPreset;
  ruleOptions: Record<string, unknown>;
  typeAware: boolean;
  rules: ProjectLintRules;
}

interface ProjectLinterEntry extends ProjectLinterConfig {
  files: string[];
  ignores?: string[];
  name?: string;
}

export async function readProjectLintRules(
  rootDir: string,
  loadPreset: ProjectPresetLoader = loadPresetRules,
): Promise<ProjectLintRules | undefined> {
  return (await readProjectLintRuleSets(rootDir, loadPreset)).vize;
}

export async function readProjectLintRuleSets(
  rootDir: string,
  loadPreset: ProjectPresetLoader = loadPresetRules,
): Promise<ProjectLintRuleSets> {
  const project = await readProjectConfig(rootDir);
  const linter = project.linter;
  const oxlint = explicitOxlintRules(project.oxlint);
  const warnings: string[] = [];
  let vize: ProjectLintRules | undefined;
  if (linter) {
    const presetRules =
      linter.preset && linter.preset !== "incremental"
        ? await loadPreset(linter.preset, linter.typeAware)
        : {};
    const rules = applyRuleOptions(
      { ...presetRules, ...linter.rules },
      linter.ruleOptions,
      warnings,
      "vue.linter",
    );
    if (Object.keys(rules).length > 0) vize = rules;
  }
  const entries = project.entries
    .map((entry, index): ProjectLintRuleEntry | undefined => {
      const rules = applyRuleOptions(
        { ...entry.rules },
        entry.ruleOptions,
        warnings,
        `vue.entries[${String(index)}].linter`,
        vize,
      );
      if (Object.keys(rules).length === 0) return undefined;
      return {
        files: entry.files,
        ...(entry.ignores ? { ignores: entry.ignores } : {}),
        ...(entry.name ? { name: entry.name } : {}),
        rules,
      };
    })
    .filter((entry): entry is ProjectLintRuleEntry => entry !== undefined);
  return {
    ...(entries.length > 0 ? { entries } : {}),
    ...(vize ? { vize } : {}),
    ...(Object.keys(oxlint).length > 0 ? { oxlint } : {}),
    ...(warnings.length > 0 ? { warnings } : {}),
  };
}

interface ProjectConfig {
  entries: ProjectLinterEntry[];
  linter?: ProjectLinterConfig;
  oxlint?: unknown;
}

async function readProjectConfig(rootDir: string): Promise<ProjectConfig> {
  const parsed = await loadProjectViteConfigSource(rootDir);
  if (!parsed) return { entries: [] };
  const rawVue = parsed.vue && typeof parsed.vue === "object" ? parsed.vue : undefined;
  const rawLinter = rawVue && "linter" in rawVue ? projectLinter(rawVue.linter) : undefined;
  const entries = rawVue && "entries" in rawVue ? projectEntries(rawVue.entries) : [];
  return {
    entries,
    linter: rawLinter,
    oxlint: parsed.lint,
  };
}

function projectLinter(value: unknown): ProjectLinterConfig | undefined {
  if (!value || typeof value !== "object") return undefined;
  const raw = value as {
    preset?: unknown;
    ruleOptions?: unknown;
    rules?: unknown;
    typeAware?: unknown;
  };
  return {
    preset: isConfigPreset(raw.preset) ? raw.preset : undefined,
    ruleOptions: explicitRuleOptions(raw.ruleOptions),
    typeAware: raw.typeAware === true,
    rules: explicitRules(raw.rules),
  };
}

function projectEntries(value: unknown): ProjectLinterEntry[] {
  if (!Array.isArray(value)) return [];
  return value.flatMap((candidate) => {
    if (!candidate || typeof candidate !== "object") return [];
    const raw = candidate as {
      basePath?: unknown;
      files?: unknown;
      ignores?: unknown;
      linter?: unknown;
      name?: unknown;
    };
    const linter = projectLinter(raw.linter);
    const files = scopedGlobs(raw.files, raw.basePath);
    if (!linter || files.length === 0) return [];
    const ignores = scopedGlobs(raw.ignores, raw.basePath);
    return [
      {
        ...linter,
        files,
        ...(ignores.length > 0 ? { ignores } : {}),
        ...(typeof raw.name === "string" && raw.name ? { name: raw.name } : {}),
      },
    ];
  });
}

function scopedGlobs(value: unknown, basePath: unknown): string[] {
  if (!Array.isArray(value)) return [];
  const base = typeof basePath === "string" && basePath ? basePath : undefined;
  return value.flatMap((glob) => {
    if (typeof glob !== "string" || !glob) return [];
    return [base && !path.isAbsolute(glob) ? path.join(base, glob) : glob];
  });
}

async function loadProjectViteConfigSource(
  rootDir: string,
): Promise<Record<string, unknown> | null> {
  let directory = path.resolve(rootDir);
  while (true) {
    for (const name of VITE_CONFIG_FILE_NAMES) {
      const filePath = path.join(directory, name);
      if (!fs.existsSync(filePath)) continue;
      const module = await importConfigModule(filePath);
      if (module === null) throw new Error("Unable to load Vite configuration at " + filePath);
      return resolveViteConfigSource(module.default ?? module, {
        command: "lint",
        mode: "development",
      });
    }
    if (
      fs.existsSync(directory) &&
      fs.readdirSync(directory).some((name) => RETIRED_CONFIG.test(name))
    ) {
      throw new Error(CONFIG_MIGRATION);
    }
    if (
      ["package.json", "tsconfig.json", "jsconfig.json"].some((name) =>
        fs.existsSync(path.join(directory, name)),
      )
    ) {
      return null;
    }
    const parent = path.dirname(directory);
    if (parent === directory) return null;
    directory = parent;
  }
}

function explicitOxlintRules(value: unknown): ProjectLintRules {
  if (!value || typeof value !== "object") return {};
  const rules = "rules" in value ? (value as { rules?: unknown }).rules : undefined;
  if (!rules || typeof rules !== "object") return {};
  const result: ProjectLintRules = {};
  for (const [id, severity] of Object.entries(rules)) {
    if (isSeverity(severity)) result[id] = severity;
  }
  return result;
}

async function loadPresetRules(
  preset: VizeRuleConfigPreset,
  typeAware: boolean,
): Promise<ProjectLintRules> {
  const { createVizeRuleConfig } = await import("oxlint-plugin-vize");
  const generated = createVizeRuleConfig({ includeTypeAware: typeAware, preset });
  const rules: ProjectLintRules = {};
  for (const [id, severity] of Object.entries(generated)) {
    if (!isSeverity(severity)) continue;
    const bare = id.startsWith("vize/") ? id.slice("vize/".length) : id;
    if (bare.length > 0) rules[bare] = severity;
  }
  return rules;
}

function explicitRules(value: unknown): ProjectLintRules {
  if (!value || typeof value !== "object") return {};
  const rules: ProjectLintRules = {};
  for (const [id, severity] of Object.entries(value)) {
    if (!isSeverity(severity)) continue;
    const bare = id.startsWith("vize/") ? id.slice("vize/".length) : id;
    if (bare.length > 0) rules[bare] = severity;
  }
  return rules;
}

function explicitRuleOptions(value: unknown): Record<string, unknown> {
  if (!value || typeof value !== "object") return {};
  const options: Record<string, unknown> = {};
  for (const [id, option] of Object.entries(value)) {
    const bare = id.startsWith("vize/") ? id.slice("vize/".length) : id;
    if (bare.length > 0) options[bare] = option;
  }
  return options;
}

function applyRuleOptions(
  source: ProjectLintRules,
  options: Record<string, unknown>,
  warnings: string[],
  scope: string,
  fallback: ProjectLintRules = {},
): ProjectLintRules {
  const rules = { ...source };
  for (const [ruleId, option] of Object.entries(options)) {
    const configured = rules[ruleId] ?? fallback[ruleId];
    const severity = Array.isArray(configured) ? configured[0] : configured;
    if (!isSeverity(severity)) {
      warnings.push(
        `${scope}.ruleOptions[${JSON.stringify(ruleId)}] has no enabled rule severity to attach to`,
      );
      continue;
    }
    // `createVizeLintConfig` accepts the shared CLI option shape directly;
    // keep the generated Nuxt config on that same bridge contract.
    rules[ruleId] = [severity, option];
  }
  return rules;
}

function isConfigPreset(value: unknown): value is ConfigPreset {
  return typeof value === "string" && (CONFIG_PRESETS as readonly string[]).includes(value);
}

function isSeverity(value: unknown): value is NuxtLintSeverity {
  return value === "off" || value === "warn" || value === "error";
}
