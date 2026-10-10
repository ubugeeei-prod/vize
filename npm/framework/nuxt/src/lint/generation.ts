/** Nuxt module wiring for generated oxlint configuration. */
import { createRequire } from "node:module";
import { lstat, mkdir, open, readFile, rename, unlink, writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { randomUUID } from "node:crypto";

import {
  buildNuxtLintPlan,
  collectNuxtLintDirs,
  resolveNuxtLintFeatures,
  type NuxtLintConfigItem,
} from "@vizejs/nuxt-lint-config";

import type { VizeNuxtLintOptions } from "../options.ts";
import { getDetectedNuxtMajor } from "../builder.ts";
import { setupNuxtLintConfigAddons, type NuxtLintConfigAddonNuxt } from "./addons.ts";
import { renderNuxtOxlintConfig } from "./emitter.ts";
import { toNuxtLintProjectState, type NuxtLintSourceOptions } from "./nuxt-state.ts";
import { readProjectLintRules, type ProjectLintRules } from "./project-rules.ts";
import { writeOwnedNuxtConfig } from "./owned-config.ts";

const GENERATED_CONFIG_NAME = ".oxlint.vize.json";

/** Root config names supported by oxlint's config discovery. */
export const ROOT_OXLINT_CONFIG_NAMES = [
  ".oxlintrc.json",
  ".oxlintrc.jsonc",
  "oxlint.config.ts",
  "oxlint.config.mts",
] as const;

// Existing Vite configurations own their toolchain setup, including computed
// Vite+ lint blocks. Do not evaluate them or create a competing root config.
const ROOT_LINT_CONFIG_NAMES = [
  ...ROOT_OXLINT_CONFIG_NAMES,
  "vite.config.js",
  "vite.config.mjs",
  "vite.config.ts",
  "vite.config.cjs",
  "vite.config.mts",
  "vite.config.cts",
] as const;

type Awaitable<T> = T | Promise<T>;

interface NuxtLintGenerationNuxt {
  options: { buildDir: string; rootDir: string; [key: string]: unknown };
  hook(name: string, callback: (...args: unknown[]) => unknown): void;
}

export interface NuxtLintGenerationDependencies {
  /** Phase #3614 supplies fresh addon items through this seam. */
  resolveAddons?: () => Awaitable<readonly NuxtLintConfigItem[]>;
  hasTypeScript?: (rootDir: string) => boolean;
  resolvePluginSpecifier?: (configDir: string) => string;
  /** Shared project rules. Tests inject this so generation never loads the native preset binding. */
  resolveProjectLintRules?: (rootDir: string) => Awaitable<ProjectLintRules | undefined>;
}

export interface NuxtLintConfigGeneration {
  configFile: string;
  root: string;
  regenerate(): Promise<boolean>;
  resolvePlan(fresh?: boolean): Promise<readonly NuxtLintConfigItem[]>;
}

export { setupLintInspector } from "./inspector.ts";

function isNotFound(error: unknown): boolean {
  return (error as NodeJS.ErrnoException).code === "ENOENT";
}

/**
 * Write bytes only when they differ from the current regular file.
 *
 * The length fast path mirrors Canon's materializer. A full byte comparison is
 * still required before skipping so equal-size edits are never mistaken for a
 * cache hit. Symlinks are rejected instead of followed.
 */
export async function writeFileIfChanged(file: string, content: string): Promise<boolean> {
  await mkdir(path.dirname(file), { recursive: true });
  try {
    const metadata = await lstat(file);
    if (!metadata.isFile() || metadata.isSymbolicLink()) {
      throw new Error(`Generated oxlint config must be a regular file: ${file}`);
    }
    if (metadata.size === Buffer.byteLength(content)) {
      const existing = await readFile(file);
      if (existing.equals(Buffer.from(content))) return false;
    }
  } catch (error) {
    if (!isNotFound(error)) throw error;
  }

  const temporary = `${file}.${process.pid}.${randomUUID()}.tmp`;
  try {
    await writeFile(temporary, content, { flag: "wx" });
    await rename(temporary, file);
  } finally {
    await unlink(temporary).catch((error: unknown) => {
      if (!isNotFound(error)) throw error;
    });
  }
  return true;
}

function hasTypeScript(rootDir: string): boolean {
  try {
    createRequire(path.join(rootDir, "package.json")).resolve("typescript");
    return true;
  } catch {
    return false;
  }
}

function relativeSpecifier(from: string, target: string): string {
  const relative = path.relative(from, target);
  if (path.isAbsolute(relative)) return pathToFileURL(target).href;
  const normalized = relative.split(path.sep).join("/");
  return normalized.startsWith("./") || normalized.startsWith("../")
    ? normalized
    : `./${normalized}`;
}

function resolveVizePluginSpecifier(configDir: string): string {
  const entry = fileURLToPath(import.meta.resolve("oxlint-plugin-vize"));
  return relativeSpecifier(configDir, entry);
}

async function findRootLintConfig(rootDir: string): Promise<string | undefined> {
  let directory = path.resolve(rootDir);
  while (true) {
    for (const name of ROOT_LINT_CONFIG_NAMES) {
      const candidate = path.join(directory, name);
      try {
        const metadata = await lstat(candidate);
        if (metadata.isFile() || metadata.isSymbolicLink()) return candidate;
      } catch (error) {
        if (!isNotFound(error)) throw error;
      }
    }

    const parent = path.dirname(directory);
    if (parent === directory) return undefined;
    directory = parent;
  }
}

function renderRootOxlintConfig(rootDir: string, generatedConfig: string): string {
  const specifier = relativeSpecifier(rootDir, generatedConfig);
  return [
    "// Generated by @vizejs/nuxt.",
    'import { readFileSync } from "node:fs";',
    "",
    "type OxlintJsPlugin = string | { name?: string; specifier: string };",
    "",
    `const generatedUrl = new URL(${JSON.stringify(specifier)}, import.meta.url);`,
    'const config = JSON.parse(readFileSync(generatedUrl, "utf8"));',
    "config.jsPlugins = config.jsPlugins.map((plugin: OxlintJsPlugin) =>",
    '  typeof plugin === "string"',
    "    ? new URL(plugin, generatedUrl).href",
    "    : { ...plugin, specifier: new URL(plugin.specifier, generatedUrl).href },",
    ");",
    "",
    "export default config;",
    "",
  ].join("\n");
}

async function initRootOxlintConfig(
  rootDir: string,
  generatedConfig: string,
  previousDefault?: string,
): Promise<void> {
  const existing = await findRootLintConfig(rootDir);
  if (existing) {
    // Only the exact former generated loader can migrate automatically.
    if (
      existing === path.join(rootDir, "oxlint.config.mts") &&
      previousDefault &&
      (await lstat(existing)).isFile() &&
      (await readFile(existing, "utf8")) === renderRootOxlintConfig(rootDir, previousDefault)
    ) {
      await writeFileIfChanged(existing, renderRootOxlintConfig(rootDir, generatedConfig));
    }
    return;
  }

  if (path.dirname(generatedConfig) !== rootDir) {
    throw new Error("Set lint.autoInit to false when lint.configFile is outside the Nuxt root");
  }

  const target = path.join(rootDir, "oxlint.config.mts");
  const handle = await open(target, "wx").catch((error: unknown) => {
    if ((error as NodeJS.ErrnoException).code === "EEXIST") return undefined;
    throw error;
  });
  if (!handle) return;
  try {
    await handle.writeFile(renderRootOxlintConfig(rootDir, generatedConfig), "utf8");
  } finally {
    await handle.close();
  }
}

/**
 * Generate the initial config and register the Nuxt regeneration hook.
 *
 * Addons are resolved inside `regenerate`, not at setup time, so Nuxt's import
 * registry and third-party module hooks can change between generateApp passes.
 */
export async function setupNuxtLintConfigGeneration(
  lint: boolean | VizeNuxtLintOptions | undefined,
  nuxt: NuxtLintGenerationNuxt,
  dependencies: NuxtLintGenerationDependencies = {},
): Promise<NuxtLintConfigGeneration | undefined> {
  if (lint === false) return undefined;

  const config = typeof lint === "object" && lint !== null ? lint : {};
  const {
    autoInit = true,
    configFile: configuredFile,
    rootDir: configuredRoot,
    ...featureOptions
  } = config;
  const nuxtRoot = path.resolve(nuxt.options.rootDir);
  const planRoot = configuredRoot ? path.resolve(nuxtRoot, configuredRoot) : nuxtRoot;
  const configFile = path.resolve(nuxtRoot, configuredFile ?? GENERATED_CONFIG_NAME);
  const ownsReservedConfig = configFile === path.join(nuxtRoot, GENERATED_CONFIG_NAME);
  const hasTypeScriptProbe = dependencies.hasTypeScript ?? hasTypeScript;
  const resolvePluginSpecifier = dependencies.resolvePluginSpecifier ?? resolveVizePluginSpecifier;
  const resolveAddons =
    dependencies.resolveAddons ??
    ("callHook" in nuxt
      ? setupNuxtLintConfigAddons(nuxt as NuxtLintGenerationNuxt & NuxtLintConfigAddonNuxt)
      : undefined);
  const resolveProjectRules = dependencies.resolveProjectLintRules ?? readProjectLintRules;

  let currentPlan: readonly NuxtLintConfigItem[] = [];

  const regenerate = async (): Promise<boolean> => {
    const features = resolveNuxtLintFeatures(featureOptions, () => hasTypeScriptProbe(planRoot));
    const project = toNuxtLintProjectState(nuxt.options as NuxtLintSourceOptions, {
      rootDir: planRoot,
    });
    const plan = buildNuxtLintPlan(
      features,
      collectNuxtLintDirs(project),
      getDetectedNuxtMajor(nuxt) ?? 3,
    );
    const addons = (await resolveAddons?.()) ?? [];
    const projectRules = await resolveProjectRules(planRoot);
    const projectItems =
      projectRules && Object.keys(projectRules).length > 0
        ? [{ name: "project/preset", rules: projectRules }]
        : [];
    const nextPlan = [...projectItems, ...plan, ...addons];
    const artifact = renderNuxtOxlintConfig(
      nextPlan,
      resolvePluginSpecifier(path.dirname(configFile)),
      {
        rootDir: planRoot,
        configDir: path.dirname(configFile),
        owner: ownsReservedConfig ? "@vizejs/nuxt" : undefined,
      },
    );
    const changed = ownsReservedConfig
      ? await writeOwnedNuxtConfig(configFile, artifact, writeFileIfChanged)
      : await writeFileIfChanged(configFile, artifact);
    currentPlan = nextPlan;
    return changed;
  };

  const resolvePlan = async (fresh = false): Promise<readonly NuxtLintConfigItem[]> => {
    if (fresh) await regenerate();
    return currentPlan;
  };

  await regenerate();
  nuxt.hook("builder:generateApp", regenerate);
  if (getDetectedNuxtMajor(nuxt) === 2) {
    // Nuxt 2 clears its build directory after module setup. Regenerate once
    // template generation begins so the root loader never points to ENOENT.
    nuxt.hook("build:templates", regenerate);
  }
  if (autoInit)
    await initRootOxlintConfig(
      nuxtRoot,
      configFile,
      ownsReservedConfig
        ? path.resolve(nuxtRoot, nuxt.options.buildDir, "oxlint.config.json")
        : undefined,
    );

  return { configFile, root: planRoot, regenerate, resolvePlan };
}
