import type { ResolvedVizeConfig } from "../types.ts";

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
