import { windowsInvocation } from "../../command-invocation.ts";
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

export function shellQuote(value: string): string {
  return value !== "" && /^[A-Za-z0-9._/:@%+=,-]+$/.test(value)
    ? value
    : `'${value.replaceAll("'", "'\\''")}'`;
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
  const code = result.status ?? 1;
  const status = process.platform === "win32" && code >= 0x80000000 ? code - 0x100000000 : code;
  return { status, stdout: result.stdout, stderr: result.stderr };
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
  // BigInts come only from exact i64/u64 source tokens in readJson. Emit JSON
  // numbers, not quoted strings, without passing through a rounded Number.
  if (typeof value === "bigint") {
    const json = JSON as typeof JSON & { rawJSON(source: string): unknown };
    return json.rawJSON(value.toString());
  }
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
