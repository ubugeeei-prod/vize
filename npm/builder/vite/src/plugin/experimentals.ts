import type { ResolvedVizeConfig, VizeOptions } from "../types.ts";

function isExperimentalEnabled(value: unknown): boolean {
  return value !== undefined && value !== null && value !== false;
}

export function resolveExperimentalOptions(
  options: VizeOptions["experimentals"] | undefined,
  config: ResolvedVizeConfig["experimentals"] | undefined,
) {
  const pick = (...values: unknown[]) => values.find((value) => value !== undefined);
  const enabled = (...values: unknown[]) => isExperimentalEnabled(pick(...values));

  return {
    vapor: enabled(options?.vapor, config?.vapor),
    jsxVapor: enabled(options?.jsxVapor, config?.jsxVapor),
    inTagComments: enabled(
      options?.intagComment,
      options?.inTagComment,
      config?.intagComment,
      config?.inTagComment,
    ),
    patternedTemplate: enabled(
      options?.pattenedTemplate,
      options?.patternedTemplate,
      config?.pattenedTemplate,
      config?.patternedTemplate,
    ),
    serverScript: enabled(
      options?.serverScript,
      options?.["server script"],
      config?.serverScript,
      config?.["server script"],
    ),
  };
}

export function resolveExperimentalCompilerOptions(
  options: VizeOptions,
  compilerConfig: ResolvedVizeConfig["compiler"],
  config: ResolvedVizeConfig["experimentals"] | undefined,
) {
  const experimentals = resolveExperimentalOptions(options.experimentals, config);
  return {
    vapor: options.vapor ?? compilerConfig?.vapor ?? experimentals.vapor,
    jsxMode:
      options.jsxMode ?? compilerConfig?.jsxMode ?? (experimentals.jsxVapor ? "vapor" : undefined),
    jsxCompat: options.jsxCompat ?? compilerConfig?.jsxCompat,
    experimentalInTagComments: experimentals.inTagComments,
    experimentalPatternedTemplate: experimentals.patternedTemplate,
    experimentalServerScript: experimentals.serverScript,
  };
}
