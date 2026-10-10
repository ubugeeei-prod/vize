import fs from "node:fs";
import path from "node:path";

import { loadConfigFromFile, type PluginOption, type UserConfig } from "vite";
import { loadConfig, resolveViteConfigExport } from "@vizejs/vite-plugin";

import type { MuseaOptions, MuseaVrtOptions } from "../types/index.js";
import { readMuseaOptions } from "../plugin/options.js";
import { publicBasePathFromViteBase } from "../static-base.js";

const DEFAULT_INCLUDE = ["**/*.art.vue"];
const DEFAULT_EXCLUDE = ["node_modules/**", "dist/**"];
const CONFIG_ENV = { command: "serve", mode: "development", isSsrBuild: false } as const;

interface LoadedViteConfig {
  root: string;
  configDir: string;
  config: UserConfig;
  plugins: unknown[];
}

export interface MuseaFileSet {
  root: string;
  projectRoot: string;
  include: string[];
  exclude: string[];
}

export async function loadMuseaVrtOptions(
  configPath?: string,
  cwd = process.cwd(),
): Promise<MuseaVrtOptions | undefined> {
  return (await loadMuseaConfiguration(configPath, cwd)).vrt;
}

export async function loadMuseaFileSet(
  configPath?: string,
  cwd = process.cwd(),
): Promise<MuseaFileSet> {
  return (await loadMuseaConfiguration(configPath, cwd)).fileSet;
}

export async function loadMuseaPreviewBasePath(
  configPath?: string,
  cwd = process.cwd(),
): Promise<string> {
  return (await loadMuseaConfiguration(configPath, cwd)).previewBasePath;
}

/** Resolve all CLI gallery metadata from one evaluation of the Vite config. */
export async function loadMuseaConfiguration(
  configPath?: string,
  cwd = process.cwd(),
): Promise<{
  fileSet: MuseaFileSet;
  vrt: MuseaVrtOptions | undefined;
  configDir: string;
  previewBasePath: string;
}> {
  const loaded = await loadViteConfig(configPath, cwd);
  const pluginOptions = loaded ? readPluginMuseaOptions(loaded.plugins) : undefined;
  const vizeOptions = await readSharedMuseaOptions(loaded, cwd);
  const include = pluginOptions?.include ?? vizeOptions?.include ?? DEFAULT_INCLUDE;
  const exclude = pluginOptions?.exclude ?? vizeOptions?.exclude ?? DEFAULT_EXCLUDE;
  const sharedVrt = vizeOptions?.vrt;
  const { outDir, ...sharedVrtOptions } = sharedVrt ?? {};
  const vrt = sharedVrt
    ? {
        ...sharedVrtOptions,
        ...(outDir === undefined ? {} : { snapshotDir: outDir }),
        ...pluginOptions?.vrt,
      }
    : pluginOptions?.vrt;
  const root = loaded?.root ?? cwd;
  return {
    fileSet: {
      root,
      projectRoot: path.resolve(root, pluginOptions?.projectRoot ?? "."),
      include,
      exclude,
    },
    vrt,
    configDir: loaded?.configDir ?? cwd,
    previewBasePath: publicBasePathFromViteBase(
      loaded?.config.base,
      pluginOptions?.basePath ?? vizeOptions?.basePath ?? "/__musea__",
    ),
  };
}

async function loadViteConfig(
  configPath: string | undefined,
  cwd: string,
): Promise<LoadedViteConfig | undefined> {
  const resolvedConfigPath = configPath ? path.resolve(cwd, configPath) : undefined;
  if (resolvedConfigPath && !(await fileExists(resolvedConfigPath))) return undefined;

  const loaded = await loadConfigFromFile(
    {
      command: "serve",
      mode: "development",
      isPreview: false,
      isSsrBuild: false,
    },
    resolvedConfigPath,
    cwd,
  );
  if (!loaded) return undefined;

  const plugins: unknown[] = [];
  await collectPlugins(loaded.config.plugins, plugins);
  const configDir = path.dirname(loaded.path);
  const configuredRoot = loaded.config.root;
  return {
    root: configuredRoot ? path.resolve(configDir, configuredRoot) : configDir,
    configDir,
    config: loaded.config,
    plugins,
  };
}

function readPluginMuseaOptions(plugins: unknown[]): MuseaOptions | undefined {
  for (const plugin of plugins) {
    const options = readMuseaOptions(plugin);
    if (options) return options;
  }
  return undefined;
}

async function readSharedMuseaOptions(loaded: LoadedViteConfig | undefined, cwd: string) {
  // Project the config Vite already evaluated, including custom --config paths.
  // Re-importing it could repeat plugin factories and other user side effects.
  const config = loaded
    ? ((await loadConfig(loaded.configDir, { viteConfig: false, env: CONFIG_ENV })) ??
      (await resolveViteConfigExport(loaded.config, CONFIG_ENV)))
    : await loadConfig(cwd, { mode: "auto", env: CONFIG_ENV });
  return config?.musea;
}

async function collectPlugins(
  input: PluginOption | Promise<PluginOption> | undefined,
  plugins: unknown[],
) {
  const resolved = await input;
  if (!resolved) return;

  if (Array.isArray(resolved)) {
    for (const item of resolved) {
      await collectPlugins(item, plugins);
    }
    return;
  }

  plugins.push(resolved);
}

async function fileExists(filePath: string): Promise<boolean> {
  try {
    await fs.promises.access(filePath);
    return true;
  } catch {
    return false;
  }
}
