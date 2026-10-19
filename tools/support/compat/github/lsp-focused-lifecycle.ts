// Replay the selected original real-project LSP packets serially, retaining
// every spawn failure, wire verdict and optional resource observation.
// usage (workflow, CAMPAIGN_DIR and CORSA_PATH set): node lsp-focused-lifecycle.ts
import assert from "node:assert/strict";
import { spawn, spawnSync, type ChildProcess } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { receiptJson } from "./lsp-focused-campaign.ts";

export const packetCommand = [
  "node",
  "--test",
  "--test-concurrency=1",
  "tests/tooling/real-project-lsp.test.ts",
] as const;

// Reuse the existing Linux sampler without changing the packet or its own
// Maestro-tree resource oracle. This observer's root also includes the harness
// and observer; raw PID/birth/exe rows preserve the distinction from native
// Vize/Corsa work. Trace-only overhead makes these observations unsuitable for
// paired speed claims.
export const resourceObserver = `
import fs from 'node:fs';
import crypto from 'node:crypto';
import { createSampler } from './tools/support/compat/davinci/lib/lsp-resource-sampler.mjs';
const [rootPid, directory] = process.argv.slice(1);
const source = 'tools/support/compat/davinci/lib/lsp-resource-sampler.mjs';
const sampler = createSampler(Number(rootPid));
const started = process.hrtime.bigint();
let count = 0;
const metadata = {
  sourceRevision: process.env.GITHUB_SHA,
  samplerSource: source,
  samplerSha256: crypto.createHash('sha256').update(fs.readFileSync(source)).digest('hex'),
  rootPid: Number(rootPid),
  scope: 'workflow harness and sampled descendants, including observer; reparented CPU after leaving the tree is excluded',
  instrumented: true,
};
function record() {
  const snapshot = sampler.sample();
  const processes = snapshot.processes.map((entry) => {
    let executable = null;
    try { executable = fs.readlinkSync(\`/proc/\${entry.pid}/exe\`); } catch {}
    return { ...entry, executable };
  });
  fs.appendFileSync(\`\${directory}/resource-samples.jsonl\`, JSON.stringify({
    ...snapshot, processes, wallTime: new Date().toISOString(),
    elapsedNs: String(process.hrtime.bigint() - started),
  }) + '\\n');
  count++;
  return snapshot;
}
record();
const interval = setInterval(record, 500);
process.once('SIGTERM', () => {
  clearInterval(interval);
  let final;
  let result;
  let failure;
  try { final = record(); } catch (error) { failure = error; }
  try { result = sampler.stop(); } catch (error) { failure ??= error; }
  if (failure) {
    fs.writeFileSync(\`\${directory}/resource-error.txt\`, String(failure));
    process.exitCode = 1;
    return;
  }
  try {
    fs.writeFileSync(\`\${directory}/resource-summary.json\`, JSON.stringify({
      ...metadata, ...result, final, rawSamples: count,
    }, null, 2) + '\\n');
  } catch (error) {
    fs.writeFileSync(\`\${directory}/resource-error.txt\`, String(error));
    process.exitCode = 1;
  }
});
`;

export class ObserverTimeout extends Error {}

/** A running resource observer. */
export interface Observer {
  /** Exit status, or null while running. */
  poll(): number | null;
  signal(signal: NodeJS.Signals): void;
  /** Resolve the exit status; reject with ObserverTimeout after `timeoutMs`. */
  wait(timeoutMs?: number): Promise<number | null>;
  kill(): void;
}

export interface RunOptions {
  readonly env?: NodeJS.ProcessEnv;
  readonly stdout?: number;
  readonly stderr?: number;
}

/** Process creation; both methods throw when the process cannot be spawned. */
export interface LifecycleProcesses {
  run(command: readonly string[], options?: RunOptions): number | null;
  observe(command: readonly string[], stdout: number, stderr: number): Observer;
}

export interface SessionReceipt {
  readonly id: string;
  readonly command: readonly string[];
  readonly exitStatus: number | null;
  readonly commandError: string | null;
  readonly wireExitStatus: number | null;
  readonly durationMs: number;
  readonly resourceExitStatus: number | null;
}

interface Plan {
  readonly preparationTrace: boolean;
  readonly shardCount: number;
  readonly projects: readonly { readonly id: string; readonly shardIndex: number }[];
}

function readPlan(file: string): Plan {
  const value: unknown = JSON.parse(fs.readFileSync(file, "utf8"));
  assert.ok(typeof value === "object" && value !== null, "plan.json");
  const { preparationTrace, shardCount, projects } = value as Record<string, unknown>;
  assert.ok(Array.isArray(projects), "plan.json projects");
  return {
    preparationTrace: preparationTrace === true,
    shardCount: Number(shardCount),
    projects: projects.map((project: unknown) => {
      const { id, shardIndex } = project as Record<string, unknown>;
      assert.equal(typeof id, "string", "plan.json project id");
      return { id: String(id), shardIndex: Number(shardIndex) };
    }),
  };
}

const describe = (error: unknown): string =>
  error instanceof Error ? (error.stack ?? String(error)) : String(error);

const sha256 = (file: string): string =>
  createHash("sha256").update(fs.readFileSync(file)).digest("hex");

function writeJson(file: string, value: unknown): void {
  fs.writeFileSync(file, receiptJson(value));
}

async function stopObserver(observer: Observer, directory: string): Promise<number | null> {
  let status: number | null = null;
  try {
    if (observer.poll() === null) observer.signal("SIGTERM");
    status = await observer.wait(10_000);
  } catch (error) {
    if (error instanceof ObserverTimeout) {
      observer.kill();
      status = await observer.wait();
    } else {
      fs.writeFileSync(path.join(directory, "resource-error.txt"), describe(error));
      status = -1;
    }
  } finally {
    if (observer.poll() === null) {
      observer.kill();
      await observer.wait();
    }
  }
  return status;
}

async function replay(
  out: string,
  plan: Plan,
  project: Plan["projects"][number],
  processes: LifecycleProcesses,
): Promise<SessionReceipt> {
  const directory = path.join(out, project.id);
  const env = {
    ...process.env,
    FIXTURE_SHARD_COUNT: String(plan.shardCount),
    FIXTURE_SHARD_INDEX: String(project.shardIndex),
    FIXTURE_REPORT_DIR: directory,
  };
  const before = path.join(directory, "lsp-wire-before.json");
  const verifier = "tests/tooling/support/lsp/verify-session-captures.ts";
  const snapshot = processes.run(["node", verifier, "snapshot", before]);
  assert.equal(snapshot, 0, `wire snapshot failed for ${project.id}`);
  let observer: Observer | null = null;
  let resourceStatus: number | null = null;
  const resourceStreams: number[] = [];
  if (plan.preparationTrace) {
    try {
      for (const stream of ["stdout", "stderr"])
        resourceStreams.push(fs.openSync(path.join(directory, `resource-observer.${stream}`), "w"));
      const [stdout = -1, stderr = -1] = resourceStreams;
      observer = processes.observe(
        ["node", "--input-type=module", "--eval", resourceObserver, String(process.pid), directory],
        stdout,
        stderr,
      );
    } catch (error) {
      fs.writeFileSync(path.join(directory, "resource-error.txt"), describe(error));
      resourceStatus = -1;
    }
  }
  const started = performance.now();
  let exitStatus: number | null = null;
  let commandError: string | null = null;
  let durationMs = 0;
  try {
    const stdout = fs.openSync(path.join(directory, "test.stdout"), "w");
    const stderr = fs.openSync(path.join(directory, "test.stderr"), "w");
    try {
      exitStatus = processes.run(packetCommand, { env, stdout, stderr });
    } catch (error) {
      commandError = describe(error);
      fs.writeSync(stderr, commandError);
    } finally {
      fs.closeSync(stdout);
      fs.closeSync(stderr);
    }
  } finally {
    durationMs = performance.now() - started;
    try {
      if (observer !== null) resourceStatus = await stopObserver(observer, directory);
    } finally {
      for (const stream of resourceStreams) fs.closeSync(stream);
    }
  }
  const wireStdout = fs.openSync(path.join(directory, "wire.stdout"), "w");
  const wireStderr = fs.openSync(path.join(directory, "wire.stderr"), "w");
  let wireExitStatus: number | null;
  try {
    const integrity = path.join(directory, "lsp-wire-integrity.json");
    wireExitStatus = processes.run(["node", verifier, "verify", before, integrity], {
      stdout: wireStdout,
      stderr: wireStderr,
    });
  } finally {
    fs.closeSync(wireStdout);
    fs.closeSync(wireStderr);
  }
  return {
    id: project.id,
    command: packetCommand,
    exitStatus,
    commandError,
    wireExitStatus,
    durationMs,
    resourceExitStatus: resourceStatus,
  };
}

/** Replay every planned project; the final assertion fails after all receipts are written. */
export async function runLifecycle(
  out: string,
  provider: string,
  processes: LifecycleProcesses,
): Promise<SessionReceipt[]> {
  const plan = readPlan(path.join(out, "plan.json"));
  const resolved = fs.realpathSync(provider);
  writeJson(path.join(out, "provider.json"), {
    path: resolved,
    sha256: sha256(resolved),
    lockfileSha256: sha256("pnpm-lock.yaml"),
  });
  const results: SessionReceipt[] = [];
  for (const project of plan.projects) {
    results.push(await replay(out, plan, project, processes));
    writeJson(path.join(out, "sessions.json"), results);
    const directory = path.join(out, project.id);
    fs.writeSync(1, fs.readFileSync(path.join(directory, "test.stdout")));
    fs.writeSync(2, fs.readFileSync(path.join(directory, "test.stderr")));
  }
  assert.ok(
    results.length === plan.projects.length &&
      results.every((result) => result.exitStatus === 0 && result.wireExitStatus === 0),
    "all selected raw packet outcomes retained",
  );
  return results;
}

/** Exit code, or the negated signal number for a signalled process, as receipts record it. */
function exitStatus(code: number | null, signal: string | null): number | null {
  if (code !== null) return code;
  if (signal === null) return null;
  const number: unknown = Object.entries(os.constants.signals).find(
    ([name]) => name === signal,
  )?.[1];
  return typeof number === "number" ? -number : null;
}

function childObserver(child: ChildProcess): Observer {
  const exited = new Promise<number | null>((resolve) => {
    child.once("exit", (code, signal) => resolve(exitStatus(code, signal)));
  });
  return {
    poll: () => exitStatus(child.exitCode, child.signalCode),
    signal: (signal) => void child.kill(signal),
    kill: () => void child.kill("SIGKILL"),
    wait: (timeoutMs) =>
      timeoutMs === undefined
        ? exited
        : new Promise((resolve, reject) => {
            const timer = setTimeout(() => reject(new ObserverTimeout("observer wait")), timeoutMs);
            void exited.then((status) => {
              clearTimeout(timer);
              resolve(status);
            });
          }),
  };
}

export const nodeProcesses: LifecycleProcesses = {
  run(command, options = {}) {
    const [file = "", ...args] = command;
    const result = spawnSync(file, args, {
      env: options.env ?? process.env,
      stdio: ["inherit", options.stdout ?? "inherit", options.stderr ?? "inherit"],
    });
    if (result.error) throw result.error;
    return exitStatus(result.status, result.signal);
  },
  observe(command, stdout, stderr) {
    const [file = "", ...args] = command;
    const child = spawn(file, args, { stdio: ["ignore", stdout, stderr] });
    child.once("error", () => {});
    // spawn reports ENOENT asynchronously; a missing pid means it never started.
    if (child.pid === undefined) throw new Error(`observer spawn failed: ${file}`);
    return childObserver(child);
  },
};

if (import.meta.main) {
  const out = process.env.CAMPAIGN_DIR;
  const provider = process.env.CORSA_PATH;
  assert.ok(out && provider, "CAMPAIGN_DIR and CORSA_PATH are required");
  await runLifecycle(out, provider, nodeProcesses);
}
