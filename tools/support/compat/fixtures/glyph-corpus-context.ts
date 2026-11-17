// Give a corpus workspace the rest of its fixture project. Linting reads
// sibling sources (imported prop types, tsconfig paths, `nuxt.config`,
// `package.json`), so a workspace holding only the formatted Vue files would
// compare its findings against a baseline that saw a different project.
import { copyFileSync, linkSync, mkdirSync, readdirSync, readlinkSync, symlinkSync } from "node:fs";
import { join } from "node:path";

/**
 * Recreate the fixture tree around the workspace's own files. Files are hard
 * links (copies where linking fails), so tools see them at workspace paths:
 * a symlinked `tsconfig.json` resolves to the fixture directory, and its
 * `include` and `paths` then miss every workspace file. Nothing writes the
 * tree after this runs, and removing the workspace drops only the links.
 */
export function mirrorProjectContext(
  fixtureDir: string,
  workspaceDir: string,
  files: readonly string[],
): void {
  const owned = new Set(files);
  const walk = (relative: string): void => {
    for (const entry of readdirSync(join(fixtureDir, relative), { withFileTypes: true })) {
      if (entry.name === ".git") continue;
      const child = relative === "" ? entry.name : `${relative}/${entry.name}`;
      if (owned.has(child)) continue;
      const source = join(fixtureDir, child);
      const target = join(workspaceDir, child);
      if (entry.isDirectory()) {
        mkdirSync(target, { recursive: true });
        walk(child);
      } else if (entry.isSymbolicLink()) {
        symlinkSync(readlinkSync(source), target);
      } else if (entry.isFile()) {
        try {
          linkSync(source, target);
        } catch {
          copyFileSync(source, target);
        }
      }
    }
  };
  walk("");
}
