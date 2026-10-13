/**
 * Shared process, path and file helpers for the TypeScript build, CI and
 * development commands under `tools/support/tasks` and `tools/support/github`.
 *
 * These commands replaced MoonBit command packages. The helpers keep the
 * observable contracts those packages had: child processes inherit stdio and
 * report their exit status, filesystem probes treat IO errors as "absent",
 * path joins normalize without letting an absolute right-hand side override
 * the base, and string ordering is the shortlex order used by MoonBit's
 * `Array::sort`.
 */
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import { constants as osConstants } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

export type CommandEnvironment = Record<string, string | undefined>;

export interface CommandOptions {
  readonly cwd?: string;
  /** Complete child environment. Defaults to the current process environment. */
  readonly env?: CommandEnvironment;
}

/** Whitespace removed by MoonBit's default `String::trim`. */
const moonTrimCharacters = "\t\n\r ";

/** Trims only tab, line feed, carriage return and space from both ends. */
export function trimWhitespace(value: string): string {
  let start = 0;
  let end = value.length;
  while (start < end && moonTrimCharacters.includes(value.charAt(start))) start += 1;
  while (end > start && moonTrimCharacters.includes(value.charAt(end - 1))) end -= 1;
  return value.slice(start, end);
}

/** Orders strings by UTF-16 length first, then by UTF-16 code units. */
export function compareShortlex(left: string, right: string): number {
  if (left.length !== right.length) return left.length - right.length;
  for (let index = 0; index < left.length; index += 1) {
    const difference = left.charCodeAt(index) - right.charCodeAt(index);
    if (difference !== 0) return difference;
  }
  return 0;
}

/** Joins two fragments and normalizes the result; `/b` under `a` is `a/b`. */
export function joinPath(base: string, child: string): string {
  return path.join(base, child);
}

/** Resolves a path against the current directory without trailing separators. */
export function resolvePath(value: string): string {
  return path.resolve(value);
}

/** Returns whether a path exists, following symbolic links. */
export function pathExists(target: string): boolean {
  return fs.existsSync(target);
}

/** Returns whether a path is a directory, treating IO errors as false. */
export function isDirectory(target: string): boolean {
  try {
    return fs.statSync(target).isDirectory();
  } catch {
    return false;
  }
}

/** Returns whether a path is a regular file, treating IO errors as false. */
export function isRegularFile(target: string): boolean {
  try {
    return fs.statSync(target).isFile();
  } catch {
    return false;
  }
}

/** Reads directory entry names, returning an empty list when enumeration fails. */
export function readDirectoryNames(target: string): string[] {
  try {
    return fs.readdirSync(target);
  } catch {
    return [];
  }
}

/** Reads a UTF-8 file, returning undefined when it cannot be read. */
export function readTextFile(target: string): string | undefined {
  try {
    return fs.readFileSync(target, "utf8");
  } catch {
    return undefined;
  }
}

function errorCode(error: unknown): string | undefined {
  if (typeof error !== "object" || error === null || !("code" in error)) return undefined;
  const { code } = error;
  return typeof code === "string" ? code : undefined;
}

/**
 * Creates a directory, creating missing parents first.
 *
 * Unlike `fs.mkdirSync(..., { recursive: true })`, an already existing target
 * is an error. Only a missing parent triggers the parent walk, which splits on
 * the last separator exactly as the previous MoonBit helper did.
 */
export function createDirectory(target: string, mode: number): void {
  try {
    fs.mkdirSync(target, { mode });
  } catch (error) {
    if (errorCode(error) !== "ENOENT") throw error;
    let separator = target.lastIndexOf("/");
    if (separator < 0 && process.platform === "win32") separator = target.lastIndexOf("\\");
    if (separator < 0) throw error;
    let parent = target.slice(0, separator).replace(/\/+$/u, "");
    if (process.platform === "win32") parent = parent.replace(/\\+$/u, "");
    createDirectory(parent, mode);
    fs.mkdirSync(target, { mode });
  }
}

/**
 * Writes a whole file, creating it with `mode` or truncating an existing file,
 * and flushes the written data before closing it.
 */
export function writeFileDurably(target: string, data: string | Uint8Array, mode: number): void {
  const descriptor = fs.openSync(target, "w", mode);
  try {
    fs.writeFileSync(descriptor, data);
    fs.fdatasyncSync(descriptor);
  } finally {
    fs.closeSync(descriptor);
  }
}

/**
 * Adds variables to an inherited environment without replacing inherited values.
 *
 * The MoonBit process runtime appended these variables after the inherited
 * block. Cargo and Node read the first entry of a duplicated key, so an
 * inherited value kept precedence; this preserves what those children saw.
 */
export function addMissingVariables(
  variables: CommandEnvironment,
  env: CommandEnvironment = process.env,
): CommandEnvironment {
  return { ...variables, ...env };
}

const signalNumbers = new Map<string, number>(Object.entries(osConstants.signals));

function exitStatus(status: number | null, signal: string | null): number {
  if (status !== null) return status;
  // Linux reports a signalled child's signal number as its status.
  if (signal !== null) return signalNumbers.get(signal) ?? 1;
  return 1;
}

/** Runs a command with inherited stdio and returns its exit status. */
export function runCommand(
  command: string,
  args: readonly string[],
  options: CommandOptions = {},
): number {
  const result = spawnSync(command, [...args], {
    cwd: options.cwd,
    env: options.env ?? process.env,
    stdio: "inherit",
  });
  if (result.error !== undefined) throw result.error;
  return exitStatus(result.status, result.signal);
}

export interface CollectedOutput {
  readonly exitCode: number;
  readonly stdout: string;
}

/** Runs a command, capturing UTF-8 stdout while stdin and stderr stay inherited. */
export function collectStdout(
  command: string,
  args: readonly string[],
  options: CommandOptions = {},
): CollectedOutput {
  const result = spawnSync(command, [...args], {
    cwd: options.cwd,
    env: options.env ?? process.env,
    stdio: ["inherit", "pipe", "inherit"],
    maxBuffer: Number.POSITIVE_INFINITY,
  });
  if (result.error !== undefined) throw result.error;
  return {
    exitCode: exitStatus(result.status, result.signal),
    stdout: result.stdout.toString("utf8"),
  };
}

/** Prints one line to stdout. */
export function printLine(message: string): void {
  process.stdout.write(`${message}\n`);
}

/** Returns the command-line arguments after the script path. */
export function commandArguments(): string[] {
  return process.argv.slice(2);
}

/** Returns whether `moduleUrl` is the script Node was asked to run. */
export function isEntrypoint(moduleUrl: string): boolean {
  const entry = process.argv[1];
  return entry !== undefined && path.resolve(entry) === fileURLToPath(moduleUrl);
}

/**
 * Runs a command entrypoint and sets the process exit code from its result.
 * An unexpected error is reported on stderr and exits with status 1.
 */
export function runEntrypoint(main: () => number): void {
  try {
    process.exitCode = main();
  } catch (error) {
    process.stderr.write(`${error instanceof Error ? error.message : String(error)}\n`);
    process.exitCode = 1;
  }
}
