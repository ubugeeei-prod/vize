import fs from "node:fs";
import path from "node:path";

import { loadConfigFromFile, type PluginOption } from "vite";

import type { MuseaVrtOptions } from "../types/index.js";
import { readMuseaOptions } from "../plugin/options.js";

export async function loadMuseaVrtOptions(
  configPath: string,
  cwd = process.cwd(),
): Promise<MuseaVrtOptions | undefined> {
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

  for (const plugin of plugins) {
    const options = readMuseaOptions(plugin);
    if (options?.vrt) return options.vrt;
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
