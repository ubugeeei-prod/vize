import assert from "node:assert/strict";
import { test } from "node:test";

import {
  budgetedArgv,
  outputFailures,
  resolveBudget,
  verifyBudget,
} from "./support/check-fixtures/cycle-runner.ts";
import { checkArgv, cycleTargets } from "./support/check-fixtures/cycle-targets.ts";
import {
  parseProcLimits,
  readUlimitProcesses,
  readUlimitProcessesHard,
} from "./support/check-fixtures/runner-facts.ts";

// The budget is what turns "stays within a documented peak" from a comparison
// into something the kernel enforces: exceed it and `clone` returns `EAGAIN`.
test("the constrained PID budget is the measured baseline plus documented headroom", () => {
  const echo = (budget: number) => budget;
  assert.deepEqual(resolveBudget(1000, 320, 63812, echo), {
    applied: 1320,
    baselineTasks: 1000,
    clamped: false,
    taskBudget: 320,
    ulimitProcesses: 1320,
  });
  assert.deepEqual(resolveBudget(1000, 320, "unlimited", echo), {
    applied: 1320,
    baselineTasks: 1000,
    clamped: false,
    taskBudget: 320,
    ulimitProcesses: 1320,
  });
  assert.deepEqual(resolveBudget(4000, 320, 4100, echo), {
    applied: 4100,
    baselineTasks: 4000,
    clamped: true,
    taskBudget: 320,
    ulimitProcesses: 4100,
  });
});

// `/bin/sh` is dash on Ubuntu, and dash's `ulimit` names `RLIMIT_NPROC` `-p`
// and rejects bash's `-u`. Trying one spelling silently ran every cycle
// unconstrained, so both are tried and neither working is an explicit exit.
test("the budgeted argv lowers the limit in either shell dialect, then execs in place", () => {
  assert.deepEqual(budgetedArgv(1320, "/bin/vize", ["check", "src"]), [
    "-c",
    'ulimit -u 1320 2>/dev/null || ulimit -p 1320 2>/dev/null || exit 97; exec "$@"',
    "sh",
    "/bin/vize",
    "check",
    "src",
  ]);
});

test("`/proc/self/limits` reports the soft and hard process ceilings", () => {
  const limits = [
    "Limit                     Soft Limit           Hard Limit           Units",
    "Max open files            1048576              1048576              files",
    "Max processes             472925               unlimited            processes",
    "",
  ].join("\n");
  assert.deepEqual(parseProcLimits(limits, "Max processes"), {
    hard: "unlimited",
    soft: 472925,
  });
  assert.deepEqual(parseProcLimits(limits, "Max stack size"), { hard: null, soft: null });
});

// The prologue returning zero only proves a builtin accepted the flag. This
// reads the limit back out of a child spawned through that same prologue, so a
// cycle can never claim a constraint the kernel never applied.
test("a budgeted child really runs under the requested process limit", () => {
  const hard = readUlimitProcessesHard();
  const soft = readUlimitProcesses();
  const headroom = typeof soft === "number" ? Math.max(soft - 1, 1) : 1024;
  const budget = typeof hard === "number" ? Math.min(headroom, hard) : headroom;
  assert.equal(verifyBudget(budget), budget);
});

// Reproducibility alone is not parity: a checker that degraded to zero files
// would produce the same hash every cycle. The corpus size and the fixture's
// authored diagnostics are asserted on their own.
test("a cycle rejects a check that degraded, sampled, or stopped reporting", () => {
  const clean = JSON.stringify({ errorCount: 26, fileCount: 18, files: [], warningCount: 0 });
  assert.deepEqual(outputFailures(clean, { diagnostics: true, fileCount: 18 }), []);
  assert.deepEqual(
    outputFailures(JSON.stringify({ errorCount: 26, fileCount: 4 }), {
      diagnostics: true,
      fileCount: 18,
    }),
    ["fileCount 4 != 18"],
  );
  assert.deepEqual(
    outputFailures(JSON.stringify({ errorCount: 0, fileCount: 18 }), {
      diagnostics: true,
      fileCount: 18,
    }),
    ["errorCount 0 must stay above zero"],
  );
  // A corpus without planted errors is pinned by the cycle hash, so its count
  // is free to move with unrelated parity work.
  assert.deepEqual(
    outputFailures(JSON.stringify({ errorCount: 3, fileCount: 42 }), {
      diagnostics: false,
      fileCount: 42,
    }),
    [],
  );
  assert.deepEqual(
    outputFailures(JSON.stringify({ errorCount: 3, fileCount: 41 }), {
      diagnostics: false,
      fileCount: 42,
    }),
    ["fileCount 41 != 42"],
  );
  assert.match(
    outputFailures("", { diagnostics: true, fileCount: 18 })[0] ?? "",
    /^check output is not JSON: /,
  );
});

test("the cycle targets pin one single-Corsa and one maximum-shard fixture", () => {
  assert.deepEqual(
    cycleTargets.map((target) => `${target.id}:${target.corsaProcesses}`),
    ["single-corsa:1", "maximum-shard:8"],
  );
  const [single, shard] = cycleTargets as [(typeof cycleTargets)[0], (typeof cycleTargets)[1]];
  assert.equal(single.projectDir, "tests/_fixtures/_projects/typecheck-errors");
  assert.equal(single.servers, null, "the 18-file fixture must keep auto-tuning to one process");
  assert.equal(single.peakGroupProcessBound, 2, "one vize process plus one Corsa process");
  assert.equal(single.expectedFileCount, 18, "the fixture the issue names has 18 Vue files");
  assert.equal(single.expectsDiagnostics, true, "typecheck-errors plants intentional errors");
  assert.equal(shard.projectDir, "tests/_fixtures/_git/create-vue");
  assert.equal(shard.servers, 8, "the shard cap is eight Corsa servers");
  assert.equal(shard.peakGroupProcessBound, 9, "one vize process plus eight Corsa processes");
  assert.equal(shard.expectedFileCount, 42, "the pinned create-vue submodule carries 42 templates");
  assert.equal(shard.expectsDiagnostics, false, "the create-vue corpus plants no errors");
  // The bound has to scale with the runner: a Corsa process sizes its Go
  // runtime from the core count, and every thread is a task in the budget.
  assert.equal(single.taskBudget(32), 320);
  assert.equal(shard.taskBudget(32), 1152);
  assert.ok(shard.taskBudget(32) > single.taskBudget(32));
  assert.deepEqual(checkArgv(single, "/bin/tsgo"), [
    "check",
    "src/**/*.vue",
    "--format",
    "json",
    "--quiet",
    "--corsa-path",
    "/bin/tsgo",
  ]);
  assert.deepEqual(checkArgv(shard, "/bin/tsgo"), [
    "check",
    "template/**/*.vue",
    "--format",
    "json",
    "--quiet",
    "--corsa-path",
    "/bin/tsgo",
    "--servers",
    "8",
  ]);
});
