// Give a corpus workspace the rest of its fixture project. Linting reads
// sibling sources (imported prop types, tsconfig paths, `nuxt.config`,
// `package.json`) and walks up to ancestor `node_modules`, so both sides of a
// comparison lint the same mirrored project, at sibling paths outside any
// checkout, where nothing above the fixture's own tree can resolve.
import {
  copyFileSync,
  cpSync,
  existsSync,
  linkSync,
  mkdirSync,
  mkdtempSync,
  readdirSync,
  readlinkSync,
  realpathSync,
  rmSync,
  symlinkSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { basename, dirname, join } from "node:path";

/** Entries whose presence above a workspace would leak into resolution. */
const ENCLOSING_CONTEXT = ["node_modules", "package.json", "tsconfig.json"];

/**
 * Throw if any ancestor of `dir` holds an entry that module or project
 * resolution would pick up, so neither side sees packages from outside the
 * fixture's own tree.
 */
export function assertNoEnclosingContext(dir: string): void {
  let current = realpathSync(dirname(dir));
  for (;;) {
    const found = ENCLOSING_CONTEXT.filter((entry) => existsSync(join(current, entry)));
    if (found.length > 0) {
      throw new Error(`corpus workspace ${dir} is enclosed by ${join(current, found[0])}`);
    }
    const parent = dirname(current);
    if (parent === current) return;
    current = parent;
  }
}

/**
 * Create a root for the comparison's workspaces outside the repository, on
 * the runner's temp volume when there is one (hard links then stay on the
 * checkout's disk), and prove nothing above it can resolve.
 */
export function createIsolatedWorkspaceRoot(): string {
  const base = process.env.RUNNER_TEMP || tmpdir();
  const root = mkdtempSync(join(base, "vize-glyph-lint-"));
  try {
    assertNoEnclosingContext(root);
  } catch (error) {
    rmSync(root, { recursive: true, force: true });
    throw error;
  }
  return root;
}

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
 * from the checkout that encloses it: after `vp install`, the repository's own
 * `tests/node_modules` provided `reka-ui` to n8n's Popover.vue.
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
