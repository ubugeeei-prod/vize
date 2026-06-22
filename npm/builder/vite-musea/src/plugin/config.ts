import type { ResolvedConfig } from "vite";
import { VIZE_CONFIG_FILE_ENV, loadConfig, vizeConfigStore } from "@vizejs/vite-plugin";

export async function resolveMuseaSharedConfig(resolvedConfig: ResolvedConfig) {
  const sharedConfig = vizeConfigStore.get(resolvedConfig.root);
  if (sharedConfig) {
    return sharedConfig;
  }

  const configFile = process.env[VIZE_CONFIG_FILE_ENV];
  if (!configFile) {
    return null;
  }

  try {
    return await loadConfig(resolvedConfig.root, {
      configFile,
      env: {
        mode: resolvedConfig.mode,
        command: resolvedConfig.command === "build" ? "build" : "serve",
        isSsrBuild: !!resolvedConfig.build?.ssr,
      },
    });
  } catch (error) {
    throw new Error(
      `[musea] Failed to load Vize config from ${configFile}: ${
        error instanceof Error ? error.message : String(error)
      }`,
      { cause: error },
    );
  }
}
