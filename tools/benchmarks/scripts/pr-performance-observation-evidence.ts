/** Evidence custody for advisory PR measurements; no credentials are inherited. */
import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
import {
  closeSync,
  existsSync,
  lstatSync,
  mkdirSync,
  openSync,
  readdirSync,
  readFileSync,
  readlinkSync,
  readSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { join, relative, resolve } from "node:path";
import { performance } from "node:perf_hooks";
import { laneEnvironment } from "./compare-pr-lanes.ts";

export function sha256(file: string): string {
  const hash = createHash("sha256"),
    buffer = Buffer.allocUnsafe(1024 * 1024);
  const fd = openSync(file, "r");
  try {
    for (;;) {
      const bytes = readSync(fd, buffer, 0, buffer.length, null);
      if (!bytes) break;
      hash.update(buffer.subarray(0, bytes));
    }
  } finally {
    closeSync(fd);
  }
  return hash.digest("hex");
}

export function writeJson(file: string, value: unknown): void {
  writeFileSync(file, JSON.stringify(value, null, 2) + "\n");
}

export function corpusInventory(root: string) {
  const files: { path: string; bytes: number; sha256: string }[] = [];
  const links: { path: string; target: string }[] = [];
  const skipped = new Set(["node_modules", ".git", "target", "dist", ".vize", "__out__"]);
  const walk = (directory: string) => {
    for (const name of readdirSync(directory).sort()) {
      if (skipped.has(name)) continue;
      const file = join(directory, name),
        info = lstatSync(file);
      const path = relative(root, file).replaceAll("\\", "/");
      if (info.isSymbolicLink()) links.push({ path, target: readlinkSync(file) });
      else if (info.isDirectory()) walk(file);
      else if (info.isFile()) files.push({ path, bytes: info.size, sha256: sha256(file) });
    }
  };
  walk(root);
  return {
    root,
    files,
    links,
    bytes: files.reduce((sum, file) => sum + file.bytes, 0),
    sfcFiles: files.filter((file) => file.path.endsWith(".vue")).map((file) => file.path),
    sha256: createHash("sha256").update(JSON.stringify({ files, links })).digest("hex"),
  };
}

export function observationEnvironment(threads: number | "max"): NodeJS.ProcessEnv {
  const env: NodeJS.ProcessEnv = { NO_COLOR: "1", VIZE_BENCH: "1", LC_ALL: "C", LANG: "C" };
  // Reconstruct the complete child environment so raw evidence can safely store every key.
  for (const key of ["PATH", "HOME", "TMPDIR", "TMP", "TEMP", "XDG_CACHE_HOME", "GOMAXPROCS"])
    if (process.env[key] !== undefined) env[key] = process.env[key];
  return laneEnvironment(threads, env);
}

export type Invocation = {
  executable: string;
  args: string[];
  cwd: string;
  env: NodeJS.ProcessEnv;
  rawDir: string;
  timeoutMs: number;
  memory?: boolean;
};

export function parseGnuTime(text: string): number | null {
  const match = /^\s*Maximum resident set size \(kbytes\):\s*(\d+)\s*$/mu.exec(text);
  return match && Number(match[1]) > 0 ? Number(match[1]) * 1024 : null;
}

export function runObservation(options: Invocation) {
  mkdirSync(options.rawDir, { recursive: true });
  const timedOutByWrapper = process.platform === "linux" && existsSync("/usr/bin/timeout");
  let executable = options.executable,
    args = options.args;
  if (timedOutByWrapper) {
    args = [
      "--signal=TERM",
      "--kill-after=2s",
      `${options.timeoutMs / 1000}s`,
      executable,
      ...args,
    ];
    executable = "/usr/bin/timeout";
  }
  const timeFile = join(options.rawDir, "gnu-time.txt");
  if (options.memory) {
    if (process.platform !== "linux" || !existsSync("/usr/bin/time"))
      return {
        receipt: {
          status: null,
          signal: null,
          error: "GNU time unavailable",
          wallMs: null,
          maxRssBytes: null,
        },
        stdout: "",
        stderr: "",
      };
    args = ["-v", "-o", timeFile, executable, ...args];
    executable = "/usr/bin/time";
  }
  const startedAt = new Date().toISOString(),
    start = performance.now();
  const result = spawnSync(executable, args, {
    cwd: options.cwd,
    env: options.env,
    encoding: "utf8",
    maxBuffer: 32 * 1024 * 1024,
    timeout: timedOutByWrapper ? options.timeoutMs + 5000 : options.timeoutMs,
    killSignal: "SIGKILL",
  });
  const wallMs = performance.now() - start;
  writeFileSync(join(options.rawDir, "stdout.txt"), result.stdout ?? "");
  writeFileSync(join(options.rawDir, "stderr.txt"), result.stderr ?? "");
  const receipt = {
    startedAt,
    finishedAt: new Date().toISOString(),
    command: {
      executable: options.executable,
      args: options.args,
      cwd: options.cwd,
      env: options.env,
    },
    supervisor: {
      executable,
      args,
      timeoutMs: options.timeoutMs,
      processGroupTimeout: timedOutByWrapper,
    },
    status: result.status,
    signal: result.signal,
    error: result.error?.message ?? null,
    wallMs,
    maxRssBytes:
      options.memory && existsSync(timeFile) ? parseGnuTime(readFileSync(timeFile, "utf8")) : null,
    rawDir: resolve(options.rawDir),
  };
  writeJson(join(options.rawDir, "invocation.json"), receipt);
  return { receipt, stdout: result.stdout ?? "", stderr: result.stderr ?? "" };
}

export function invocationSucceeded(run: ReturnType<typeof runObservation>, allowOne: boolean) {
  return (
    !run.receipt.error &&
    !run.receipt.signal &&
    (run.receipt.status === 0 || (allowOne && run.receipt.status === 1)) &&
    !/panicked at|unrecognized (?:option|subcommand)|unknown (?:option|argument)|unexpected argument|failed to (?:start|spawn|load|resolve)|error loading|invalid configuration|error formatting|format error|file\(s\) had errors|^Error:/imu.test(
      run.stderr,
    )
  );
}

export function binaryEvidence(file: string, rawDir: string, timeoutMs: number) {
  const version = runObservation({
    executable: file,
    args: ["--version"],
    cwd: process.cwd(),
    env: observationEnvironment(1),
    rawDir,
    timeoutMs,
  });
  return {
    path: file,
    bytes: statSync(file).size,
    sha256: sha256(file),
    version: version.stdout.trim() || version.stderr.trim(),
    versionStatus: version.receipt.status,
  };
}
