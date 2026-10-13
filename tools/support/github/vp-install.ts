/**
 * Installs workspace dependencies from the frozen lockfile.
 *
 * Usage: node tools/support/github/vp-install.ts [vp install arguments...]
 *
 * Extra arguments, such as `--filter` selections, are appended to
 * `vp install --frozen-lockfile --prefer-offline`.
 */
import { commandArguments, isEntrypoint, runCommand, runEntrypoint } from "../command-runtime.ts";

export function main(args: readonly string[]): number {
  return runCommand("vp", ["install", "--frozen-lockfile", "--prefer-offline", ...args]);
}

if (isEntrypoint(import.meta.url)) runEntrypoint(() => main(commandArguments()));
