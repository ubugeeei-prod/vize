#!/usr/bin/env node

import { pathToFileURL } from "node:url";

import { createFrescoProject } from "./scaffold.js";
import { commandsFor } from "./templates.js";
import type { PackageManager } from "./types.js";
import { CREATE_FRESCO_VERSION } from "./version.js";

export { CREATE_FRESCO_VERSION } from "./version.js";

/** Resolved command-line options for the create-fresco executable. */
export interface CliOptions {
  /** Project directory, relative to the current process. @default "fresco-app" */
  directory: string;
  /** Allow replacing scaffold-owned files in a non-empty directory. @default false */
  force: boolean;
  /**
   * Package manager written into setup instructions.
   * @default auto-detected from npm_config_user_agent, otherwise "pnpm"
   */
  packageManager?: PackageManager;
  /** Print usage without creating a project. @default false */
  help: boolean;
  /** Print the installed create-fresco version without creating a project. @default false */
  version: boolean;
}

const packageManagers = new Set<PackageManager>(["npm", "pnpm", "yarn", "bun"]);

function readPackageManager(value: string | undefined): PackageManager {
  if (value && packageManagers.has(value as PackageManager)) return value as PackageManager;
  throw new Error(
    `CREATE_FRESCO_PACKAGE_MANAGER: expected npm, pnpm, yarn, or bun; received ${value ?? "nothing"}`,
  );
}

/** Parse create-fresco arguments or throw a stable `CREATE_FRESCO_*` diagnostic. */
export function parseCliArgs(args: readonly string[]): CliOptions {
  let directory: string | undefined;
  let force = false;
  let packageManager: PackageManager | undefined;
  let help = false;
  let version = false;

  for (let index = 0; index < args.length; index += 1) {
    const argument = args[index];
    if (argument === "--force") force = true;
    else if (argument === "--help" || argument === "-h") help = true;
    else if (argument === "--version" || argument === "-v") version = true;
    else if (argument === "--package-manager" || argument === "--pm") {
      packageManager = readPackageManager(args[index + 1]);
      index += 1;
    } else if (argument?.startsWith("--package-manager=") || argument?.startsWith("--pm=")) {
      packageManager = readPackageManager(argument.slice(argument.indexOf("=") + 1));
    } else if (argument?.startsWith("-")) {
      throw new Error(`CREATE_FRESCO_UNKNOWN_OPTION: ${argument}`);
    } else if (directory === undefined && argument) directory = argument;
    else if (argument) throw new Error(`CREATE_FRESCO_EXTRA_ARGUMENT: ${argument}`);
  }

  return { directory: directory ?? "fresco-app", force, packageManager, help, version };
}

/** Copyable usage text printed by `--help`. */
export const helpText = `create-fresco [directory] [options]

Options:
  --package-manager, --pm <name>  npm, pnpm, yarn, or bun
  --force                        overwrite scaffold-owned files in a non-empty directory
  --help, -h                     show this help
  --version, -v                  show the package version`;

/** Create a project from command-line arguments and print the next setup commands. */
export function runCli(
  args: readonly string[],
  output: (message: string) => void = console.log,
): void {
  const options = parseCliArgs(args);
  if (options.help) {
    output(helpText);
    return;
  }
  if (options.version) {
    output(CREATE_FRESCO_VERSION);
    return;
  }

  const result = createFrescoProject(options);
  const commands = commandsFor(result.packageManager);
  output(`Created ${result.packageName} in ${result.targetDirectory}`);
  output(`\n  cd ${options.directory}\n  ${commands.install}\n  ${commands.dev}\n`);
}

const invokedPath = process.argv[1];
if (invokedPath && import.meta.url === pathToFileURL(invokedPath).href) {
  try {
    runCli(process.argv.slice(2));
  } catch (error) {
    console.error(error instanceof Error ? error.message : error);
    process.exitCode = 1;
  }
}
