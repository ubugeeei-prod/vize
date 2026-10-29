import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { once } from "node:events";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { ChurnMetrics } from "../performance/support/churn-metrics.ts";
import type { LspChurnBudget } from "../performance/support/churn-metrics.ts";
import {
  cachedExecutableSha256,
  classifyExecutableKind,
  classifyRuntimeRole,
} from "../performance/support/linux-process-custody.ts";
import {
  budgetRegistryPath,
  IncrementalMetrics,
  selectPeakProcessTreeSample,
} from "../performance/support/incremental-metrics.ts";
import type { LspIncrementalBudget } from "../performance/support/incremental-metrics.ts";
import { writeIncrementalArtifacts } from "../performance/support/incremental-report.ts";
import type { IncrementalReportData } from "../performance/support/incremental-report.ts";
import {
  assertStableProcessTreeSample,
  parseLinuxProcStat,
  processTreeRss,
  retryStableProcessTreeSample,
  sameLinuxProcessAnchor,
  sameLinuxProcessCustody,
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
          executableKind: "node",
          executableSha256: "a".repeat(64),
          runtimeRole: "native-lsp",
          argvCount: 2,
          argvSha256: "b".repeat(64),
        },
      },
    ],
  };
}

function procStat(
  pid: number,
  parentPid: number | string,
  birthTicks: string,
  rssPages: number | string,
  command = "worker ) name",
): string {
  const fields = Array.from({ length: 21 }, () => "0");
  fields[0] = String(parentPid);
  fields[18] = birthTicks;
  fields[20] = String(rssPages);
  return `${pid} (${command}) S ${fields.join(" ")}`;
}

function reportData(
  sample: AttributedProcessTreeRssSample,
  label = "probe",
): IncrementalReportData {
  return {
    schemaVersion: 3,
    status: "passed",
    fixture: "privacy-sentinel",
    fixtureRevision: "test",
    corpus: { vueFiles: 1, vueAndTypeScriptFiles: 1, baselineDiagnostics: 0 },
    budget: {
      scale: 1,
      suite: "privacy-sentinel",
      laneHardTimeoutMs: 1_000,
      maxPeakRssMiB: 1_000,
      maxPeakProcessTreeRssMiB: 1_000,
      maxProcessTreeSize: 10,
      laneBudgetsMs: { [label]: 1_000 },
    },
    timingsMs: { [label]: 1 },
    processTreeAttributionSetupMs: 1.5,
    processTreeAttributionMs: { [label]: 2 },
    sampledPeakRssKiB: sample.members.find((member) => member.role === "root")?.rssKiB ?? 0,
    sampledPeakProcessTreeRssKiB: sample.totalKiB,
    sampledPeakProcessTreeSize: sample.processes,
    sampledPeakProcessTree: { label, ...sample, memberRssSumKiB: sample.totalKiB },
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
  const privateArguments = [
    "--token=must-not-leak",
    "https://private.invalid/session/must-not-leak",
    "/private/customer/must-not-leak",
  ];
  const child = spawn(
    process.execPath,
    ["-e", "setInterval(() => {}, 1000)", ...privateArguments],
    { stdio: "ignore" },
  );
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
    assert.equal(root.identity.executableKind, "node");
    assert.match(root.identity.executableSha256, /^[0-9a-f]{64}$/);
    assert.match(root.identity.argvSha256, /^[0-9a-f]{64}$/);
    assert.equal(root.identity.argvCount, 6);
    assert.equal(root.identity.runtimeRole, "other");
    const secondSample = processTreeRss(child.pid, {
      stableIdentity: true,
      expectedRootBirthTicks: root.identity.birthTicks,
    });
    assert.ok(secondSample != null);
    const secondRoot = secondSample.members.find((member) => member.role === "root");
    assert.equal(secondRoot?.identity?.executableSha256, root.identity.executableSha256);
    assert.equal(secondRoot?.identity?.argvSha256, root.identity.argvSha256);
    assert.equal(
      processTreeRss(child.pid, {
        stableIdentity: true,
        expectedRootBirthTicks: `${root.identity.birthTicks}0`,
      }),
      null,
      "the first stable sample rejects a root PID whose birth identity changed",
    );
    const outputDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-process-identity-"));
    try {
      writeIncrementalArtifacts(outputDir, "Process identity privacy", reportData(sample));
      const metricsJson = fs.readFileSync(path.join(outputDir, "metrics.json"), "utf8");
      const summary = fs.readFileSync(path.join(outputDir, "summary.md"), "utf8");
      for (const privateArgument of privateArguments) {
        assert.ok(!metricsJson.includes(privateArgument));
        assert.ok(!summary.includes(privateArgument));
      }
      assert.equal(JSON.parse(metricsJson).schemaVersion, 3);
      assert.equal(JSON.parse(metricsJson).processTreeAttributionSetupMs, 1.5);
      assert.doesNotMatch(metricsJson, /"argv"\s*:/);
      assert.doesNotMatch(metricsJson, /resolvedExecutable/);
      assert.ok(!metricsJson.includes(process.execPath));
      assert.match(metricsJson, /"argvSha256"\s*:/);
      assert.match(
        summary,
        /Stable process attribution ran outside lane timings: setup 1\.5 ms; max sample 2\.0 ms/,
      );
    } finally {
      fs.rmSync(outputDir, { recursive: true, force: true });
    }
  } finally {
    child.kill();
    await Promise.race([once(child, "exit"), delay(1_000)]);
  }
});

test("Linux proc parsing binds first-sample ancestry, birth, and RSS custody", () => {
  const before = parseLinuxProcStat(procStat(41, 7, "100", 12), 4);
  assert.deepEqual(before, { pid: 41, parentPid: 7, birthTicks: "100", rssKiB: 48 });
  assert.deepEqual(
    [
      sameLinuxProcessAnchor(before, parseLinuxProcStat(procStat(41, 7, "101", 12), 4)),
      sameLinuxProcessAnchor(before, parseLinuxProcStat(procStat(41, 8, "100", 12), 4)),
    ],
    [false, false],
    "birth or parent changes must reject first-sample identity custody",
  );
  const changedRss = parseLinuxProcStat(procStat(41, 7, "100", 13), 4);
  assert.equal(
    sameLinuxProcessCustody(before, changedRss),
    true,
    "RSS may change while argv and executable hashes are captured",
  );
  assert.equal(before.rssKiB, 48, "the attributed sample keeps the anchored before RSS");
  assert.throws(() => parseLinuxProcStat(procStat(41, 7, "100", -1), 4), /invalid Linux/);
  assert.throws(() => parseLinuxProcStat(procStat(41, "1e2", "100", 12), 4), /invalid Linux/);
  assert.throws(() => parseLinuxProcStat(procStat(41, 7, "100", "0x10"), 4), /invalid Linux/);
  assert.throws(() => parseLinuxProcStat("41 (truncated) S 7", 4), /invalid Linux/);
});

test("closed runtime roles match production tokens without prefix capture", () => {
  assert.equal(classifyRuntimeRole("vize", ["/private/vize", "lsp"]), "native-lsp");
  assert.equal(classifyRuntimeRole("vize", ["/private/vize", "api"]), "native-api");
  assert.equal(classifyRuntimeRole("other", ["/private/tsgo", "--lsp"]), "typescript-lsp");
  assert.equal(classifyRuntimeRole("other", ["/private/tsgo", "--api"]), "typescript-api");
  assert.equal(classifyRuntimeRole("vize", ["/private/vize", "lsp-private"]), "other");
  assert.equal(classifyRuntimeRole("node", ["/private/node", "lsp"]), "other");
});

test("executable hash cache reuses exact metadata and invalidates changed files", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-executable-hash-"));
  const executable = path.join(root, "private-project-name");
  let reads = 0;
  const readFile = (fileDescriptor: number) => {
    reads += 1;
    return fs.readFileSync(fileDescriptor);
  };
  try {
    fs.writeFileSync(executable, "first");
    const firstFd = fs.openSync(executable, "r");
    const firstStat = fs.fstatSync(firstFd, { bigint: true });
    const firstHash = cachedExecutableSha256(firstFd, firstStat, readFile);
    fs.closeSync(firstFd);

    const cachedFd = fs.openSync(executable, "r");
    const cachedHash = cachedExecutableSha256(
      cachedFd,
      fs.fstatSync(cachedFd, { bigint: true }),
      readFile,
    );
    fs.closeSync(cachedFd);
    assert.equal(cachedHash, firstHash);
    assert.equal(reads, 1, "an exact dev/ino/size/mtime/ctime identity reuses its hash");

    fs.writeFileSync(executable, "second-longer");
    const changedFd = fs.openSync(executable, "r");
    const changedHash = cachedExecutableSha256(
      changedFd,
      fs.fstatSync(changedFd, { bigint: true }),
      readFile,
    );
    fs.closeSync(changedFd);
    assert.notEqual(changedHash, firstHash);
    assert.equal(reads, 2, "changed executable metadata invalidates the hash cache");
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

test("closed executable kinds and Markdown escaping do not retain hostile basenames", () => {
  const hostile = "private|<script>`[leak](https://private.invalid)";
  assert.equal(classifyExecutableKind(`/customer/${hostile}`), "other");
  const outputDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-markdown-custody-"));
  try {
    const sample = attributedSample(41, 400);
    const data = reportData(sample, hostile);
    data.fixture = hostile;
    data.fixtureRevision = hostile;
    writeIncrementalArtifacts(outputDir, hostile, data);
    const summary = fs.readFileSync(path.join(outputDir, "summary.md"), "utf8");
    assert.ok(!summary.includes("<script>"));
    assert.ok(!summary.includes("[leak](https://private.invalid)"));
    assert.ok(!summary.includes("private|"));
  } finally {
    fs.rmSync(outputDir, { recursive: true, force: true });
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

test("stable process attribution retries the complete ambiguous snapshot a fixed number", () => {
  const witness = attributedSample(41, 400);
  let attempts = 0;
  assert.equal(
    retryStableProcessTreeSample(() => {
      attempts += 1;
      return attempts === 3 ? witness : null;
    }),
    witness,
  );
  assert.equal(attempts, 3);

  attempts = 0;
  assert.equal(
    retryStableProcessTreeSample(() => {
      attempts += 1;
      return null;
    }),
    null,
  );
  assert.equal(attempts, 3, "persistent ambiguity stays fail-closed after the fixed retry bound");
});

test("sampling ambiguity cannot replace a lane or hard-timeout error", async () => {
  const metrics = new IncrementalMetrics(process.pid, suite);
  const budget = mutableBudget(metrics);
  budget.laneHardTimeoutMs = 1;
  budget.laneBudgetsMs = { probe: 1_000 };
  metrics.sampleRss = () => {
    assert.ok(
      (metrics as unknown as { timingsMs: Record<string, number> }).timingsMs.probe >= 0,
      "lane timing is finalized before attribution sampling starts",
    );
    throw new Error("sampling failed");
  };
  await assert.rejects(
    metrics.measure("probe", () => new Promise<never>(() => {})),
    (error: Error) => {
      assert.match(error.message, /lane "probe" is still not settled/);
      assert.doesNotMatch(error.message, /sampling failed/);
      return true;
    },
  );

  const successful = new IncrementalMetrics(process.pid, suite);
  mutableBudget(successful).laneBudgetsMs = { probe: 1_000 };
  successful.sampleRss = (name) => metrics.sampleRss(name);
  await assert.rejects(
    successful.measure("probe", async () => 42),
    /sampling failed/,
  );
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
