import fs from "node:fs";
import path from "node:path";

import { loadConfigFromFile, type PluginOption } from "vite";

import type { MuseaOptions, MuseaVrtOptions } from "../types/index.js";
import { readMuseaOptions } from "../plugin/options.js";

const DEFAULT_INCLUDE = ["**/*.art.vue"];
const DEFAULT_EXCLUDE = ["node_modules/**", "dist/**"];
const VIZE_CONFIG_NAMES = [
  "vize.config.ts",
  "vize.config.mts",
  "vize.config.js",
  "vize.config.mjs",
  "vize.config.cjs",
];

export interface MuseaFileSet {
  root: string;
  include: string[];
  exclude: string[];
}

export async function loadMuseaVrtOptions(
  configPath: string,
  cwd = process.cwd(),
): Promise<MuseaVrtOptions | undefined> {
  const loaded = await loadViteConfig(configPath, cwd);
  if (!loaded) return undefined;
  return readPluginMuseaOptions(loaded.plugins)?.vrt;
}

export async function loadMuseaFileSet(
  configPath: string,
  cwd = process.cwd(),
): Promise<MuseaFileSet> {
  const loaded = await loadViteConfig(configPath, cwd);
  const pluginOptions = loaded ? readPluginMuseaOptions(loaded.plugins) : undefined;
  const vizeOptions = pluginOptions?.include
    ? undefined
    : await readVizeMuseaOptions(loaded?.configDir ?? cwd);
  const include = pluginOptions?.include ?? vizeOptions?.include ?? DEFAULT_INCLUDE;
  const exclude = pluginOptions?.exclude ?? vizeOptions?.exclude ?? DEFAULT_EXCLUDE;
  return {
    root: loaded?.root ?? cwd,
    include,
    exclude,
  };
}

async function loadViteConfig(
  configPath: string,
  cwd: string,
): Promise<{ root: string; configDir: string; plugins: unknown[] } | undefined> {
  const resolvedConfigPath = path.isAbsolute(configPath)
    ? configPath
    : path.resolve(cwd, configPath);
  if (!(await fileExists(resolvedConfigPath))) return undefined;

  const loaded = await loadConfigFromFile(
    {
      command: "serve",
      mode: "development",
      isPreview: false,
      isSsrBuild: false,
    },
    resolvedConfigPath,
  );
  if (!loaded) return undefined;

  const plugins: unknown[] = [];
  await collectPlugins(loaded.config.plugins, plugins);
  const configDir = path.dirname(resolvedConfigPath);
  const configuredRoot = loaded.config.root;
  return {
    root: configuredRoot ? path.resolve(configDir, configuredRoot) : configDir,
    configDir,
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

async function readVizeMuseaOptions(
  dir: string,
): Promise<Pick<MuseaOptions, "include" | "exclude"> | undefined> {
  for (const name of VIZE_CONFIG_NAMES) {
    const file = path.join(dir, name);
    if (!(await fileExists(file))) continue;
    try {
      const loaded = await loadConfigFromFile(
        {
          command: "serve",
          mode: "development",
          isPreview: false,
          isSsrBuild: false,
        },
        file,
      );
      const musea = (loaded?.config as { musea?: Pick<MuseaOptions, "include" | "exclude"> }).musea;
      if (musea?.include || musea?.exclude) return musea;
    } catch {
      continue;
    }
  }
  return undefined;
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
