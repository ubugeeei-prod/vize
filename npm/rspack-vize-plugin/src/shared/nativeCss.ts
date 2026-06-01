export type LegacyNativeCssState = "enabled" | "disabled" | "unavailable";

type CompilerOptionsWithLegacyCss = {
  experiments?: {
    css?: unknown;
  };
};

export function getLegacyNativeCssState(
  compilerOptions: unknown,
): LegacyNativeCssState {
  const experiments = (
    compilerOptions as CompilerOptionsWithLegacyCss | undefined
  )?.experiments;
  if (!experiments || typeof experiments !== "object") {
    return "unavailable";
  }

  if (!Object.prototype.hasOwnProperty.call(experiments, "css")) {
    return "unavailable";
  }

  return experiments.css ? "enabled" : "disabled";
}

export function resolveNativeCss(
  explicitNativeCss: boolean | undefined,
  compilerOptions: unknown,
): boolean {
  if (explicitNativeCss != null) {
    return explicitNativeCss;
  }

  return getLegacyNativeCssState(compilerOptions) === "enabled";
}
