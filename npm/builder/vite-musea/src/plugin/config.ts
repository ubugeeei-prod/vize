import type { ResolvedConfig } from "vite";
import { VIZE_CONFIG_FILE_ENV, loadConfig, resolveViteConfigExport } from "@vizejs/vite-plugin";
import { getResolvedVizeConfigRegistration } from "@vizejs/vite-plugin/internal/config-bridge";

export async function resolveMuseaSharedConfig(
  resolvedConfig: ResolvedConfig,
  loadConfigFile: typeof loadConfig = loadConfig,
  projectViteConfig: typeof resolveViteConfigExport = resolveViteConfigExport,
) {
  let registration = getResolvedVizeConfigRegistration(resolvedConfig);
  if (!registration.registered) {
    // Vite starts configResolved hooks in parallel. Yield once so Vize can
    // register the exact config even when Musea appears first in plugin order.
    await Promise.resolve();
    registration = getResolvedVizeConfigRegistration(resolvedConfig);
  }
  if (registration.registered) {
    return await registration.config;
  }

  const configFile = process.env[VIZE_CONFIG_FILE_ENV];
  try {
    const env = {
      mode: resolvedConfig.mode,
      command: resolvedConfig.command === "build" ? ("build" as const) : ("serve" as const),
      isSsrBuild: !!resolvedConfig.build?.ssr,
    };
    const dedicatedConfig = await loadConfigFile(resolvedConfig.root, {
      configFile,
      viteConfig: false,
      env,
    });
    return dedicatedConfig ?? (await projectViteConfig(resolvedConfig, env));
  } catch (error) {
    throw new Error(
      `[musea] Failed to load Vize config from ${configFile ?? resolvedConfig.root}: ${
        error instanceof Error ? error.message : String(error)
      }`,
      { cause: error },
    );
  }
}
