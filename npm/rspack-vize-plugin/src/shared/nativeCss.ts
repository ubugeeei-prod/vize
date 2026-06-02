export type LegacyNativeCssState = "enabled" | "disabled" | "unavailable";

type CompilerOptionsWithLegacyCss = {
  experiments?: {
    css?: unknown;
  };
};

export function getLegacyNativeCssState(compilerOptions: unknown): LegacyNativeCssState {
  const experiments = (compilerOptions as CompilerOptionsWithLegacyCss | undefined)?.experiments;
  if (!experiments || typeof experiments !== "object") {
    return "unavailable";
  }

  if (!Object.prototype.hasOwnProperty.call(experiments, "css")) {
    return "unavailable";
  }

  return experiments.css ? "enabled" : "disabled";
}

/** Parse the Rspack major version from a string like "2.0.3". Returns null if unparseable. */
export function getRspackMajor(rspackVersion: unknown): number | null {
  if (typeof rspackVersion !== "string") {
    return null;
  }
  const major = Number.parseInt(rspackVersion, 10);
  return Number.isNaN(major) ? null : major;
}

export function resolveNativeCss(
  explicitNativeCss: boolean | undefined,
  compilerOptions: unknown,
  rspackVersion?: unknown,
): boolean {
  // 1. Explicit `css.native` always wins.
  if (explicitNativeCss != null) {
    return explicitNativeCss;
  }

  // 2. Rspack 1.x legacy signal: honor explicit `experiments.css` (true or false).
  const legacyState = getLegacyNativeCssState(compilerOptions);
  if (legacyState !== "unavailable") {
    return legacyState === "enabled";
  }

  // 3. No `experiments.css`: native CSS is the default capability in Rspack 2.x+,
  //    but not in 1.x (where omitting `experiments.css` means non-native).
  const major = getRspackMajor(rspackVersion);
  return major != null && major >= 2;
}
