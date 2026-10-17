import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

export function shellQuote(value: string): string {
  return value !== "" && /^[A-Za-z0-9._/:@%+=,-]+$/.test(value)
    ? value
    : `'${value.replaceAll("'", "'\\''")}'`;
}
// Windows npm/cmd-shims need cmd.exe; pass escaped arguments rather than
// shell:true, which joins paths containing spaces or metacharacters verbatim.
function windowsInvocation(command: string, args: readonly string[]) {
  if (process.platform !== "win32") return { command, args: [...args], verbatim: false };
  let executable = command;
  if (!path.extname(executable)) {
    const directories =
      command.includes("/") || command.includes("\\")
        ? [""]
        : (process.env.PATH ?? "").split(path.delimiter);
    const extensions = (process.env.PATHEXT ?? ".COM;.EXE;.BAT;.CMD").split(";");
    outer: for (const directory of directories)
      for (const extension of extensions) {
        const candidate = path.join(directory, command + extension.toLowerCase());
        if (isFile(candidate)) {
          executable = candidate;
          break outer;
        }
      }
  }
  if (!/\.(cmd|bat)$/i.test(executable))
    return { command: executable, args: [...args], verbatim: false };
  const doubleEscape = /node_modules[\\/]\.bin[\\/]/i.test(executable);
  const escaped = args.map((arg) => escapeWindowsArgument(arg, doubleEscape));
  const line = [path.normalize(executable).replace(WINDOWS_META, "^$1"), ...escaped].join(" ");
  return {
    command: process.env.ComSpec ?? "cmd.exe",
    args: ["/d", "/s", "/c", `"${line}"`],
    verbatim: true,
  };
}
/* Argument escaping adapted from cross-spawn (MIT), Copyright (c) 2018 Made With
 * MOXY Lda <hello@moxy.studio>. Permission is hereby granted, free of charge, to
 * any person obtaining a copy of this software and associated documentation
 * files (the "Software"), to deal in the Software without restriction, including
 * without limitation the rights to use, copy, modify, merge, publish, distribute,
 * sublicense, and/or sell copies of the Software, and to permit persons to whom
 * the Software is furnished to do so, subject to the following conditions:
 * The above copyright notice and this permission notice shall be included in
 * all copies or substantial portions of the Software. THE SOFTWARE IS PROVIDED
 * "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT
 * LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR
 * PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT
 * HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN
 * ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION
 * WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
 */
const WINDOWS_META = /([()\][%!^"`<>&|;, *?])/g;
function escapeWindowsArgument(value: string, twice: boolean): string {
  const quoted = `"${value.replace(/(?=(\\+?)?)\1"/g, '$1$1\\"').replace(/(?=(\\+?)?)\1$/, "$1$1")}"`;
  const escaped = quoted.replace(WINDOWS_META, "^$1");
  return twice ? escaped.replace(WINDOWS_META, "^$1") : escaped;
}
export function runCommandAllowFailure(command: string, args: readonly string[], cwd: string) {
  const invocation = windowsInvocation(command, args);
  const result = spawnSync(invocation.command, invocation.args, {
    cwd,
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
    maxBuffer: Infinity,
    windowsVerbatimArguments: invocation.verbatim,
  });
  if (result.error) throw new Error(`failed to run ${command}: ${result.error.message}`);
  return { status: result.status ?? 1, stdout: result.stdout, stderr: result.stderr };
}
export function runCommand(command: string, args: readonly string[], cwd: string): string {
  const output = runCommandAllowFailure(command, args, cwd);
  if (output.status !== 0) {
    const detail = `${output.stdout}${output.stderr}`.trim();
    const commandLine = [command, ...args.map(shellQuote)].join(" ");
    throw new Error(
      `${commandLine} failed with exit ${output.status}${detail ? `\n${detail}` : ""}`.trim(),
    );
  }
  return output.stdout;
}
export function sortedJson(value: unknown): unknown {
  if (Array.isArray(value)) return value.map(sortedJson);
  if (value !== null && typeof value === "object") {
    return Object.fromEntries(
      Object.entries(value)
        .sort(([a], [b]) => (a < b ? -1 : a > b ? 1 : 0))
        .map(([key, item]) => [key, sortedJson(item)]),
    );
  }
  return value;
}
export function writeJsonPretty(filename: string, value: unknown) {
  fs.mkdirSync(path.dirname(filename), { recursive: true });
  fs.writeFileSync(filename, `${JSON.stringify(sortedJson(value), null, 2)}\n`);
}
export function assertExecutable(filename: string, message: string) {
  if (process.platform !== "win32" && (fs.statSync(filename).mode & 0o111) === 0)
    throw new Error(message);
}
export function copyDir(source: string, target: string) {
  fs.mkdirSync(target, { recursive: true });
  for (const entry of fs.readdirSync(source)) {
    const from = path.join(source, entry),
      to = path.join(target, entry);
    if (fs.statSync(from).isDirectory()) copyDir(from, to);
    else fs.copyFileSync(from, to);
  }
}
export function isFile(filename: string): boolean {
  try {
    return fs.statSync(filename).isFile();
  } catch {
    return false;
  }
}
export function repoRoot(): string {
  const valid = (directory: string) =>
    isFile(path.join(directory, "Cargo.toml")) &&
    isFile(path.join(directory, "pnpm-workspace.yaml"));
  const configured = process.env.VIZE_REPO_ROOT;
  if (configured !== undefined && valid(configured)) return fs.realpathSync.native(configured);
  const candidates =
    configured === undefined
      ? [
          process.env.RUST_SCRIPT_PATH,
          process.env.RUST_SCRIPT_BASE_PATH,
          process.cwd(),
          fileURLToPath(import.meta.url),
        ]
      : [fileURLToPath(import.meta.url)];
  for (const candidate of candidates) {
    if (!candidate) continue;
    let directory = isFile(candidate) ? path.dirname(candidate) : path.resolve(candidate);
    for (;;) {
      if (valid(directory)) return fs.realpathSync.native(directory);
      const parent = path.dirname(directory);
      if (parent === directory) break;
      directory = parent;
    }
  }
  throw new Error("cannot resolve Vize repository root from script path");
}
