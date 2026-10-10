import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { performance } from "node:perf_hooks";

export const hostVersions = [
  { version: "1.78.0", types: "7.0.2001" },
  { version: "1.86.0", types: "7.0.2003" },
];

export function parseHostOptions(args, environment = process.env) {
  const options = new Map();
  for (let index = 0; index < args.length; index++) {
    const separator = args[index].indexOf("=");
    const flag = separator < 0 ? args[index] : args[index].slice(0, separator);
    const inline = separator < 0 ? undefined : args[index].slice(separator + 1);
    assert.ok(["--host-mode", "--phase-walltime"].includes(flag), `unknown option ${flag}`);
    assert.ok(!options.has(flag), `duplicate option ${flag}`);
    const value = inline ?? args[++index];
    assert.ok(value && !value.startsWith("--"), `missing value for ${flag}`);
    options.set(flag, value);
  }
  const mode = options.get("--host-mode") ?? environment.VIZE_OXLINT_HOST_MODE ?? "concurrent";
  assert.ok(["serial", "concurrent"].includes(mode), "host mode must be serial or concurrent");
  const phaseWalltime = options.get("--phase-walltime") ?? environment.VIZE_OXLINT_PHASE_WALLTIME;
  if (phaseWalltime !== undefined) assert.ok(phaseWalltime.length > 0, "empty phase walltime path");
  return { mode, workers: mode === "serial" ? 1 : 2, ...(phaseWalltime ? { phaseWalltime } : {}) };
}

// The existing per-host checks contain synchronous tools. Run those checks in
// separate real processes; Promise wrappers around spawnSync do not overlap.
export async function runHostProcesses(jobs, { workers, timeoutMs = 1_800_000 }) {
  assert.ok([1, 2].includes(workers), "host concurrency is bounded to one or two workers");
  assert.equal(new Set(jobs.map(({ id }) => id)).size, jobs.length, "duplicate host worker");
  const results = Array.from({ length: jobs.length });
  const active = new Set();
  let next = 0;
  let interrupted;
  const kill = (child) => {
    if (!child.pid) return;
    try {
      // A worker owns its complete process group, including synchronous npm,
      // Oxlint and type-aware descendants. Timeout must not orphan those tools.
      process.kill(process.platform === "win32" ? child.pid : -child.pid, "SIGKILL");
    } catch (error) {
      if (error.code !== "ESRCH") throw error;
    }
  };
  const stop = (signal) => {
    interrupted = `interrupted by ${signal}`;
    for (const child of active) kill(child);
  };
  const onInterrupt = () => stop("SIGINT");
  const onTerminate = () => stop("SIGTERM");
  const onExit = () => {
    for (const child of active) kill(child);
  };
  process.once("SIGINT", onInterrupt);
  process.once("SIGTERM", onTerminate);
  process.once("exit", onExit);
  const run = (job) =>
    new Promise((resolve) => {
      fs.mkdirSync(job.output, { recursive: true });
      const started = performance.now();
      const finish = (result) => {
        const row = { id: job.id, ...result, elapsedMs: performance.now() - started };
        fs.writeFileSync(
          path.join(job.output, "worker-process.json"),
          JSON.stringify(row, null, 2) + "\n",
        );
        resolve(row);
      };
      if (interrupted) return finish({ status: null, signal: null, error: interrupted });
      const stdout = fs.openSync(path.join(job.output, "worker.stdout.log"), "w");
      const stderr = fs.openSync(path.join(job.output, "worker.stderr.log"), "w");
      let child;
      try {
        child = spawn(job.command, job.args, {
          cwd: job.cwd,
          env: job.env ?? process.env,
          detached: process.platform !== "win32",
          stdio: ["ignore", stdout, stderr],
        });
      } catch (error) {
        finish({ status: null, signal: null, error: String(error) });
        return;
      } finally {
        fs.closeSync(stdout);
        fs.closeSync(stderr);
      }
      active.add(child);
      let failure;
      const timer = setTimeout(() => {
        failure = `host worker timed out after ${job.timeoutMs ?? timeoutMs} ms`;
        kill(child);
      }, job.timeoutMs ?? timeoutMs);
      child.once("error", (error) => {
        failure = String(error);
      });
      child.once("close", (status, signal) => {
        clearTimeout(timer);
        active.delete(child);
        finish({
          pid: child.pid,
          status,
          signal,
          ...(failure || interrupted ? { error: failure ?? interrupted } : {}),
        });
      });
    });
  try {
    await Promise.all(
      Array.from({ length: Math.min(workers, jobs.length) }, async () => {
        while (next < jobs.length) {
          const index = next++;
          results[index] = await run(jobs[index]);
        }
      }),
    );
    return results;
  } finally {
    for (const child of active) kill(child);
    process.removeListener("SIGINT", onInterrupt);
    process.removeListener("SIGTERM", onTerminate);
    process.removeListener("exit", onExit);
  }
}

export function requireSuccessfulHosts(results) {
  const failures = results.filter(({ status, signal, error }) => status !== 0 || signal || error);
  if (failures.length)
    throw new AggregateError(
      failures.map(
        ({ id, status, signal, error }) =>
          new Error(`${id}: ${error ?? signal ?? `exit ${status}`}`),
      ),
      `Oxlint qualification failed for ${failures.map(({ id }) => id).join(", ")}; all worker evidence retained`,
    );
}

export function aggregateNativeEvidence(custody, inventory, segments) {
  const counts = new Map();
  const calls = new Map();
  const phases = new Set();
  let events = 0;
  let processes = 0;
  for (const segment of segments) {
    assert.deepEqual(segment.custody.source, custody.source);
    assert.deepEqual(segment.custody.toolchain, custody.toolchain);
    assert.deepEqual(segment.custody.binary, custody.binary);
    assert.deepEqual(segment.observed.inventory, inventory);
    for (const phase of segment.phases) {
      assert.ok(!phases.has(phase), `duplicate native phase ${phase}`);
      phases.add(phase);
    }
    for (const [phase, entries] of segment.observed.counts) {
      assert.ok(segment.phases.includes(phase), `unexpected native phase ${phase}`);
      counts.set(phase, entries);
    }
    for (const [phase, entries] of segment.observed.calls) {
      assert.ok(segment.phases.includes(phase), `unexpected native phase ${phase}`);
      calls.set(phase, entries);
    }
    events += segment.observed.events;
    processes += segment.observed.processes;
  }
  return { counts, calls, events, processes, inventory };
}
