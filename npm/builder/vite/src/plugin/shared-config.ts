import { nativeProjectSource } from "./host-config-source.ts";
import {
  VIZE_CONFIG_FILE_ENV,
  loadConfig,
  resolveConfigExport,
  resolveViteConfigExport,
} from "../config.ts";
import type { createLogger } from "../transform.ts";
import type { ConfigEnv, ResolvedVizeConfig, VizeOptions } from "../types.ts";

export function mergeSharedConfig(
  baseConfig: ResolvedVizeConfig | null,
  overrideConfig: ResolvedVizeConfig | null,
): ResolvedVizeConfig | null {
  if (!baseConfig) return overrideConfig;
  if (!overrideConfig) return baseConfig;

  return {
    ...baseConfig,
    ...overrideConfig,
    compiler: {
      ...baseConfig.compiler,
      ...overrideConfig.compiler,
    },
    vite: {
      ...baseConfig.vite,
      ...overrideConfig.vite,
    },
    linter: {
      ...baseConfig.linter,
      ...overrideConfig.linter,
    },
    typeChecker: {
      ...baseConfig.typeChecker,
      ...overrideConfig.typeChecker,
    },
    formatter: {
      ...baseConfig.formatter,
      ...overrideConfig.formatter,
    },
    languageServer: {
      ...baseConfig.languageServer,
      ...overrideConfig.languageServer,
    },
    musea: {
      ...baseConfig.musea,
      ...overrideConfig.musea,
    },
    globalTypes: {
      ...baseConfig.globalTypes,
      ...overrideConfig.globalTypes,
    },
    entries: [...baseConfig.entries, ...overrideConfig.entries],
  };
}

export async function resolveSharedConfig(
  options: Pick<VizeOptions, "config" | "configFile" | "configMode">,
  root: string,
  env: ConfigEnv,
  logger: ReturnType<typeof createLogger>,
  viteConfig?: unknown,
  hostContext?: "nuxt",
): Promise<ResolvedVizeConfig | null> {
  let fileConfig: ResolvedVizeConfig | null = null;
  if (options.configMode !== false) {
    const configFile = options.configFile ?? process.env[VIZE_CONFIG_FILE_ENV];
    fileConfig = await loadConfig(root, {
      mode: options.configMode ?? "root",
      configFile,
      env,
      viteConfig: false,
    });
    if (fileConfig) logger.log("Loaded explicitly selected native Vue settings");
  }

  const inlineConfig = options.config ? await resolveConfigExport(options.config, env) : null;
  if (inlineConfig) logger.log("Loaded inline Vue settings from plugin options");

  const projectConfig =
    viteConfig === undefined || options.configMode === false
      ? null
      : await resolveViteConfigExport(nativeProjectSource(viteConfig, hostContext), env);
  return mergeSharedConfig(mergeSharedConfig(projectConfig, fileConfig), inlineConfig);
}
