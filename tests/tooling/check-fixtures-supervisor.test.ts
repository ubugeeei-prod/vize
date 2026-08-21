import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { CHECK_FIXTURE_ENV, CHECK_FIXTURE_NODE_ARGS } from "./support/check-fixtures/manifest.ts";
import { LEAK_PID_FILE_ENV } from "./support/check-fixtures/control-fixtures.ts";
import {
  classifyPhase,
  describePhase,
  phaseEnv,
  runPhase,
} from "./support/check-fixtures/phase-runner.ts";
import { isCorsaRuntimeCommand } from "./support/check-fixtures/cycle-runner.ts";
import { readProcessTable } from "./support/check-fixtures/process-table.ts";
import { REPORT_SCHEMA } from "./support/check-fixtures/report.ts";
import { settleGuardedTasks } from "./support/check-fixtures/supervised-command.ts";
import { runSupervisor } from "./support/check-fixtures/supervisor.ts";
import { root } from "./support/github-workflows.ts";

const FIXTURE_DIR = "tests/tooling/support/check-fixtures";

function phaseOptions(env: NodeJS.ProcessEnv = {}) {
  return {
    capture: true,
    cwd: root,
    env: { ...process.env, ...CHECK_FIXTURE_ENV, ...env },
    nodeArgs: CHECK_FIXTURE_NODE_ARGS,
    sampleIntervalMs: 25,
    startedAt: performance.now(),
  };
}

function pidIsLive(pid: number): boolean {
  return readProcessTable().some((task) => task.pid === pid && task.state !== "Z");
}

function tempFile(name: string): string {
  return path.join(fs.mkdtempSync(path.join(os.tmpdir(), "vize-check-fixtures-")), name);
}

// The negative control. A phase that passes every assertion it makes and still
// abandons a `node` child has to fail the lane, or the supervisor would be a
// telemetry collector rather than a guard. The leaked child is spawned into the
// phase's process group and then unreferenced, so by the time the phase returns
// `init` owns it and no parent link remains — exactly the case that makes a
// descendant-only guard blind.
test("the descendant guard turns red when a phase leaks a node child", async () => {
  const pidFile = tempFile("leaked.pid");
  const outcome = await runPhase(
    { file: `${FIXTURE_DIR}/leaked-child-fixture.ts`, id: "leaked-child" },
    phaseOptions({ [LEAK_PID_FILE_ENV]: pidFile }),
  );

  assert.equal(outcome.exitCode, 0, `the leaking phase must pass its own tests: ${outcome.output}`);
  assert.equal(outcome.signal, null);
  assert.equal(outcome.spawnError, null);
  // A phase that silently ran nothing would leak nothing and look green, so the
  // control proves it executed before it judges the guard.
  assert.ok(
    fs.existsSync(pidFile),
    `the phase must actually run its fixture; its output was:\n${outcome.output}`,
  );
  const leakedPid = Number.parseInt(fs.readFileSync(pidFile, "utf8"), 10);
  assert.ok(Number.isInteger(leakedPid));

  assert.ok(
    outcome.survivors.some((survivor) => survivor.pid === leakedPid),
    `the guard must report pid ${leakedPid}; it reported ${JSON.stringify(outcome.survivors)}` +
      `, phase pgid ${String(outcome.pgid)}, and the live row for that pid is ` +
      `${JSON.stringify(readProcessTable().find((task) => task.pid === leakedPid) ?? null)}`,
  );
  assert.equal(outcome.status, "leaked");
  assert.deepEqual(
    [...new Set(outcome.survivors.map((task) => task.command))],
    ["node"],
    "only the leaked node process should be reported",
  );
  for (const survivor of outcome.survivors) {
    assert.equal(survivor.pgid, outcome.pgid, "a survivor is found through its process group");
  }
  assert.equal(outcome.reaped, true, "the guard must clean up after recording the leak");
  assert.match(
    describePhase(outcome),
    /^leaked leaked-child \(\d+ms, peak group tasks \d+\): live node pid=\d+ ppid=\d+ pgid=\d+/,
  );
  for (const survivor of outcome.survivors) {
    assert.equal(pidIsLive(survivor.pid), false, `pid ${survivor.pid} should have been reaped`);
  }
});

// The positive control. Without it the negative control would only prove the
// guard can be red, not that a phase which reaps its own children stays green.
test("the descendant guard stays quiet when a phase reaps its children", async () => {
  const outcome = await runPhase(
    { file: `${FIXTURE_DIR}/clean-child-fixture.ts`, id: "clean-child" },
    phaseOptions(),
  );

  assert.equal(outcome.exitCode, 0, `the clean phase must pass: ${outcome.output}`);
  assert.equal(outcome.status, "passed");
  assert.deepEqual(outcome.survivors, []);
  assert.equal(outcome.reaped, false);
  assert.equal(outcome.samples.after.group.processes.length, 0);
});

// A `tsgo` that exits with its phase is still in the table for a moment, first
// running and then as a zombie until it is reaped. Judging on the instant the
// phase closed failed the lane for those corpses, so the guard waits for the
// group to settle. What it must not do is settle away a task that only left
// because it was killed, which is the leak it exists to catch.
test("a task that exits on its own is not a leak, and one that stays is", async () => {
  const leaked = { command: "tsgo", pgid: 4242, pid: 4243, ppid: 1, state: "Z", threads: 1 };
  let reads = 0;
  const exiting = await settleGuardedTasks(4242, 1, () => {
    reads += 1;
    return reads === 1 ? [leaked] : [];
  });
  assert.deepEqual(exiting.survivors, [], "a task already on its way out is not a leak");
  assert.ok(reads > 1, "the guard must re-read rather than trust the first sample");

  const staying = await settleGuardedTasks(4242, 1, () => [leaked], 30);
  assert.deepEqual(
    staying.survivors.map((record) => record.pid),
    [4243],
    "a task that outlasts the settle window is still reported",
  );
  assert.deepEqual(staying.records, [leaked], "the artifact reads the same table as the verdict");
});

// The lane never meets these because it starts from a task runner, but the
// guard's own tests supervise a phase from inside another `node --test`, and a
// child that inherits the outer run's context exits zero without executing
// anything — a green phase that checked nothing.
test("a phase does not inherit another runner's context", () => {
  const stripped = phaseEnv({
    NODE_CHANNEL_FD: "3",
    NODE_OPTIONS: "--disable-warning=DEP0040",
    NODE_TEST_CONTEXT: "child-v8",
    NODE_UNIQUE_ID: "1",
    NODE_V8_COVERAGE: "/tmp/coverage",
    PATH: "/usr/bin",
    VIZE_TEST_REQUIRE_TSGO: "1",
  });
  assert.deepEqual(stripped, {
    NODE_OPTIONS: "--disable-warning=DEP0040",
    PATH: "/usr/bin",
    VIZE_TEST_REQUIRE_TSGO: "1",
  });
});

test("phase classification separates assertion failures, leaks, and spawn failures", () => {
  const survivor = {
    command: "tsgo",
    pgid: 10,
    pid: 11,
    ppid: 1,
    state: "S",
    threads: 4,
  };
  assert.equal(
    classifyPhase({ exitCode: 0, signal: null, spawnError: null, survivors: [] }),
    "passed",
  );
  assert.equal(
    classifyPhase({ exitCode: 0, signal: null, spawnError: null, survivors: [survivor] }),
    "leaked",
  );
  assert.equal(
    classifyPhase({ exitCode: 1, signal: null, spawnError: null, survivors: [] }),
    "failed",
  );
  assert.equal(
    classifyPhase({ exitCode: null, signal: "SIGKILL", spawnError: null, survivors: [] }),
    "failed",
  );
  // An `EAGAIN` from `spawn` is reported, never retried and never swallowed.
  assert.equal(
    classifyPhase({
      exitCode: null,
      signal: null,
      spawnError: "Error: spawn EAGAIN",
      survivors: [survivor],
    }),
    "spawn-failed",
  );
});

test("Corsa runtime process names cover preview and TypeScript 7 stable binaries", () => {
  for (const command of ["tsgo", "corsa", "tsc", "TSC"]) {
    assert.equal(isCorsaRuntimeCommand(command), true, command);
  }
  assert.equal(isCorsaRuntimeCommand("node"), false);
});

test("the supervisor artifact records the whole process budget for every phase", async () => {
  const metricsDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-check-fixtures-"));
  const report = await runSupervisor({
    capture: true,
    cwd: root,
    metricsDir,
    phases: [
      { file: `${FIXTURE_DIR}/clean-child-fixture.ts`, id: "clean-child" },
      { file: `${FIXTURE_DIR}/leaked-child-fixture.ts`, id: "leaked-child" },
    ],
    sampleIntervalMs: 25,
  });

  const verdicts = report.phases.map((phase) => `${phase.id}:${phase.status}`);
  const outputs = report.phases.map((phase) => `${phase.id}:\n${phase.output}`).join("\n");
  assert.equal(report.status, "failed", `a leaked phase must fail the whole lane\n${outputs}`);
  assert.deepEqual(
    verdicts,
    ["clean-child:passed", "leaked-child:leaked"],
    `a failing phase must not stop later phases from being recorded\n${outputs}`,
  );

  const written = JSON.parse(
    fs.readFileSync(path.join(metricsDir, "topology.json"), "utf8"),
  ) as typeof report;
  assert.equal(written.schema, REPORT_SCHEMA);
  assert.equal(written.supervisorPid, process.pid);
  assert.equal(written.phases.length, 2);
  assert.equal(typeof written.runner.cpuCount, "number");
  assert.ok(written.runner.cpuCount > 0);
  assert.ok(
    written.runner.ulimitProcesses !== undefined,
    "`ulimit -u` must be recorded even when the platform reports it as unlimited",
  );
  assert.ok("current" in written.runner.cgroupPids, "cgroup pids.current must be recorded");
  assert.ok("max" in written.runner.cgroupPids, "cgroup pids.max must be recorded");

  for (const phase of written.phases) {
    assert.deepEqual(
      Object.keys(phase.samples).sort(),
      ["after", "before", "peak"],
      `${phase.id} must carry a before, peak, and after sample`,
    );
    for (const label of ["before", "peak", "after"] as const) {
      const sample = phase.samples[label];
      assert.ok(sample.tasks.length > 0, `${phase.id} ${label} must carry the live task tree`);
      assert.ok(sample.liveTasks >= sample.tasks.length, "live tasks include every thread");
      assert.equal(typeof sample.runner.cpuCount, "number");
    }
    assert.equal(phase.samples.before.group.pgid, null);
    assert.equal(phase.samples.after.group.pgid, phase.pgid);
  }

  const summary = fs.readFileSync(path.join(metricsDir, "summary.md"), "utf8");
  assert.match(summary, /## Vue parity fixture process topology/);
  assert.match(summary, /\| clean-child \| passed \|/);
  assert.match(summary, /\| leaked-child \| leaked \|/);
  assert.match(summary, /`ulimit -u`/);
  assert.match(summary, /cgroup `pids\.current` \/ `pids\.max`/);
  fs.rmSync(metricsDir, { force: true, recursive: true });
});
