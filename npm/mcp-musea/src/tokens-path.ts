/**
 * Resolve a design-token file or directory that stays inside the project root.
 */

import fs from "node:fs";
import { resolveProjectPath } from "./musea.js";

const AUTO_DETECT_TOKEN_DIRS = ["tokens", "design-tokens", "style-dictionary"] as const;

/**
 * Return an explicit `tokensPath` or the first auto-detected candidate that
 * realpath-resolves inside `projectRoot`. Symlinks that leave the project
 * are rejected by `resolveProjectPath`.
 */
export async function resolveConfiguredTokensPath(
  projectRoot: string,
  tokensPath?: string,
): Promise<string | null> {
  if (tokensPath) return resolveProjectPath(projectRoot, tokensPath, "tokensPath");

  for (const dir of AUTO_DETECT_TOKEN_DIRS) {
    try {
      const candidate = resolveProjectPath(projectRoot, dir, "tokensPath");
      const stat = await fs.promises.stat(candidate);
      if (stat.isDirectory() || stat.isFile()) return candidate;
    } catch {
      // missing, or a path that escapes the project root
    }
  }
  return null;
}
