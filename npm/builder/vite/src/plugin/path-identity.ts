import fs from "node:fs";
import path from "node:path";

/**
 * Every stable filesystem identity Vite may use for one authored path.
 *
 * Vite normally resolves workspace links, while chokidar and plugins can
 * report either the logical link or its physical target. Missing watched files
 * (notably a tsconfig that is created later) still inherit the real identity
 * of their nearest existing parent.
 */
export function filesystemPathIdentities(file: string): readonly string[] {
  const resolved = path.resolve(file);
  const identities = new Set([resolved]);
  let existing = resolved;
  const missing: string[] = [];

  for (;;) {
    try {
      const physical = fs.realpathSync.native(existing);
      identities.add(path.join(physical, ...missing));
      break;
    } catch (error) {
      const code = (error as NodeJS.ErrnoException).code;
      if (code !== "ENOENT" && code !== "ENOTDIR") break;
      const parent = path.dirname(existing);
      if (parent === existing) break;
      missing.unshift(path.basename(existing));
      existing = parent;
    }
  }

  return [...identities];
}
