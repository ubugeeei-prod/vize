import {
  moonTrim,
  nonEmptyEnv,
  parseMoonInt,
  replaceEvery,
  replaceFirst,
} from "../command_runtime.ts";

/** Parses a positive integer from a string and falls back on invalid input. */
export function positiveIntOr(value: string, fallback: number): number {
  const parsed = parseMoonInt(value);
  return parsed !== null && parsed > 0 ? parsed : fallback;
}

/** Reads a retry-related integer from the environment with a validated default. */
export function envPositiveInt(name: string, fallback: number): number {
  const value = nonEmptyEnv(name);
  return value === undefined ? fallback : positiveIntOr(value, fallback);
}

/**
 * Extracts the workspace package version from the root `Cargo.toml`.
 * We only need the `[workspace.package]` version line, so a lightweight line
 * scanner is more predictable than shelling out to another TOML parser.
 */
export function workspaceVersionFromContent(content: string): string | null {
  let section = "";
  for (const line of content.split("\n")) {
    const trimmed = moonTrim(line);
    if (trimmed.startsWith("[") && trimmed.endsWith("]")) {
      section = trimmed;
      continue;
    }
    if (
      section === "[workspace.package]" &&
      trimmed.startsWith('version = "') &&
      trimmed.endsWith('"')
    ) {
      return replaceEvery(replaceFirst(trimmed, 'version = "', ""), '"', "");
    }
  }
  return null;
}
