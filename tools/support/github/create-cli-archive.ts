/**
 * Archives a release CLI executable from `target/<target>/release`.
 *
 * Usage: node tools/support/github/create-cli-archive.ts
 *   <target> <archive> <executable>
 *
 * A `.zip` archive is written with `7z`; any other name uses `tar -czvf`.
 */
import {
  commandArguments,
  isEntrypoint,
  joinPath,
  printLine,
  resolvePath,
  runCommand,
  runEntrypoint,
} from "../command-runtime.ts";

const usage =
  "Usage: node tools/support/github/create-cli-archive.ts <target> <archive> <executable>";

export function releaseDir(target: string): string {
  return resolvePath(joinPath(joinPath("target", target), "release"));
}

export function main(args: readonly string[]): number {
  if (args.length !== 3) {
    printLine(usage);
    return 1;
  }
  const [target, archive, executable] = args as [string, string, string];
  const cwd = releaseDir(target);
  const archivePath = resolvePath(archive);
  if (archive.endsWith(".zip")) return runCommand("7z", ["a", archivePath, executable], { cwd });
  return runCommand("tar", ["-czvf", archivePath, executable], { cwd });
}

if (isEntrypoint(import.meta.url)) runEntrypoint(() => main(commandArguments()));
