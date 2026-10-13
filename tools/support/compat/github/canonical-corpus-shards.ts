// Process-level shards for the canonical production-reach and SSR observers.
//
// Both sweeps clear and read the process-wide profiler around every compile,
// so they cannot use threads. Each observer instead builds its test binary
// once and runs several copies concurrently: one "base" process runs every
// test except the canonical sweep, and each "shard" process runs only the
// sweep over the files whose position in the collected corpus vector is
// congruent to its index. Each shard writes the logical paths it swept to a
// NUL-terminated vector. canonical-corpus-shard-evidence.ts proves the
// vectors exactly cover files.json and sums the counters.
import assert from "node:assert/strict";
import { spawn, spawnSync } from "node:child_process";
import {
  closeSync,
  mkdirSync,
  mkdtempSync,
  openSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import {
  artifactDirectory,
  cargoArguments,
  exitLine,
  observerProcesses,
  prebuildTargets,
  processArguments,
  processEnvironment,
  processHeader,
  shardedObservers,
} from "./canonical-corpus-shard-plan.ts";
import type { Finished, ObserverProcess, Suite } from "./canonical-corpus-shard-plan.ts";

type Built = { readonly executable: string; readonly manifestDirectory: string };

/** Build one suite's test binary and return its executable. */
export function buildSuite(suite: Suite, cwd: string, output: (bytes: Buffer) => void): Built {
  const result = spawnSync(
    "cargo",
    cargoArguments(suite, ["--no-run", "--message-format=json-render-diagnostics"]),
    { cwd, maxBuffer: 1024 * 1024 * 1024, stdio: ["ignore", "pipe", "pipe"] },
  );
  output(result.stderr ?? Buffer.alloc(0));
  assert(!result.error && result.status === 0, `cargo test --no-run failed for ${suite.test}`);
  const executables = (result.stdout ?? Buffer.alloc(0))
    .toString("utf8")
    .split("\n")
    .filter((line) => line.startsWith("{"))
    .map((line) => JSON.parse(line) as Record<string, unknown>)
    .filter((message) => {
      const target = message.target as { name?: unknown; kind?: unknown } | undefined;
      const profile = message.profile as { test?: unknown } | undefined;
      return (
        message.reason === "compiler-artifact" &&
        target?.name === suite.test &&
        Array.isArray(target.kind) &&
        target.kind.includes("test") &&
        profile?.test === true &&
        typeof message.executable === "string"
      );
    });
  assert.equal(executables.length, 1, `Missing unique ${suite.test} test executable`);
  const message = executables[0] as { executable: string; manifest_path: string };
  return { executable: message.executable, manifestDirectory: dirname(message.manifest_path) };
}

function runProcess(
  entry: ObserverProcess,
  built: Built,
  cwd: string,
  outputPath: string,
): Promise<Finished> {
  const env = processEnvironment(entry, process.env, built.manifestDirectory, cwd);
  const descriptor = openSync(outputPath, "w");
  const started = Date.now();
  return new Promise((resolvePromise) => {
    const child = spawn(built.executable, processArguments(entry), {
      cwd: built.manifestDirectory,
      env,
      stdio: ["ignore", descriptor, descriptor],
    });
    let settled = false;
    const finish = (status: number | null, signal: string | null) => {
      if (settled) return;
      settled = true;
      closeSync(descriptor);
      resolvePromise({ status, signal, elapsed: Date.now() - started });
    };
    child.on("error", () => finish(null, "SPAWN"));
    child.on("close", (status, signal) => finish(status, signal));
  });
}

/** Build once, run every observer process concurrently, and write one log. */
export async function runObserver(
  observer: string,
  cwd: string,
  logName: string,
): Promise<boolean> {
  const directory = resolve(cwd, artifactDirectory);
  mkdirSync(directory, { recursive: true });
  const chunks: Buffer[] = [];
  const write = (bytes: Buffer) => {
    chunks.push(bytes);
    process.stdout.write(bytes);
  };
  const processes = observerProcesses(observer);
  const built = new Map<Suite, Built>();
  try {
    for (const suite of shardedObservers[observer] ?? [])
      built.set(suite, buildSuite(suite, cwd, write));
  } catch (error) {
    writeFileSync(join(directory, logName), Buffer.concat(chunks));
    throw error;
  }
  const scratch = mkdtempSync(join(tmpdir(), `canonical-${observer}-`));
  try {
    const finished = await Promise.all(
      processes.map(async (entry, position) => {
        const outputPath = join(scratch, `${position}.log`);
        const result = await runProcess(entry, built.get(entry.suite) as Built, cwd, outputPath);
        process.stdout.write(`${exitLine(entry, result)}\n`);
        return { entry, result, outputPath };
      }),
    );
    for (const { entry, result, outputPath } of finished) {
      const output = readFileSync(outputPath);
      const segment = Buffer.concat([
        Buffer.from(`${processHeader(entry)}\n`),
        output,
        Buffer.from(output.length && output.at(-1) !== 10 ? "\n" : ""),
        Buffer.from(`${exitLine(entry, result)}\n`),
      ]);
      chunks.push(segment);
      process.stdout.write(segment);
    }
    writeFileSync(join(directory, logName), Buffer.concat(chunks));
    return finished.every(({ result }) => result.status === 0 && result.signal === null);
  } finally {
    rmSync(scratch, { recursive: true, force: true });
  }
}

/** Compile every test binary an observer will run, without running it. */
export function prebuild(observer: string, cwd: string): void {
  const targets = prebuildTargets[observer];
  assert(targets, `Unknown canonical observer ${observer}`);
  for (const target of targets) {
    const result = spawnSync("cargo", cargoArguments(target, ["--no-run"]), {
      cwd,
      stdio: "inherit",
    });
    assert(!result.error && result.status === 0, `cargo test --no-run failed for ${target.test}`);
  }
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const [mode, observer, logName] = process.argv.slice(2);
  const cwd = process.cwd();
  if (mode === "prebuild" && observer) prebuild(observer, cwd);
  else if (mode === "run" && observer && logName) {
    if (!(await runObserver(observer, cwd, logName))) process.exitCode = 1;
  } else throw new Error("Expected canonical shards prebuild OBSERVER | run OBSERVER LOG");
}
