import fs from "node:fs";
import path from "node:path";

/**
 * Recreates an Oxlint fixture project as an isolated git repository root.
 *
 * These fixtures live under the workspace's `target/` directory, which the
 * repository `.gitignore` excludes. Oxlint resolves the enclosing git
 * repository at walk time and honours its ignore rules, so without a boundary
 * every fixture file is filtered out and the run reports "No files found to
 * lint" instead of the diagnostics under test. `--no-ignore` does not help: it
 * only disables `.eslintignore` sources.
 *
 * An empty `.git` directory is enough to make the walker treat the fixture as
 * its own repository, which is also what a real consumer project looks like.
 */
export function resetFixtureDir(fixtureDir: string): void {
  fs.rmSync(fixtureDir, { force: true, recursive: true });
  fs.mkdirSync(path.join(fixtureDir, ".git"), { recursive: true });
}
