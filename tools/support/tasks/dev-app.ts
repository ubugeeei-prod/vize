/**
 * Starts a real-world development application.
 *
 * Usage: node tools/support/tasks/dev-app.ts [target]
 *
 * The optional target is passed to `tests/tooling/support/dev-app.ts` through
 * the `usage_target` environment variable, which owns target validation. An
 * inherited `usage_target` keeps precedence, as it did for the previous
 * MoonBit command.
 */
import {
  addMissingVariables,
  commandArguments,
  isEntrypoint,
  printLine,
  runCommand,
  runEntrypoint,
} from "../command-runtime.ts";

const usage = "Usage: node tools/support/tasks/dev-app.ts [target]";

export function main(args: readonly string[]): number {
  if (args.length > 1) {
    printLine(usage);
    return 1;
  }
  const env = args.length === 1 ? addMissingVariables({ usage_target: args[0] }) : process.env;
  return runCommand("node", ["tests/tooling/support/dev-app.ts"], { env });
}

if (isEntrypoint(import.meta.url)) runEntrypoint(() => main(commandArguments()));
