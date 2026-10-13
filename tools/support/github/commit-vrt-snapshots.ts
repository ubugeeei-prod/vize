/**
 * Commits updated playground VRT snapshots and pushes them to a branch.
 *
 * Usage: node tools/support/github/commit-vrt-snapshots.ts <branch>
 */
import {
  commandArguments,
  isEntrypoint,
  printLine,
  runCommand,
  runEntrypoint,
} from "../command-runtime.ts";

const usage = "Usage: node tools/support/github/commit-vrt-snapshots.ts <branch>";

export function main(args: readonly string[]): number {
  if (args.length !== 1) {
    printLine(usage);
    return 1;
  }
  const branch = args[0]!;
  if (runCommand("git", ["config", "user.name", "github-actions[bot]"]) !== 0) return 1;
  if (
    runCommand("git", ["config", "user.email", "github-actions[bot]@users.noreply.github.com"]) !==
    0
  ) {
    return 1;
  }
  if (runCommand("git", ["add", "playground/e2e/vrt/__snapshots__/"]) !== 0) return 1;
  const diffExit = runCommand("git", ["diff", "--cached", "--quiet"]);
  if (diffExit === 0) {
    printLine("No snapshot changes");
    return 0;
  }
  if (diffExit !== 1) return diffExit;
  if (runCommand("git", ["commit", "-m", "chore: update VRT snapshots"]) !== 0) return 1;
  return runCommand("git", ["push", "origin", `HEAD:${branch}`]);
}

if (isEntrypoint(import.meta.url)) runEntrypoint(() => main(commandArguments()));
