/** Untimed native observers: byte-exact RPC tee and compiler-statistics capture. */
import { createHash } from "node:crypto";
import { spawn } from "node:child_process";
import {
  createReadStream,
  createWriteStream,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  realpathSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { join, resolve } from "node:path";
import { createInterface } from "node:readline";
import { finished } from "node:stream/promises";
import type { Readable, Writable } from "node:stream";
import { pathToFileURL } from "node:url";
const environmentKeys = [
  "GOMAXPROCS",
  "RAYON_NUM_THREADS",
  "GOGC",
  "GOMEMLIMIT",
  "GODEBUG",
  "GOEXPERIMENT",
] as const;
const environment = (env: NodeJS.ProcessEnv) =>
  Object.fromEntries(environmentKeys.map((key) => [key, env[key] ?? null]));
const sha = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");
const quote = (value: string) => "'" + value.replaceAll("'", "'\\''") + "'";
export function nativeProxyLauncher(options: {
  node: string;
  module: string;
  out: string;
  executable: string;
  mode: "rpc" | "stats";
}) {
  return (
    "#!/bin/sh\nexec " +
    [options.node, options.module, options.mode, options.out, options.executable]
      .map(quote)
      .join(" ") +
    (options.mode === "stats" ? ' "$@"' : "") +
    "\n"
  );
}
export function classifyNativeInvocation(args: string[], config: unknown, configReadable: boolean) {
  if (
    args.some((arg) =>
      /^(?:--version|-v|--help|-h|-\?|--all|--showConfig|--listFilesOnly)$/iu.test(arg),
    )
  )
    return { classification: "probe", actualProgram: false };
  if (configReadable && config && typeof config === "object" && !Array.isArray(config)) {
    const value = config as { files?: unknown; include?: unknown; references?: unknown };
    if (
      Array.isArray(value.files) &&
      value.files.length === 0 &&
      Array.isArray(value.include) &&
      value.include.length === 0 &&
      (!Array.isArray(value.references) || value.references.length === 0)
    )
      return { classification: "options-validation", actualProgram: false };
    return { classification: "project-compilation", actualProgram: true };
  }
  if (args.some((arg) => !arg.startsWith("-") && /\.(?:[cm]?tsx?|vue)$/iu.test(arg)))
    return { classification: "explicit-source-compilation", actualProgram: true };
  return { classification: "unclassified-config", actualProgram: false };
}
export function nativeObserverCommand(
  args: string[],
  env: NodeJS.ProcessEnv,
  classification: { actualProgram: boolean },
) {
  const childArgs = [...args],
    childEnv = { ...env };
  let addedStatistics = false;
  if (classification.actualProgram) {
    if (!args.some((arg) => /^--extendedDiagnostics(?:=|$)/iu.test(arg))) {
      childArgs.push("--extendedDiagnostics");
      addedStatistics = true;
    }
    childEnv.GODEBUG = [
      ...(env.GODEBUG ?? "")
        .split(",")
        .filter((entry) => entry && entry.split("=")[0] !== "gctrace"),
      "gctrace=1",
    ].join(",");
  }
  return { args: childArgs, env: childEnv, addedStatistics };
}
export function teeNativeBytes(
  source: Readable,
  destination: Writable,
  archive: Writable,
  endDestination: boolean,
) {
  const hash = createHash("sha256");
  let bytes = 0;
  source.on("data", (chunk: Buffer) => {
    bytes += chunk.length;
    hash.update(chunk);
  });
  source.pipe(destination, { end: endDestination });
  source.pipe(archive);
  return {
    done: finished(archive),
    snapshot: () => ({ bytes, sha256: hash.copy().digest("hex") }),
    stop: () => {
      source.unpipe(destination);
      source.unpipe(archive);
      archive.end();
    },
  };
}
async function executableIdentity(path: string) {
  const resolved = realpathSync(path),
    hash = createHash("sha256"),
    stream = createReadStream(resolved);
  for await (const bytes of stream) hash.update(bytes);
  return {
    path: resolve(path),
    realpath: resolved,
    bytes: statSync(resolved).size,
    sha256: hash.digest("hex"),
  };
}
function projectConfig(args: string[], cwd: string) {
  let path: string | null = null;
  for (const [index, arg] of args.entries()) {
    if (["--project", "-p"].includes(arg)) path = args[index + 1] ?? null;
    if (/^(?:--project|-p)=/u.test(arg)) path = arg.slice(arg.indexOf("=") + 1);
  }
  if (!path && args.some((arg) => !arg.startsWith("-") && /\.(?:[cm]?tsx?|vue)$/iu.test(arg)))
    return { path: null, bytes: null, parsed: null, readable: false };
  path = path
    ? resolve(cwd, path)
    : existsSync(join(cwd, "tsconfig.json"))
      ? join(cwd, "tsconfig.json")
      : null;
  if (path && existsSync(path) && statSync(path).isDirectory()) path = join(path, "tsconfig.json");
  if (!path || !existsSync(path)) return { path, bytes: null, parsed: null, readable: false };
  let bytes: Buffer;
  try {
    bytes = readFileSync(path);
  } catch {
    return { path, bytes: null, parsed: null, readable: false };
  }
  try {
    return { path, bytes, parsed: JSON.parse(bytes.toString("utf8")) as unknown, readable: true };
  } catch {
    return { path, bytes, parsed: null, readable: false };
  }
}
const countLabels = new Set([
  "Files",
  "Identifiers",
  "Symbols",
  "Types",
  "Instantiations",
  "Assignability cache size",
  "Identity cache size",
  "Subtype cache size",
  "Strict subtype cache size",
  "Allocations",
  "Total allocations",
  "Total allocated bytes",
]);
async function nativeStatistics(path: string) {
  const values: Record<string, { raw: string; count: number | null }> = {};
  for await (const line of createInterface({
    input: createReadStream(path),
    crlfDelay: Infinity,
  })) {
    const match = /^([^:]+):\s+(.+)$/u.exec(line);
    const label = match?.[1],
      value = match?.[2];
    if (
      label !== undefined &&
      value !== undefined &&
      (countLabels.has(label) || label === "Memory used" || / time$/u.test(label))
    )
      values[label] = {
        raw: value,
        count:
          countLabels.has(label) &&
          /^[0-9,]+$/u.test(value) &&
          Number.isSafeInteger(Number(value.replaceAll(",", "")))
            ? Number(value.replaceAll(",", ""))
            : null,
      };
  }
  return values;
}
export async function runNativeProxy(
  mode: "rpc" | "stats",
  root: string,
  executable: string,
  originalArgs: string[],
) {
  mkdirSync(root, { recursive: true });
  const out = mkdtempSync(join(root, mode + "-")),
    cwd = process.cwd();
  const beforeEnv = environment(process.env),
    before = await executableIdentity(executable);
  const probe = classifyNativeInvocation(originalArgs, null, false).classification === "probe";
  const config =
    mode === "stats" && !probe
      ? projectConfig(originalArgs, cwd)
      : { path: null, bytes: null, parsed: null, readable: false };
  if (config.bytes) writeFileSync(join(out, "config.bytes"), config.bytes);
  const classification =
    mode === "rpc"
      ? { classification: "mapper-rpc", actualProgram: false }
      : classifyNativeInvocation(originalArgs, config.parsed, config.readable);
  const observed =
    mode === "stats"
      ? nativeObserverCommand(originalArgs, process.env, classification)
      : { args: ["content-mapper"], env: { ...process.env }, addedStatistics: false };
  const archive = (name: string) => createWriteStream(join(out, name + ".raw"), { flags: "wx" });
  const streams = {
    stdin: archive("stdin"),
    stdout: archive("stdout"),
    stderr: archive("stderr"),
  };
  const child = spawn(executable, observed.args, {
    cwd,
    env: observed.env,
    stdio: ["pipe", "pipe", "pipe"],
  });
  const io = {
    stdin: teeNativeBytes(process.stdin, child.stdin, streams.stdin, true),
    stdout: teeNativeBytes(child.stdout, process.stdout, streams.stdout, false),
    stderr: teeNativeBytes(child.stderr, process.stderr, streams.stderr, false),
  };
  const errors: string[] = [];
  child.stdin.on("error", (error) => errors.push("child stdin: " + error.message));
  for (const channel of Object.values(io))
    void channel.done.catch((error: Error) => errors.push(error.message));
  const signals: NodeJS.Signals[] = ["SIGINT", "SIGTERM", "SIGHUP"];
  const handlers = signals.map((signal) => {
    const handler = () => {
      child.kill(signal);
    };
    process.on(signal, handler);
    return { signal, handler };
  });
  const exit = await new Promise<{ code: number | null; signal: NodeJS.Signals | null }>((done) => {
    child.once("error", (error) => {
      errors.push(error.message);
      done({ code: 1, signal: null });
    });
    child.once("close", (code, signal) => done({ code, signal }));
  });
  io.stdin.stop();
  process.stdin.pause();
  await Promise.allSettled(Object.values(io).map((channel) => channel.done));
  for (const { signal, handler } of handlers) process.removeListener(signal, handler);
  const after = await executableIdentity(executable);
  const receipt = {
    schemaVersion: 1,
    observerMode: "instrumented-untimed",
    mode,
    executable: before,
    executableAfter: after,
    executableUnchanged: before.sha256 === after.sha256,
    captureQualified: errors.length === 0 && before.sha256 === after.sha256,
    cwd,
    originalArgs,
    actualArgs: observed.args,
    classification: classification.classification,
    actualProgram: classification.actualProgram,
    addedStatistics: observed.addedStatistics,
    observedCheckersArgument: observed.args.includes("--checkers")
      ? (observed.args[observed.args.indexOf("--checkers") + 1] ?? null)
      : null,
    config: {
      path: config.path,
      bytes: config.bytes?.length ?? null,
      sha256: config.bytes ? sha(config.bytes) : null,
      parsed: config.readable,
    },
    environmentBefore: beforeEnv,
    environmentEffective: environment(observed.env),
    environmentAfter: environment(process.env),
    exit,
    errors,
    streams: Object.fromEntries(
      Object.entries(io).map(([key, channel]) => [key, channel.snapshot()]),
    ),
    nativeStatistics: mode === "stats" ? await nativeStatistics(join(out, "stdout.raw")) : null,
    timingUsable: false,
    allocationCounts: null,
    processRss: null,
    statisticsGcScope: "extended statistics and gctrace are observer-only and may force GC",
    memoryScope:
      "Memory used is preserved as a native tool report; Rust allocations and process RSS are separate",
    mapperScope:
      mode === "rpc" ? "explicit source CLI mapper override; byte forwarding only" : null,
  };
  writeFileSync(join(out, "receipt.json"), JSON.stringify(receipt, null, 2) + "\n");
  return exit;
}
if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  const [mode, out, executable, ...args] = process.argv.slice(2);
  if ((mode !== "rpc" && mode !== "stats") || !out || !executable)
    throw new Error("rpc|stats OUT EXEC [args]");
  void runNativeProxy(mode, resolve(out), resolve(executable), args)
    .then((exit) => {
      if (exit.signal) process.kill(process.pid, exit.signal);
      else process.exitCode = exit.code ?? 1;
    })
    .catch((error: Error) => {
      process.stderr.write(error.message + "\n");
      process.exitCode = 1;
    });
}
