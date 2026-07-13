import { VIZE_CONFIG_FILE_ENV, loadConfig, resolveConfigExport } from "../config.ts";
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
): Promise<ResolvedVizeConfig | null> {
  let fileConfig: ResolvedVizeConfig | null = null;
  if (options.configMode !== false) {
    const configFile = options.configFile ?? process.env[VIZE_CONFIG_FILE_ENV];
    try {
      fileConfig = await loadConfig(root, {
        mode: options.configMode ?? "root",
        configFile,
        env,
      });
      if (fileConfig) logger.log("Loaded config from vize.config file");
    } catch (error) {
      logger.warn(`Failed to load vize config from ${configFile ?? root}:`, error);
    }
  }

  let inlineConfig: ResolvedVizeConfig | null = null;
  if (options.config) {
    try {
      inlineConfig = await resolveConfigExport(options.config, env);
      logger.log("Loaded inline vize config from plugin options");
    } catch (error) {
      logger.warn("Failed to resolve inline vize config:", error);
    }
  }

  return mergeSharedConfig(fileConfig, inlineConfig);
}
