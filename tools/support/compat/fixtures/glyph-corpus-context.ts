// Give a corpus workspace the rest of its fixture project. Linting reads
// sibling sources (imported prop types, tsconfig paths, `nuxt.config`,
// `package.json`) and walks up to ancestor `node_modules`, so both sides of a
// comparison must lint the same mirrored project at sibling paths.
import {
  copyFileSync,
  cpSync,
  linkSync,
  mkdirSync,
  mkdtempSync,
  readdirSync,
  readlinkSync,
  rmSync,
  symlinkSync,
} from "node:fs";
import { basename, dirname, join } from "node:path";

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

/**
 * Run `run` on a mirrored copy of the pristine fixture created next to
 * `siblingDir`. Linting the fixture tree in place would also resolve packages
 * from whatever encloses the checkout (the repository's own
 * `tests/node_modules`), which a workspace under `target/` never sees.
 */
export function withPristineWorkspace<Result>(
  fixtureDir: string,
  files: readonly string[],
  siblingDir: string,
  run: (workspaceDir: string) => Result,
): Result {
  const prefix = basename(siblingDir).replace(/[^-]*$/, "");
  const workspaceDir = mkdtempSync(join(dirname(siblingDir), prefix));
  try {
    for (const file of files) {
      mkdirSync(dirname(join(workspaceDir, file)), { recursive: true });
      cpSync(join(fixtureDir, file), join(workspaceDir, file));
    }
    mirrorProjectContext(fixtureDir, workspaceDir, files);
    return run(workspaceDir);
  } finally {
    rmSync(workspaceDir, { recursive: true, force: true });
  }
}
