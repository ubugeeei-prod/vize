#!/usr/bin/env node
// Observe one unchanged command; core capture requires a verified private guest.
import { spawn } from "node:child_process";
import fs from "node:fs";
import { constants } from "node:os";
import path from "node:path";
import { setTimeout } from "node:timers/promises";
import { fileURLToPath } from "node:url";
import { diagnoseCore } from "./native-fault-core.ts";
import {
  errorName,
  hashMeasurements,
  observeDescendants,
  privateGuest,
  readText,
  returnCode,
  runCapture,
  sha256File,
  sourceSnapshot,
  writeJson,
} from "./native-fault-support.ts";
import type { Packet } from "./native-fault-support.ts";
import { processRetention } from "./native-fault-retention.ts";

export const observers = { sourceSnapshot, privateGuest, observeDescendants, diagnoseCore };
const uiPhases = [
  { name: "native-build", command: ["vp", "run", "--filter", "./npm/native", "build:ci"] },
  { name: "ui-check", command: ["vp", "run", "--filter", "./npm/ui", "check"] },
];
type Invocation = { command: string[] } | { uiConformance: true };

function attempt<T>(receipt: Packet, name: string, action: () => T): T | undefined {
  const started = performance.now();
  try {
    return action();
  } catch (error) {
    (receipt.observationGaps ??= {})[name] = errorName(error);
  } finally {
    const times = (receipt.observationWalltimeSeconds ??= {});
    times[name] = (times[name] ?? 0) + (performance.now() - started) / 1000;
  }
}

function prepareObservation(invocation: Invocation): { output: string; receipt: Packet } {
  const identity = ["GITHUB_RUN_ID", "GITHUB_RUN_ATTEMPT", "GITHUB_JOB"]
    .map((key) => {
      if (!process.env[key]) throw new Error(`Missing ${key}`);
      return process.env[key];
    })
    .join("-");
  const output = path.join(process.env.RUNNER_TEMP!, `js-native-fault-evidence-${identity}`);
  fs.mkdirSync(output);
  const helper = fileURLToPath(import.meta.url);
  const receipt: Packet = {
    ...("command" in invocation
      ? { command: invocation.command }
      : { mode: "ui-conformance", declaredPhases: uiPhases }),
    cwd: process.cwd(),
    startedAt: Date.now() / 1000,
    orchestrationSha: process.env.GITHUB_SHA,
    helpers: [
      helper,
      path.join(path.dirname(helper), "native-fault-core.ts"),
      path.join(path.dirname(helper), "native-fault-support.ts"),
      path.join(path.dirname(helper), "native-fault-retention.ts"),
    ].map((filename) => ({ path: filename, sha256: sha256File(filename) })),
    sampling: "read-only /proc descendants; sampled maps are not complete-call evidence",
    coreCapture: "unavailable",
    cores: [],
    controlledEnvironment: Object.fromEntries(
      [
        "NODE_OPTIONS",
        "CARGO_BUILD_JOBS",
        "CARGO_TARGET_DIR",
        "RUSTFLAGS",
        "CARGO_ENCODED_RUSTFLAGS",
        "VIZE_TEST_REQUIRE_TSGO",
        "VIZE_TEST_DISABLE_TSGO",
        "VIZE_NUXT_CONFIG_ITERATIONS",
        "VIZE_VUE_RUNTIME_PRODUCTION",
        "VIZE_VUE_RUNTIME_DEVTOOLS",
      ].map((name) => [name, process.env[name] ?? null]),
    ),
  };
  return { output, receipt };
}

function coreLimits(): { soft: string; hard: string } {
  const match = /^Max core file size\s+(\S+)\s+(\S+)/m.exec(
    fs.readFileSync("/proc/self/limits", "utf8"),
  );
  if (!match) throw new Error("Core limits unavailable");
  return { soft: match[1], hard: match[2] };
}

function setCoreLimit(soft: string, hard: string): Packet {
  const result = runCapture(["prlimit", "--pid", String(process.pid), `--core=${soft}:${hard}`]);
  if (result.status !== 0) throw new Error("Unable to update observer core limit");
  return result;
}

async function runOriginal(
  command: string[],
  receipt: Packet,
  sample?: (pid: number) => void,
): Promise<number> {
  receipt.commandStartedAt = Date.now() / 1000;
  const child = spawn(command[0], command.slice(1), { stdio: "inherit" });
  receipt.pid = child.pid;
  let completed = false;
  let spawnError: Error | undefined;
  const completion = new Promise<number>((resolve) => {
    child.once("error", (error) => {
      spawnError = error;
      receipt.spawnError = errorName(error);
      Object.assign(receipt, {
        commandExitCode: null,
        commandSignal: null,
        commandSpawnError: {
          name: errorName(error),
          code: (error as NodeJS.ErrnoException).code ?? null,
        },
      });
      completed = true;
      resolve(1);
    });
    child.once("exit", (status, signal) => {
      Object.assign(receipt, { commandExitCode: status, commandSignal: signal });
      completed = true;
      resolve(returnCode(status, signal));
    });
  });
  const signals = ["SIGINT", "SIGTERM"] as const;
  const previous = signals.map((signal) => process.listeners(signal));
  for (const signal of signals) {
    process.removeAllListeners(signal);
    process.on(signal, () => {
      receipt.forwardedSignal = constants.signals[signal];
      if (!completed) child.kill(signal);
    });
  }
  try {
    while (!completed && sample) {
      if (child.pid !== undefined) sample(child.pid);
      await Promise.race([completion, setTimeout(250, undefined, { ref: false })]);
    }
    const status = await completion;
    Object.assign(receipt, { commandReturnCode: status, commandFinishedAt: Date.now() / 1000 });
    if (spawnError) throw spawnError;
    return status;
  } finally {
    for (const [index, signal] of signals.entries()) {
      process.removeAllListeners(signal);
      for (const handler of previous[index]) process.on(signal, handler);
    }
  }
}

async function runInvocation(
  invocation: Invocation,
  receipt: Packet,
  sample?: (pid: number) => void,
  checkpoint: () => void = () => {},
  packetCount: () => number = () => 0,
): Promise<number> {
  if ("command" in invocation) return runOriginal(invocation.command, receipt, sample);
  receipt.commandStartedAt = Date.now() / 1000;
  receipt.phases = [];
  let status = 0;
  for (const declared of uiPhases) {
    const phase: Packet = { ...declared, sampleSequenceStart: packetCount() };
    receipt.phases.push(phase);
    receipt.activePhase = declared.name;
    checkpoint();
    try {
      status = await runOriginal(
        declared.command,
        phase,
        sample &&
          ((pid) => {
            receipt.pid = pid;
            sample(pid);
          }),
      );
    } catch (error) {
      const code = phase.commandSpawnError?.code;
      if (code !== "ENOENT" && code !== "EACCES") throw error;
      status = code === "ENOENT" ? 127 : 126;
    } finally {
      receipt.pid = phase.pid;
      phase.sampleSequenceEnd = packetCount();
      checkpoint();
    }
    // The original shell && reports 128+signal instead of signalling its parent.
    receipt.commandReturnCode = status < 0 ? 128 - status : status;
    phase.shellReturnCode = receipt.commandReturnCode;
    receipt.commandFinishedAt = phase.commandFinishedAt;
    if (status !== 0) break;
  }
  return receipt.commandReturnCode;
}

async function observeCommand(
  invocation: Invocation,
  output: string,
  receipt: Packet,
): Promise<number> {
  attempt(receipt, "source-before", () =>
    writeJson(path.join(output, "source-before.json"), observers.sourceSnapshot()),
  );
  const guest = attempt(receipt, "guest", () => observers.privateGuest()) ?? {
    privateFirecrackerGuest: false,
  };
  attempt(receipt, "guest-packet", () => writeJson(path.join(output, "guest.json"), guest));
  const originalPattern = readText("/proc/sys/kernel/core_pattern");
  const coreDirectory = path.join(
    process.env.RUNNER_TEMP!,
    path.basename(output).replace("js-native-fault-evidence-", "js-native-fault-cores-"),
  );
  let patternChanged = false;
  if (guest.privateFirecrackerGuest && originalPattern !== null) {
    try {
      const { soft, hard } = coreLimits();
      if (hard === "0") throw new Error("Hard core limit is zero");
      fs.mkdirSync(coreDirectory);
      patternChanged = true;
      receipt.coreSetup = runCapture([
        "sudo",
        "-n",
        "sysctl",
        "-w",
        `kernel.core_pattern=${path.join(coreDirectory, "core.%p")}`,
      ]);
      if (receipt.coreSetup.status === 0) {
        receipt.coreLimitUpdate = setCoreLimit(hard, hard);
        receipt.coreCapture = "enabled-private-guest";
        receipt.originalCoreLimit = [soft, hard].map((limit) =>
          limit === "unlimited" ? -1 : Number(limit),
        );
      }
    } catch (error) {
      receipt.coreLimit = { unavailable: errorName(error) };
    }
  }
  const seen = new Set<string>();
  const providers = new Map<string, Packet>();
  let status: number;
  try {
    status = await runInvocation(
      invocation,
      receipt,
      (pid) => {
        attempt(receipt, "process-maps", () =>
          observers.observeDescendants(pid, seen, providers, output),
        );
      },
      () =>
        attempt(receipt, "phase-outcome", () =>
          writeJson(path.join(output, "outcome.json"), receipt),
        ),
      () => processRetention(seen).observedPackets,
    );
  } finally {
    if (typeof receipt.pid === "number")
      attempt(receipt, "process-maps-terminal", () =>
        observers.observeDescendants(receipt.pid, seen, providers, output, "/proc", "terminal"),
      );
    receipt.processRetention = processRetention(seen);
    Object.assign(receipt, { finishedAt: Date.now() / 1000, sampledMapPackets: seen.size });
    if (patternChanged)
      attempt(receipt, "restore", () => {
        receipt.coreRestore = runCapture([
          "sudo",
          "-n",
          "sysctl",
          "-w",
          `kernel.core_pattern=${originalPattern}`,
        ]);
      });
    attempt(receipt, "outcome", () => writeJson(path.join(output, "outcome.json"), receipt));
  }
  const cores =
    attempt(receipt, "core-inventory", () =>
      fs.existsSync(coreDirectory)
        ? fs
            .readdirSync(coreDirectory)
            .filter((name) => name.startsWith("core."))
            .sort()
            .map((name) => path.join(coreDirectory, name))
        : [],
    ) ?? [];
  for (const core of cores)
    attempt(receipt, `core-${path.basename(core)}`, () =>
      observers.diagnoseCore(core, receipt, output),
    );
  attempt(receipt, "source-after", () =>
    writeJson(path.join(output, "source-after.json"), observers.sourceSnapshot()),
  );
  receipt.observerFileHash = { ...hashMeasurements };
  receipt.observerFinishedAt = Date.now() / 1000;
  attempt(receipt, "outcome-final", () => writeJson(path.join(output, "outcome.json"), receipt));
  return status;
}

export async function main(argv: string[], platform = process.platform): Promise<number> {
  const uiConformance = argv.length === 1 && argv[0] === "--ui-conformance";
  if (!uiConformance && (argv[0] !== "--" || argv.length < 2))
    throw new Error("Expected -- followed by the original command");
  const invocation: Invocation = uiConformance
    ? { uiConformance: true }
    : { command: argv.slice(1) };
  let observation: ReturnType<typeof prepareObservation> | undefined;
  if (platform === "linux" && process.env.GITHUB_ACTIONS === "true") {
    // Setup refusal must still execute the original command once, unchanged.
    try {
      observation = prepareObservation(invocation);
    } catch {
      /* pass through */
    }
  }
  const status = observation
    ? await observeCommand(invocation, observation.output, observation.receipt)
    : await runInvocation(invocation, {});
  if (status < 0) {
    if (platform === "linux")
      attempt(observation?.receipt ?? {}, "observer-core-limit", () =>
        setCoreLimit("0", coreLimits().hard),
      );
    const signal = Object.entries(constants.signals).find(([, number]) => number === -status)?.[0];
    if (signal) {
      process.removeAllListeners(signal);
      process.kill(process.pid, signal);
      // Keep the process alive until its original terminating signal arrives.
      await new Promise(() => {});
    }
  }
  return status;
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    process.exitCode = await main(process.argv.slice(2));
  } catch (error) {
    console.error(error);
    process.exitCode = 1;
  }
}
