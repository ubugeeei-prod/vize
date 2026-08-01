import { spawn } from "node:child_process";
import { existsSync } from "node:fs";
import { glob } from "node:fs/promises";
import { createRequire } from "node:module";
import path from "node:path";
import { isMainThread, parentPort, Worker } from "node:worker_threads";

export interface NuxtLintCheckerTask {
  cwd: string;
  configFile: string;
  targets: string[];
  exclude: string[];
  formatter: string;
  emitWarning: boolean;
  emitError: boolean;
  fix: boolean;
  /** Test/integration override. Normal callers resolve the project's oxlint. */
  oxlintEntrypoint?: string;
}

export interface NuxtLintCheckerResult {
  diagnosticCount: number;
  hasErrors: boolean;
  hasWarnings: boolean;
  output: string;
}

interface OxlintDiagnostic {
  code?: string;
  filePath?: string;
  filename?: string;
  labels?: Array<{ span?: { column?: number; line?: number } }>;
  message?: string;
  severity?: string | number;
}

interface OxlintPayload {
  diagnostics: OxlintDiagnostic[];
  [key: string]: unknown;
}

interface WorkerRequest {
  id: number;
  task: NuxtLintCheckerTask;
}

type WorkerResponse =
  | { id: number; result: NuxtLintCheckerResult }
  | { error: { message: string; stack?: string }; id: number };

function resolveOxlintEntrypoint(cwd: string): string {
  try {
    const manifest = createRequire(path.join(cwd, "package.json")).resolve("oxlint/package.json");
    return path.join(path.dirname(manifest), "bin", "oxlint");
  } catch (error) {
    throw new Error(
      `Unable to resolve oxlint from ${cwd}. Install oxlint before enabling the Nuxt lint checker.`,
      { cause: error },
    );
  }
}

function projectPattern(pattern: string, cwd: string): string {
  if (!path.isAbsolute(pattern)) return pattern;
  const relative = path.relative(cwd, pattern);
  if (path.isAbsolute(relative) || relative === ".." || relative.startsWith(`..${path.sep}`)) {
    return pattern;
  }
  return relative.split(path.sep).join("/") || ".";
}

function expandBraces(pattern: string): string[] {
  const match = /\{([^{}]+)\}/u.exec(pattern);
  if (!match || match.index === undefined) return [pattern];
  const before = pattern.slice(0, match.index);
  const after = pattern.slice(match.index + match[0].length);
  return match[1].split(",").flatMap((part) => expandBraces(`${before}${part}${after}`));
}

function oxlintPatterns(pattern: string, cwd: string): string[] {
  const rebased = projectPattern(pattern, cwd);
  const patterns = expandBraces(rebased).flatMap((expanded) => {
    const shallow = expanded.replace(/(^|\/)\*\*\//gu, "$1");
    return shallow === expanded ? [expanded] : [expanded, shallow];
  });
  return [...new Set(patterns)];
}

async function oxlintTargets(pattern: string, cwd: string): Promise<string[]> {
  const rebased = projectPattern(pattern, cwd);
  const defaultSuffix = "/**/*.{js,jsx,ts,tsx,vue}";
  if (rebased.endsWith(defaultSuffix)) return [rebased.slice(0, -defaultSuffix.length) || "."];
  if (!/[*?[\]{}()]/u.test(rebased)) return [rebased];
  const matches = await Array.fromAsync(glob(rebased, { cwd }));
  return matches.length > 0 ? matches : oxlintPatterns(pattern, cwd);
}

async function checkerArgs(task: NuxtLintCheckerTask): Promise<string[]> {
  const args = ["--config", task.configFile, "--format", "json", "--no-error-on-unmatched-pattern"];
  if (task.fix) args.push("--fix");
  for (const pattern of task.exclude) {
    for (const exclude of oxlintPatterns(pattern, task.cwd)) {
      args.push("--ignore-pattern", exclude);
    }
  }
  for (const target of task.targets) args.push(...(await oxlintTargets(target, task.cwd)));
  return args;
}

function spawnOxlint(
  node: string,
  entrypoint: string,
  args: string[],
  cwd: string,
): Promise<string> {
  return new Promise((resolve, reject) => {
    const child = spawn(node, [entrypoint, ...args], {
      cwd,
      stdio: ["ignore", "pipe", "pipe"],
    });
    const stdout: Buffer[] = [];
    const stderr: Buffer[] = [];
    child.stdout.on("data", (chunk: Buffer | string) => stdout.push(Buffer.from(chunk)));
    child.stderr.on("data", (chunk: Buffer | string) => stderr.push(Buffer.from(chunk)));
    child.on("error", (error) => {
      reject(
        new Error(`Failed to start oxlint at ${entrypoint}: ${error.message}`, { cause: error }),
      );
    });
    child.on("close", (status) => {
      const output = Buffer.concat(stdout).toString("utf8");
      if (output.trim()) {
        resolve(output);
        return;
      }
      const details = Buffer.concat(stderr).toString("utf8").trim();
      if (status === 0) {
        resolve('{"diagnostics":[]}\n');
        return;
      }
      reject(
        new Error(
          `oxlint exited with ${String(status)} without JSON output${details ? `: ${details}` : ""}`,
        ),
      );
    });
  });
}

async function executeOxlint(entrypoint: string, args: string[], cwd: string): Promise<string> {
  if (!existsSync(entrypoint)) {
    throw new Error(`Failed to start oxlint: entrypoint does not exist: ${entrypoint}`);
  }
  // Vite+ can launch Node through a short-lived bundled runtime path. Its
  // executable can disappear between the existence check and spawn, so retry
  // the ENOENT race through the stable PATH shim.
  const node = existsSync(process.execPath) ? process.execPath : "node";
  try {
    return await spawnOxlint(node, entrypoint, args, cwd);
  } catch (error) {
    const cause = (error as Error & { cause?: NodeJS.ErrnoException }).cause;
    if (node !== "node" && cause?.code === "ENOENT") {
      return spawnOxlint("node", entrypoint, args, cwd);
    }
    throw error;
  }
}

function diagnosticKind(diagnostic: OxlintDiagnostic): "error" | "warning" | undefined {
  if (diagnostic.severity === 2 || diagnostic.severity === "error") return "error";
  if (
    diagnostic.severity === 1 ||
    diagnostic.severity === "warn" ||
    diagnostic.severity === "warning"
  ) {
    return "warning";
  }
  return undefined;
}

function readableDiagnostics(diagnostics: OxlintDiagnostic[], formatter: string): string {
  const unix = formatter === "unix";
  const lines = diagnostics.map((diagnostic) => {
    const span = diagnostic.labels?.find((label) => label.span)?.span;
    const file = diagnostic.filename ?? diagnostic.filePath ?? "<unknown>";
    const line = span?.line ?? 1;
    const column = span?.column ?? 1;
    const severity = diagnosticKind(diagnostic) ?? "warning";
    const message = diagnostic.message ?? "lint diagnostic";
    const code = diagnostic.code ? ` (${diagnostic.code})` : "";
    return unix
      ? `${file}:${line}:${column}: ${message} [${severity}${code}]`
      : `${file}:${line}:${column}  ${severity}  ${message}${code}`;
  });
  return lines.length > 0 ? `${lines.join("\n")}\n` : "";
}

function resultFromPayload(
  payload: OxlintPayload,
  task: NuxtLintCheckerTask,
): NuxtLintCheckerResult {
  const diagnostics = payload.diagnostics.filter((diagnostic) => {
    const kind = diagnosticKind(diagnostic);
    return (kind === "error" && task.emitError) || (kind === "warning" && task.emitWarning);
  });
  const hasErrors = diagnostics.some((diagnostic) => diagnosticKind(diagnostic) === "error");
  const hasWarnings = diagnostics.some((diagnostic) => diagnosticKind(diagnostic) === "warning");
  const output =
    task.formatter === "json"
      ? diagnostics.length > 0
        ? `${JSON.stringify({ ...payload, diagnostics })}\n`
        : ""
      : readableDiagnostics(diagnostics, task.formatter);
  return { diagnosticCount: diagnostics.length, hasErrors, hasWarnings, output };
}

/** Run one checker pass. This function executes inside the long-lived worker. */
export async function runNuxtLintCheckerTask(
  task: NuxtLintCheckerTask,
): Promise<NuxtLintCheckerResult> {
  if (!task.emitError && !task.emitWarning && !task.fix) {
    return { diagnosticCount: 0, hasErrors: false, hasWarnings: false, output: "" };
  }
  const entrypoint = task.oxlintEntrypoint ?? resolveOxlintEntrypoint(task.cwd);
  const stdout = await executeOxlint(entrypoint, await checkerArgs(task), task.cwd);
  let payload: OxlintPayload;
  try {
    payload = JSON.parse(stdout) as OxlintPayload;
  } catch (error) {
    throw new Error(`oxlint returned invalid JSON: ${stdout.slice(0, 240)}`, { cause: error });
  }
  if (!Array.isArray(payload.diagnostics)) throw new Error("oxlint JSON has no diagnostics array");
  return resultFromPayload(payload, task);
}

function serializeError(error: unknown): { message: string; stack?: string } {
  if (error instanceof Error) return { message: error.message, stack: error.stack };
  return { message: String(error) };
}

if (!isMainThread && parentPort) {
  parentPort.on("message", async ({ id, task }: WorkerRequest) => {
    try {
      parentPort?.postMessage({
        id,
        result: await runNuxtLintCheckerTask(task),
      } satisfies WorkerResponse);
    } catch (error) {
      parentPort?.postMessage({ error: serializeError(error), id } satisfies WorkerResponse);
    }
  });
}

/** Persistent worker-thread client used by the Vite dev plugin. */
export class NuxtLintCheckerWorker {
  private readonly worker = new Worker(new URL(import.meta.url));
  private readonly pending = new Map<
    number,
    { reject(error: Error): void; resolve(result: NuxtLintCheckerResult): void }
  >();
  private nextId = 0;
  private closed = false;

  constructor() {
    this.worker.on("message", (response: WorkerResponse) => {
      const pending = this.pending.get(response.id);
      if (!pending) return;
      this.pending.delete(response.id);
      if ("error" in response) {
        const error = new Error(response.error.message);
        error.stack = response.error.stack ?? error.stack;
        pending.reject(error);
      } else {
        pending.resolve(response.result);
      }
    });
    this.worker.on("error", (error) => this.rejectAll(error));
    this.worker.on("exit", (code) => {
      if (!this.closed) this.rejectAll(new Error(`Nuxt lint worker exited with ${code}`));
    });
  }

  run(task: NuxtLintCheckerTask): Promise<NuxtLintCheckerResult> {
    if (this.closed) return Promise.reject(new Error("Nuxt lint worker is closed"));
    const id = ++this.nextId;
    return new Promise((resolve, reject) => {
      this.pending.set(id, { reject, resolve });
      this.worker.postMessage({ id, task } satisfies WorkerRequest);
    });
  }

  async close(): Promise<void> {
    if (this.closed) return;
    this.closed = true;
    this.rejectAll(new Error("Nuxt lint worker closed"));
    await this.worker.terminate();
  }

  private rejectAll(error: Error): void {
    for (const pending of this.pending.values()) pending.reject(error);
    this.pending.clear();
  }
}
