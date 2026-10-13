// Output and process helpers for the release and publishing commands. Writes
// are synchronous so text reaches inherited streams before a later child
// process starts, and child statuses follow the original Linux runtime.
import { spawn } from "node:child_process";
import fs from "node:fs";
import { constants as osConstants } from "node:os";

// C `strerror` texts for the errors a file write can report. MoonBit's IO
// errors carried these texts, while Node reports libuv's lowercase variants.
const STRERROR: Readonly<Record<string, string>> = {
  EACCES: "Permission denied",
  EBUSY: "Device or resource busy",
  EDQUOT: "Disk quota exceeded",
  EEXIST: "File exists",
  EFBIG: "File too large",
  EINVAL: "Invalid argument",
  EIO: "Input/output error",
  EISDIR: "Is a directory",
  ELOOP: "Too many levels of symbolic links",
  EMFILE: "Too many open files",
  ENAMETOOLONG: "File name too long",
  ENOENT: "No such file or directory",
  ENOSPC: "No space left on device",
  ENOTDIR: "Not a directory",
  EPERM: "Operation not permitted",
  EROFS: "Read-only file system",
  ETXTBSY: "Text file busy",
};

/** Describes a filesystem failure with the C `strerror` text MoonBit reported. */
export function ioErrorMessage(error: unknown): string {
  if (error instanceof Error && "code" in error && typeof error.code === "string") {
    const known = STRERROR[error.code];
    if (known !== undefined) return known;
    // Node formats `CODE: description, syscall 'path'`; keep the description.
    const text = error.message.startsWith(`${error.code}: `)
      ? error.message.slice(error.code.length + 2)
      : error.message;
    const comma = text.indexOf(", ");
    const description = comma === -1 ? text : text.slice(0, comma);
    if (description.length > 0) return description[0].toUpperCase() + description.slice(1);
  }
  return error instanceof Error ? error.message : String(error);
}

function writeAll(fd: number, text: string): void {
  const bytes = Buffer.from(text, "utf8");
  let offset = 0;
  while (offset < bytes.length) {
    try {
      offset += fs.writeSync(fd, bytes, offset, bytes.length - offset);
    } catch (error) {
      if (error instanceof Error && "code" in error && error.code === "EAGAIN") {
        Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, 1);
        continue;
      }
      throw error;
    }
  }
}

/** Writes text to standard output before any later child process starts. */
export function writeStdout(text: string): void {
  writeAll(1, text);
}

/** Writes text to standard error before any later child process starts. */
export function writeStderr(text: string): void {
  writeAll(2, text);
}

/** Writes one line to standard output. */
export function println(text: string): void {
  writeStdout(`${text}\n`);
}

/**
 * Waits for the given number of seconds. The millisecond product wraps like
 * the original 32-bit arithmetic, and non-positive waits return promptly.
 */
export function sleepSeconds(seconds: number): Promise<void> {
  const milliseconds = Math.max(0, Math.imul(seconds, 1000));
  return new Promise((resolve) => setTimeout(resolve, milliseconds));
}

/**
 * Maps a finished child to the status the MoonBit runtime reported on Linux:
 * the exit status, or the terminating signal number.
 */
function exitStatus(code: number | null, signal: NodeJS.Signals | null): number {
  if (code !== null) return code;
  if (signal !== null) return osConstants.signals[signal];
  return 1;
}

export interface SpawnOptions {
  cwd?: string;
}

/** Runs a command with inherited standard streams and returns its status. */
export function run(command: string, args: readonly string[], options: SpawnOptions = {}) {
  return new Promise<number>((resolve, reject) => {
    const child = spawn(command, args, { cwd: options.cwd, stdio: "inherit" });
    child.once("error", reject);
    child.once("close", (code, signal) => resolve(exitStatus(code, signal)));
  });
}

export interface CollectedOutput {
  status: number;
  stdout: string;
  stderr: string;
}

function collect(
  command: string,
  args: readonly string[],
  options: SpawnOptions,
  stderr: "pipe" | "inherit",
) {
  return new Promise<CollectedOutput>((resolve, reject) => {
    const child = spawn(command, args, { cwd: options.cwd, stdio: ["inherit", "pipe", stderr] });
    const stdoutChunks: Buffer[] = [];
    const stderrChunks: Buffer[] = [];
    child.stdout?.on("data", (chunk: Buffer) => stdoutChunks.push(chunk));
    child.stderr?.on("data", (chunk: Buffer) => stderrChunks.push(chunk));
    child.once("error", reject);
    child.once("close", (code, signal) =>
      resolve({
        status: exitStatus(code, signal),
        stdout: Buffer.concat(stdoutChunks).toString("utf8"),
        stderr: Buffer.concat(stderrChunks).toString("utf8"),
      }),
    );
  });
}

/** Runs a command, collecting its standard output and standard error. */
export function collectOutput(
  command: string,
  args: readonly string[],
  options: SpawnOptions = {},
): Promise<CollectedOutput> {
  return collect(command, args, options, "pipe");
}

/** Runs a command, collecting only its standard output. */
export function collectStdout(
  command: string,
  args: readonly string[],
  options: SpawnOptions = {},
): Promise<CollectedOutput> {
  return collect(command, args, options, "inherit");
}

/** Returns whether an error came from starting a child process. */
export function isSpawnError(error: unknown): error is NodeJS.ErrnoException {
  return (
    error instanceof Error &&
    "syscall" in error &&
    typeof error.syscall === "string" &&
    error.syscall.startsWith("spawn")
  );
}

/** Describes a failed child start with the original runtime's error text. */
export function spawnErrorText(error: unknown): string {
  return `OSError(${JSON.stringify(`@process.run(): ${ioErrorMessage(error)}`)})`;
}

/**
 * Runs a command entry point and exits with its status. An uncaught failure
 * exits with status 1; a failed child start prints the original runtime's
 * error text on standard output, and other failures print their stack.
 */
export function runMain(main: () => Promise<number>): void {
  main().then(
    (status) => {
      process.exitCode = status;
    },
    (error: unknown) => {
      if (isSpawnError(error)) {
        writeStdout(`${spawnErrorText(error)}\n`);
      } else {
        writeStderr(`${error instanceof Error ? (error.stack ?? error.message) : String(error)}\n`);
      }
      process.exitCode = 1;
    },
  );
}
