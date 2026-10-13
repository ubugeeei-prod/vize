/**
 * Regenerates expected compiler outputs from Vue's official compiler.
 *
 * Usage: node tools/support/tasks/generate-expected.ts [fixture | --mode <mode>]
 *
 * Every argument is forwarded to `tests/tooling/support/generate-expected.ts`.
 */
import { commandArguments, isEntrypoint, runCommand, runEntrypoint } from "../command-runtime.ts";

export function main(args: readonly string[]): number {
  return runCommand("node", ["tests/tooling/support/generate-expected.ts", ...args]);
}

if (isEntrypoint(import.meta.url)) runEntrypoint(() => main(commandArguments()));
