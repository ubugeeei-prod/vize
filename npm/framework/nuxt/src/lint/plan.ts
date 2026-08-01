/**
 * The Nuxt-aware lint config plan.
 *
 * `@nuxt/eslint` turns a Nuxt project into an ordered list of flat-config
 * items. The Nuxt-specific ones — the blocks that know about pages, layouts,
 * components, and `nuxt.config` — are the actual porting surface; the rest is
 * `@nuxt/eslint-config`'s generic JavaScript/TypeScript/Vue preset content.
 *
 * This module produces those Nuxt-specific items as an engine-neutral plan.
 * Emitting the plan as an oxlint config (Vize's execution engine) is a separate
 * concern, so the plan can be pinned against the real package by the
 * differential oracle in `test/nuxt-eslint-compat/` without ESLint in the loop.
 */
import type { NuxtLintDirs } from "./dirs.ts";
import { type ResolvedNuxtLintFeatures, shouldSortNuxtConfigKeys } from "./features.ts";
import { posixJoin } from "./paths.ts";

/** Extensions every Nuxt-aware glob covers. */
export const NUXT_LINT_GLOB_EXTS = "{js,ts,jsx,tsx,vue}";

/** Globs matching a `nuxt.config` file in either supported location. */
export const NUXT_CONFIG_GLOBS = [
  "**/.config/nuxt.?([cm])[jt]s?(x)",
  "**/nuxt.config.?([cm])[jt]s?(x)",
] as const;

/** Directories the generated config never lints. */
export const NUXT_LINT_IGNORES = [
  "**/dist",
  "**/node_modules",
  "**/.nuxt",
  "**/.output",
  "**/.vercel",
  "**/.netlify",
  "**/public",
] as const;

/** A severity as it appears in the plan. */
export type NuxtLintSeverity = "off" | "warn" | "error";

/** One item of the generated config: a named block of rules over a glob set. */
export interface NuxtLintConfigItem {
  /** Stable identity, matching `@nuxt/eslint`'s config item names. */
  name: string;
  /** Files the block applies to. Absent means "every linted file". */
  files?: string[];
  /** Paths excluded from linting entirely. */
  ignores?: string[];
  /** Rule severities keyed by eslint-compatible rule id. */
  rules?: Record<string, NuxtLintSeverity>;
  /** Globals the block declares, so undefined-variable rules stay quiet. */
  globals?: Record<string, "readonly" | "writable">;
}

/** Runtime globals Nuxt injects into every linted file. */
export const NUXT_RUNTIME_GLOBALS: Record<string, "readonly"> = { $fetch: "readonly" };

const nestedGlob = `**/*.${NUXT_LINT_GLOB_EXTS}`;

/**
 * Files allowed to have a single-word component name.
 *
 * Nuxt gives `app` and `error` a special meaning, layouts and pages are never
 * referenced by tag name, and prefixed component directories get their name
 * from the prefix — so `vue/multi-word-component-names` is off for all of them.
 */
function routeFiles(dirs: NuxtLintDirs): string[] {
  return [
    ...new Set([
      ...dirs.src.flatMap((dir) => [
        posixJoin(dir, `app.${NUXT_LINT_GLOB_EXTS}`),
        posixJoin(dir, `error.${NUXT_LINT_GLOB_EXTS}`),
      ]),
      ...dirs.layouts.map((dir) => posixJoin(dir, nestedGlob)),
      ...dirs.pages.map((dir) => posixJoin(dir, nestedGlob)),
      // Only *nested* component files are exempt: a top-level `components/Foo.vue`
      // still has to be multi-word.
      ...dirs.components.map((dir) => posixJoin(dir, "*", nestedGlob)),
      ...dirs.componentsPrefixed.map((dir) => posixJoin(dir, nestedGlob)),
    ]),
  ].sort();
}

/** Files that must render exactly one root element. */
function singleRootFiles(dirs: NuxtLintDirs): string[] {
  return [
    ...dirs.layouts.map((dir) => posixJoin(dir, nestedGlob)),
    ...dirs.pages.map((dir) => posixJoin(dir, nestedGlob)),
    ...dirs.components.map((dir) => posixJoin(dir, `**/*.server.${NUXT_LINT_GLOB_EXTS}`)),
  ].sort();
}

/** Files where `definePageMeta` is extracted at build time. */
function pageFiles(dirs: NuxtLintDirs): string[] {
  return dirs.pages.map((dir) => posixJoin(dir, nestedGlob)).sort();
}

/**
 * Build the Nuxt-specific part of the generated lint config, in emission order.
 *
 * The order is observable — later items override earlier ones — so it is part
 * of the contract the oracle pins, not an implementation detail.
 */
export function buildNuxtLintPlan(
  features: ResolvedNuxtLintFeatures,
  dirs: NuxtLintDirs,
): NuxtLintConfigItem[] {
  const items: NuxtLintConfigItem[] = [];

  // The ignore block belongs to the baseline setup, so a project that opts out
  // of `standalone` is expected to bring its own ignores.
  if (features.standalone !== false) {
    items.push({ name: "nuxt/ignores", ignores: [...NUXT_LINT_IGNORES] });
  }

  items.push({ name: "nuxt/setup", globals: { ...NUXT_RUNTIME_GLOBALS } });

  const singleRoot = singleRootFiles(dirs);
  if (singleRoot.length > 0) {
    items.push({
      name: "nuxt/vue/single-root",
      files: singleRoot,
      rules: { "vue/no-multiple-template-root": "error" },
    });
  }

  items.push({ name: "nuxt/rules", rules: { "nuxt/prefer-import-meta": "error" } });

  const pages = pageFiles(dirs);
  if (pages.length > 0) {
    items.push({
      name: "nuxt/pages",
      files: pages,
      rules: { "nuxt/no-page-meta-runtime-values": "error" },
    });
  }

  items.push({
    name: "nuxt/nuxt-config",
    files: [...NUXT_CONFIG_GLOBS],
    rules: { "nuxt/no-nuxt-config-test-key": "error" },
  });

  if (shouldSortNuxtConfigKeys(features)) {
    items.push({
      name: "nuxt/sort-config",
      files: [...NUXT_CONFIG_GLOBS],
      rules: { "nuxt/nuxt-config-keys-order": "error" },
    });
  }

  const routes = routeFiles(dirs);
  if (routes.length > 0) {
    items.push({
      name: "nuxt/disables/routes",
      files: routes,
      rules: { "vue/multi-word-component-names": "off" },
    });
  }

  return items;
}
