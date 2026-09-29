/**
 * Project lint rules layered under the generated Nuxt oxlint config.
 *
 * The artifact keeps `settings.vize.preset` at `incremental` so Nuxt-only rules
 * stay enabled. A preset from `vize.config.json` is therefore expanded into
 * explicit rule entries instead of replacing that runtime preset.
 */
import { readFile } from "node:fs/promises";
import path from "node:path";

import type { NuxtLintSeverity } from "@vizejs/nuxt-lint-config";
import type { VizeRuleConfigPreset } from "oxlint-plugin-vize";

export type ProjectLintRules = Record<string, NuxtLintSeverity>;

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

type ConfigPreset = (typeof CONFIG_PRESETS)[number];

interface ProjectLinterConfig {
  preset?: ConfigPreset;
  typeAware: boolean;
  rules: ProjectLintRules;
}

export async function readProjectLintRules(
  rootDir: string,
  loadPreset: ProjectPresetLoader = loadPresetRules,
): Promise<ProjectLintRules | undefined> {
  const linter = await readVizeConfigLinter(rootDir);
  if (!linter) return undefined;

  const presetRules =
    linter.preset && linter.preset !== "incremental"
      ? await loadPreset(linter.preset, linter.typeAware)
      : {};
  const rules = { ...presetRules, ...linter.rules };
  return Object.keys(rules).length > 0 ? rules : undefined;
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

async function readVizeConfigLinter(rootDir: string): Promise<ProjectLinterConfig | undefined> {
  const file = path.join(rootDir, "vize.config.json");
  let text: string;
  try {
    text = await readFile(file, "utf8");
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code === "ENOENT") return undefined;
    throw error;
  }

  const parsed = JSON.parse(text) as { linter?: unknown };
  if (!parsed.linter || typeof parsed.linter !== "object") return undefined;
  const linter = parsed.linter as { preset?: unknown; rules?: unknown; typeAware?: unknown };
  return {
    preset: isConfigPreset(linter.preset) ? linter.preset : undefined,
    typeAware: linter.typeAware === true,
    rules: explicitRules(linter.rules),
  };
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

function isConfigPreset(value: unknown): value is ConfigPreset {
  return typeof value === "string" && (CONFIG_PRESETS as readonly string[]).includes(value);
}

function isSeverity(value: unknown): value is NuxtLintSeverity {
  return value === "off" || value === "warn" || value === "error";
}
