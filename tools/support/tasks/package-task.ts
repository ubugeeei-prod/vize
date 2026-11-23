import { spawnSync } from "node:child_process";
import { globSync, realpathSync } from "node:fs";
import { constants } from "node:os";
import path from "node:path";
import { windowsInvocation } from "../command-invocation.ts";

export interface PackageCommand {
  program: string;
  arguments?: readonly string[];
  directory?: string | null;
  environment?: Readonly<Record<string, string>>;
  defaults?: Readonly<Record<string, string>>;
  globs?: readonly string[];
  absolutePaths?: readonly string[];
}

function argumentValues(command: PackageCommand, cwd: string): string[] {
  return (command.arguments ?? []).flatMap((argument) => {
    if (command.absolutePaths?.includes(argument)) return [path.resolve(cwd, argument)];
    if (process.platform === "win32" || !command.globs?.includes(argument)) return [argument];
    // POSIX package scripts expand each unquoted path component independently.
    const pattern = argument
      .split("/")
      .map((part) => (part === "**" ? "*" : part))
      .join("/");
    const files = globSync(pattern, { cwd }).sort((left, right) =>
      left < right ? -1 : left > right ? 1 : 0,
    );
    return files.length ? files : [argument];
  });
}

export function runPackageTask(
  tasks: Readonly<Record<string, readonly PackageCommand[]>>,
  argv: readonly string[],
  environment: NodeJS.ProcessEnv = process.env,
): number {
  const [name, ...forwarded] = argv;
  if (!name || !Object.hasOwn(tasks, name)) throw new Error("Unknown package task: " + name);
  const commands = tasks[name]!;
  for (const [index, command] of commands.entries()) {
    const cwd = realpathSync(command.directory ?? ".");
    const env = { ...environment, ...command.environment };
    for (const [key, value] of Object.entries(command.defaults ?? {}))
      if (!env[key]) env[key] = value;
    const args = argumentValues(command, cwd);
    if (index === commands.length - 1) args.push(...forwarded);
    const invocation = windowsInvocation(command.program, args);
    const result = spawnSync(invocation.command, invocation.args, {
      cwd,
      env,
      stdio: "inherit",
      windowsVerbatimArguments: invocation.verbatim,
    });
    if (result.error) {
      console.error(result.error.message);
      return (result.error as NodeJS.ErrnoException).code === "ENOENT" ? 127 : 126;
    }
    if (result.signal) return 128 + constants.signals[result.signal];
    if (result.status !== 0) return result.status ?? 1;
  }
  return 0;
}
