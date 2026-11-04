/** Linux-only raw runner; process-tree memory is sampled in a separate pass. */
import assert from "node:assert/strict";
import { spawn, spawnSync } from "node:child_process";
import { closeSync, mkdirSync, openSync, readFileSync, readdirSync, realpathSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { reapedChildCpuMs } from "./typecheck-command.mjs";
import { TreeSampler } from "./lsp-vs-volar-metrics.ts";
import { GNU_TIME_FORMAT, parseGnuTime } from "./typecheck-corpus-pair-parse.ts";
import {
  assertNativeReceipt,
  freshCache,
  saveRawRun,
  sha256,
  writeJson,
  type NativeAuthority,
  type RawRun,
} from "./typecheck-owned-pair-evidence.ts";

const hook = join(dirname(fileURLToPath(import.meta.url)), "typecheck-owned-pair-native.cjs");
export function requireLinuxActionsRunner(env = process.env, platform = process.platform) {
  assert.equal(platform, "linux", "the raw runner requires Linux");
  assert.equal(env.GITHUB_ACTIONS, "true", "the raw runner requires Actions");
}
export function requireMeasurementRunner(env = process.env, platform = process.platform) {
  requireLinuxActionsRunner(env, platform);
  assert.equal(env.VIZE_PAIR_RUNNER_LABEL, "blacksmith-32vcpu-ubuntu-2404");
  for (const key of [
    "NODE_OPTIONS",
    "NAPI_RS_NATIVE_LIBRARY_PATH",
    "NAPI_RS_FORCE_WASI",
    "VIZE_CHECKERS",
  ])
    assert.ok(!env[key], `ambient ${key} invalidates the pair`);
}
export function requireTreeSamplerUnits() {
  for (const [key, expected] of [
    ["CLK_TCK", "100"],
    ["PAGESIZE", "4096"],
  ]) {
    const result = spawnSync("getconf", [key], { encoding: "utf8" });
    assert.equal(result.status, 0);
    assert.equal(
      result.stdout.trim(),
      expected,
      `existing tree sampler assumes ${key}=${expected}`,
    );
  }
}
export type Invocation = {
  command: string[];
  cwd: string;
  out: string;
  native: NativeAuthority | null;
  timeoutMs: number;
  memory: boolean;
  probeEnv?: Record<string, string>;
};

function nativeEvidence(directory: string, expected: NativeAuthority | null) {
  const observed = readdirSync(directory).flatMap((file) =>
    readFileSync(join(directory, file), "utf8")
      .trim()
      .split("\n")
      .filter(Boolean)
      .map((line) => JSON.parse(line)),
  );
  if (!expected) assert.deepEqual(observed, [], "Vue must not load a Vize native provider");
  else {
    assert.ok(observed.length > 0, "the actual checker tree loaded no Vize native image");
    for (const row of observed) {
      assertNativeReceipt(expected, {
        ...row,
        source: expected.source,
        sha256: sha256(readFileSync(row.image)),
      });
    }
  }
  return observed;
}

/** Every launched command gets fresh user cache; no output is parsed before raw files close. */
export async function runOwnedInvocation(invocation: Invocation) {
  requireMeasurementRunner();
  return runLinuxInvocation(invocation);
}

/** The same raw execution primitive used by finite Actions integration tests.
 * Measurement entry points retain the stricter Blacksmith wrapper above. */
export async function runLinuxInvocation(invocation: Invocation) {
  requireLinuxActionsRunner();
  if (invocation.memory) requireTreeSamplerUnits();
  for (const key of Object.keys(invocation.probeEnv ?? {}))
    assert.ok(key.startsWith("VIZE_MAPPER_PROBE_"), "only byte-probe environment may be added");
  mkdirSync(invocation.out, { recursive: true });
  const nativeDir = join(invocation.out, "native");
  mkdirSync(nativeDir);
  return freshCache(async (cache) => {
    const env = {
      ...process.env,
      ...cache,
      ...invocation.probeEnv,
      NO_COLOR: "1",
      NUXT_TELEMETRY_DISABLED: "1",
      NODE_OPTIONS: `--require=${hook}`,
      VIZE_PAIR_NATIVE_RECEIPTS: nativeDir,
    };
    const stdoutFile = join(invocation.out, "stdout.raw"),
      stderrFile = join(invocation.out, "stderr.raw");
    const stdout = openSync(stdoutFile, "wx"),
      stderr = openSync(stderrFile, "wx");
    const timeFile = join(invocation.out, "gnu-time.raw");
    const args = ["-q", "-f", GNU_TIME_FORMAT, "-o", timeFile, ...invocation.command];
    const cpuBefore = reapedChildCpuMs();
    const started = performance.now();
    const child = spawn("/usr/bin/time", args, {
      cwd: invocation.cwd,
      env,
      stdio: ["ignore", stdout, stderr],
      detached: true,
    });
    const sampler =
      invocation.memory && child.pid ? new TreeSampler(() => [child.pid!], 100) : null;
    const executablePaths = new Set<string>(),
      executableErrors: string[] = [];
    if (sampler)
      sampler.onReading = (reading) => {
        for (const pid of reading.pids) {
          try {
            executablePaths.add(realpathSync(`/proc/${pid}/exe`));
          } catch (failure) {
            const code = (failure as NodeJS.ErrnoException).code;
            if (code !== "ENOENT" && code !== "ESRCH") executableErrors.push(String(failure));
          }
        }
      };
    sampler?.start();
    sampler?.mark("spawn");
    let error: string | null = null;
    let timedOut = false;
    const timer = setTimeout(() => {
      timedOut = true;
      if (child.pid) {
        try {
          process.kill(-child.pid, "SIGKILL");
        } catch {}
      }
    }, invocation.timeoutMs);
    const exit = await new Promise<{ status: number | null; signal: string | null }>((resolve) => {
      child.once("error", (failure) => {
        error = failure.message;
      });
      child.once("close", (status, signal) => resolve({ status, signal }));
    });
    clearTimeout(timer);
    sampler?.stop();
    const wallMs = performance.now() - started;
    const cpuAfter = reapedChildCpuMs();
    closeSync(stdout);
    closeSync(stderr);
    const raw: RawRun = {
      stdout: readFileSync(stdoutFile),
      stderr: readFileSync(stderrFile),
      ...exit,
      error: timedOut ? "measurement deadline exceeded" : error,
    };
    const metadata = {
      command: invocation.command,
      cwd: realpathSync(invocation.cwd),
      cache: { path: cache.XDG_CACHE_HOME, emptyAtLaunch: true },
      controlledEnvironment: {
        NO_COLOR: "1",
        NUXT_TELEMETRY_DISABLED: "1",
        NODE_OPTIONS: env.NODE_OPTIONS,
        ...invocation.probeEnv,
      },
      memoryPass: invocation.memory,
      wallMs,
      treeCpuMs: cpuBefore == null || cpuAfter == null ? null : cpuAfter - cpuBefore,
      nativeHook: { path: hook, sha256: sha256(readFileSync(hook)), includedInTiming: true },
    };
    saveRawRun(invocation.out, raw, metadata);
    if (sampler)
      writeJson(join(invocation.out, "tree-memory.json"), {
        intervalMs: 100,
        samples: sampler.samples,
        executablePaths: [...executablePaths].sort(),
        executableErrors,
        peakTreeRssBytes: sampler.peak("rssBytes"),
        peakTreePssBytes: sampler.peak("pssBytes"),
        note: "Sampled whole-tree RSS sums shared pages; PSS apportions shared pages. Separate from timed single-process GNU %M.",
      });
    assert.equal(raw.error, null);
    assert.equal(raw.signal, null);
    const time = parseGnuTime(readFileSync(timeFile, "utf8"));
    assert.equal(time.status, raw.status);
    assert.notEqual(metadata.treeCpuMs, null, "whole-tree CPU unavailable");
    const native = nativeEvidence(nativeDir, invocation.native);
    if (sampler)
      assert.ok(
        sampler.samples.some((sample) => sample.processes > 0),
        "no live tree memory sample",
      );
    const receipt = {
      ...metadata,
      status: raw.status,
      native,
      executablePaths: [...executablePaths].sort(),
      executableErrors,
      gnuTime: { ...time, maxSingleProcessRssKb: time.maxRssKb },
      memory: sampler
        ? {
            peakTreeRssBytes: sampler.peak("rssBytes"),
            peakTreePssBytes: sampler.peak("pssBytes"),
            sampleCount: sampler.samples.length,
          }
        : null,
      memoryQualified: sampler
        ? sampler.samples.some((sample) => sample.processes > 0) &&
          sampler.samples
            .filter((sample) => sample.processes > 0)
            .every((sample) => sample.pssBytes != null)
        : null,
    };
    writeJson(join(invocation.out, "receipt.json"), receipt);
    return { raw, receipt };
  });
}
