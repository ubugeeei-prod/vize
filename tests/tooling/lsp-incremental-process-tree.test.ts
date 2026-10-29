import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { once } from "node:events";
import { test } from "node:test";

import { ChurnMetrics } from "../performance/support/churn-metrics.ts";
import type { LspChurnBudget } from "../performance/support/churn-metrics.ts";
import {
  budgetRegistryPath,
  IncrementalMetrics,
  selectPeakProcessTreeSample,
} from "../performance/support/incremental-metrics.ts";
import type { LspIncrementalBudget } from "../performance/support/incremental-metrics.ts";
import {
  assertStableProcessTreeSample,
  processTreeRss,
  type AttributedProcessTreeRssSample,
} from "../performance/support/process-metrics.ts";

const suite = { id: "misskey-lsp-incremental", title: "Misskey LSP Incremental Oracle" };
const churnSuite = { id: "misskey-lsp-churn", title: "Misskey LSP Churn Stress Oracle" };
const absentPid = Number.MAX_SAFE_INTEGER;

const delay = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

function mutableBudget(metrics: IncrementalMetrics): LspIncrementalBudget {
  return (metrics as unknown as { budget: LspIncrementalBudget }).budget;
}

function mutableChurnBudget(metrics: ChurnMetrics): LspChurnBudget {
  return (metrics as unknown as { budget: LspChurnBudget }).budget;
}

function attributedSample(
  pid: number,
  rssKiB: number,
  birthTicks = "100",
): AttributedProcessTreeRssSample {
  return {
    totalKiB: rssKiB,
    processes: 1,
    members: [
      {
        role: "root",
        pid,
        parentPid: 1,
        rssKiB,
        executable: "node",
        identity: {
          birthTicks,
          resolvedExecutable: "/usr/bin/node",
          executableSha256: "a".repeat(64),
          argv: ["/usr/bin/node", "server.mjs"],
          argvSha256: "b".repeat(64),
        },
      },
    ],
  };
}

test("process-tree RSS accounting sees the probing process and a child", async () => {
  const tree = processTreeRss(process.pid);
  if (process.platform === "win32") {
    assert.equal(tree, null);
    return;
  }
  const child = spawn(process.execPath, ["-e", "setInterval(() => {}, 1000)"], {
    stdio: "ignore",
  });
  try {
    await once(child, "spawn");
    let sampled = tree;
    for (let tries = 0; tries < 50; tries += 1) {
      sampled = processTreeRss(process.pid);
      if (sampled != null && sampled.processes >= 2) break;
      await delay(20);
    }
    assert.ok(sampled != null, "ps-based tree sampling must work on POSIX hosts");
    assert.ok(sampled.processes >= 2, "the root and child processes must both be counted");
    assert.ok(sampled.totalKiB > 0, "a live process tree has resident memory");
  } finally {
    child.kill();
    await Promise.race([once(child, "exit"), delay(1_000)]);
  }
});

test("process-tree RSS accounting returns null when the root process is absent", () => {
  const tree = processTreeRss(absentPid);
  if (process.platform === "win32") {
    assert.equal(tree, null);
    return;
  }
  assert.equal(tree, null);
});

test("Linux process-tree attribution retains stable executable and argv identity", async (t) => {
  if (process.platform !== "linux") {
    t.skip("stable process identity custody uses Linux /proc");
    return;
  }
  const child = spawn(process.execPath, ["-e", "setInterval(() => {}, 1000)"], {
    stdio: "ignore",
  });
  try {
    await once(child, "spawn");
    assert.ok(child.pid != null);
    const sample = processTreeRss(child.pid, { stableIdentity: true });
    assert.ok(sample != null, "a live Linux process must have a stable attributed sample");
    assertStableProcessTreeSample(child.pid, sample, new Map());
    assert.equal(sample.members.length, sample.processes);
    assert.equal(
      sample.members.reduce((sum, member) => sum + member.rssKiB, 0),
      sample.totalKiB,
    );
    const root = sample.members.find((member) => member.role === "root");
    assert.ok(root?.identity != null);
    assert.match(root.identity.birthTicks, /^\d+$/);
    assert.match(root.identity.resolvedExecutable, /^\//);
    assert.match(root.identity.executableSha256, /^[0-9a-f]{64}$/);
    assert.match(root.identity.argvSha256, /^[0-9a-f]{64}$/);
    assert.ok(root.identity.argv.includes("-e"));
  } finally {
    child.kill();
    await Promise.race([once(child, "exit"), delay(1_000)]);
  }
});

test("peak attribution keeps the winning member sum and rejects aggregate drift", () => {
  const samples = {
    baseline: attributedSample(41, 400),
    peak: attributedSample(41, 528),
    settled: attributedSample(41, 480),
  };
  const peak = selectPeakProcessTreeSample(samples);
  assert.ok(peak != null);
  assert.equal(peak.label, "peak");
  assert.equal(peak.totalKiB, 528);
  assert.equal(peak.memberRssSumKiB, 528);
  assert.deepEqual(peak.members, samples.peak.members);

  const corrupt = attributedSample(41, 528);
  corrupt.totalKiB += 1;
  assert.throws(
    () => selectPeakProcessTreeSample({ corrupt }),
    /reports 529 KiB but its members sum to 528 KiB/,
  );
});

test("stable process attribution rejects PID reuse across samples", () => {
  const observedBirthTicks = new Map<number, string>();
  assertStableProcessTreeSample(41, attributedSample(41, 400, "100"), observedBirthTicks);
  assert.throws(
    () => assertStableProcessTreeSample(41, attributedSample(41, 420, "101"), observedBirthTicks),
    /PID 41 was reused: birth ticks changed from 100 to 101/,
  );
  assert.equal(observedBirthTicks.get(41), "100", "a rejected sample must not rewrite custody");
});

test("incremental process-tree RSS failures point at the registry ceiling", (t) => {
  if (process.platform === "win32") {
    t.skip("process-tree RSS accounting is Linux/POSIX only");
    return;
  }
  const metrics = new IncrementalMetrics(process.pid, suite);
  const budget = mutableBudget(metrics);
  budget.maxPeakRssMiB = 1_000_000;
  budget.maxPeakProcessTreeRssMiB = 1;
  metrics.sampleRss("probe");
  assert.throws(
    () => metrics.assertBudgetsSettled(),
    (error: Error) => {
      assert.match(
        error.message,
        /sampled peak LSP process-tree RSS .* MiB is over its 1 MiB budget/,
      );
      assert.match(error.message, /raising maxPeakProcessTreeRssMiB/);
      assert.ok(error.message.includes(budgetRegistryPath));
      return true;
    },
  );
});

test("incremental settlement fails when process-tree sampling misses the server pid", (t) => {
  if (process.platform === "win32") {
    t.skip("process-tree RSS accounting is Linux/POSIX only");
    return;
  }
  const metrics = new IncrementalMetrics(absentPid, suite);
  const budget = mutableBudget(metrics);
  budget.maxPeakRssMiB = 1_000_000;
  budget.maxPeakProcessTreeRssMiB = 1_000_000;
  budget.maxProcessTreeSize = 1_000_000;
  budget.laneBudgetsMs = {};
  metrics.sampleRss("missing-server");
  assert.throws(
    () => metrics.assertBudgetsSettled(),
    (error: Error) => {
      assert.match(error.message, /process-tree RSS sampling did not observe LSP server process/);
      assert.match(error.message, /cannot enforce maxPeakProcessTreeRssMiB or maxProcessTreeSize/);
      return true;
    },
  );
});

test("churn settlement fails when process-tree sampling misses the server pid", (t) => {
  if (process.platform === "win32") {
    t.skip("process-tree RSS accounting is Linux/POSIX only");
    return;
  }
  const metrics = new ChurnMetrics(absentPid, churnSuite);
  const budget = mutableChurnBudget(metrics);
  budget.cyclesPerPhase = 0;
  budget.maxPeakRssMiB = 1_000_000;
  budget.maxPeakProcessTreeRssMiB = 1_000_000;
  budget.maxProcessTreeSize = 1_000_000;
  budget.budgetsMs = {};
  metrics.sampleRss("missing-server");
  assert.throws(
    () => metrics.assertSettled(),
    (error: Error) => {
      assert.match(error.message, /process-tree RSS sampling did not observe LSP server process/);
      assert.match(error.message, /cannot enforce maxPeakProcessTreeRssMiB or maxProcessTreeSize/);
      return true;
    },
  );
});

test("incremental process count failures catch leaked bridge workers", (t) => {
  if (process.platform === "win32") {
    t.skip("process-tree process counting is Linux/POSIX only");
    return;
  }
  const metrics = new IncrementalMetrics(process.pid, suite);
  const budget = mutableBudget(metrics);
  budget.maxPeakRssMiB = 1_000_000;
  budget.maxPeakProcessTreeRssMiB = 1_000_000;
  budget.maxProcessTreeSize = 0;
  metrics.sampleRss("probe");
  assert.throws(
    () => metrics.assertBudgetsSettled(),
    (error: Error) => {
      assert.match(error.message, /the LSP process tree grew to \d+ processes/);
      assert.match(error.message, /worker session is likely leaking/);
      assert.match(error.message, /raising maxProcessTreeSize/);
      return true;
    },
  );
});
