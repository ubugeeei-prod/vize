import path from "node:path";
import type { ResolvedVizeConfig } from "../types.ts";

/** Keep project-relative scopes when the serialized config lives in OS temp. */
export function relocateTaskConfig(
  config: Partial<ResolvedVizeConfig>,
  root: string,
): ResolvedVizeConfig {
  const absolutePattern = (pattern: string) => {
    const negative = pattern.startsWith("!");
    const value = negative ? pattern.slice(1) : pattern;
    return `${negative ? "!" : ""}${path.resolve(root, value).replaceAll("\\", "/")}`;
  };
  return {
    ...config,
    basePath: path.resolve(root, config.basePath ?? ""),
    ignores: config.ignores?.map(absolutePattern),
    entries: (config.entries ?? []).map((entry) => ({
      ...entry,
      basePath: path.resolve(root, entry.basePath ?? ""),
    })),
  };
}
