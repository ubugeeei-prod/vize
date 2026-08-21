import assert from "node:assert/strict";
import { test } from "node:test";

import {
  descendantsOf,
  liveTaskCount,
  parseProcStat,
  parsePsTable,
  readProcessTable,
  resolveTaskCommand,
  type TaskRecord,
  tasksInGroup,
} from "./support/check-fixtures/process-table.ts";
import {
  parseCgroupPath,
  parseUlimitProcesses,
  readRunnerFacts,
} from "./support/check-fixtures/runner-facts.ts";
import {
  describeSurvivor,
  GUARDED_COMMANDS,
  guardedSurvivors,
  relevantTasks,
  sampleTopology,
} from "./support/check-fixtures/topology.ts";

function task(overrides: Partial<TaskRecord> & { pid: number }): TaskRecord {
  return {
    command: "node",
    pgid: overrides.pid,
    ppid: 1,
    state: "S",
    threads: 1,
    ...overrides,
  };
}

// A `comm` is unquoted and may contain both spaces and parentheses, so a parser
// that split on the first `)` would read the wrong field for every one of the
// numbers the budget is judged from.
test("proc stat parsing survives a command name with spaces and parentheses", () => {
  const line =
    "1234 (my (weird) name) S 1 1200 1200 0 -1 4194368 100 0 0 0 1 2 0 0 20 0 7 0 999 0 0";
  assert.deepEqual(parseProcStat(line), {
    command: "my (weird) name",
    pgid: 1200,
    pid: 1234,
    ppid: 1,
    state: "S",
    threads: 7,
  });
});

test("proc stat parsing reports zombies and rejects malformed lines", () => {
  const zombie = parseProcStat("77 (tsgo) Z 1 55 55 0 -1 0 0 0 0 0 0 0 0 0 20 0 1 0 9 0 0");
  assert.equal(zombie?.state, "Z");
  assert.equal(zombie?.command, "tsgo");
  assert.equal(parseProcStat("not a stat line"), null);
  assert.equal(parseProcStat(""), null);
});

test("ps table parsing reduces executable paths to their base name", () => {
  const parsed = parsePsTable(
    ["  1     0     1 Ss   /sbin/launchd", " 42     1    42 S    /usr/local/bin/tsgo", "junk"].join(
      "\n",
    ),
  );
  assert.deepEqual(parsed, [
    { command: "launchd", pgid: 1, pid: 1, ppid: 0, state: "S", threads: null },
    { command: "tsgo", pgid: 42, pid: 42, ppid: 1, state: "S", threads: null },
  ]);
});

// `comm` names a *thread*, not a binary, and Node >= 24 writes `MainThread`
// over it through V8. Reading it verbatim made the guard blind to the one
// survivor these phases actually leak: every live `node` looked like an
// unguarded command, so a leaked checker was reported as a clean phase.
test("the live table names node tasks by their executable, not their thread", () => {
  const table = readProcessTable();
  assert.equal(
    table.find((record) => record.pid === process.pid)?.command,
    "node",
    "the reader must recognise itself as `node`",
  );
  assert.deepEqual(
    table.filter((record) => record.command === "MainThread"),
    [],
    "a runtime thread name must never reach the guard",
  );
  assert.equal(resolveTaskCommand(process.pid, "MainThread"), "node");
  assert.equal(resolveTaskCommand(process.pid, "tsgo"), "tsgo", "a real `comm` is left alone");
  // A zombie exposes neither `exe` nor `cmdline`, so the thread name is all
  // that is left, and it still has to resolve to a guarded command: an
  // unreaped task holds its slot in `RLIMIT_NPROC` and in `pids.current`.
  assert.equal(resolveTaskCommand(-1, "MainThread"), "node");
});

test("live task counting adds threads and falls back to one task per process", () => {
  assert.equal(liveTaskCount([task({ pid: 1, threads: 12 }), task({ pid: 2, threads: null })]), 13);
  assert.equal(liveTaskCount([]), 0);
});

test("descendants are transitive and process groups survive reparenting", () => {
  const records = [
    task({ pid: 100, pgid: 100, ppid: 1 }),
    task({ pid: 101, pgid: 100, ppid: 100 }),
    task({ pid: 102, pgid: 100, ppid: 101 }),
    // The orphan `init` adopted: no longer a descendant, still in the group.
    task({ pid: 103, pgid: 100, ppid: 1 }),
    task({ pid: 200, pgid: 200, ppid: 1 }),
  ];
  assert.deepEqual(
    descendantsOf(records, 100).map((record) => record.pid),
    [101, 102],
  );
  assert.deepEqual(
    tasksInGroup(records, 100).map((record) => record.pid),
    [100, 101, 102, 103],
  );
});

test("the guard reports leaked and zombie checkers and nothing else", () => {
  const records = [
    task({ command: "node", pgid: 500, pid: 501, ppid: 1 }),
    task({ command: "tsgo", pgid: 500, pid: 502, ppid: 1, state: "Z" }),
    task({ command: "vize", pgid: 999, pid: 503, ppid: 900 }),
    task({ command: "corsa", pgid: 500, pid: 504, ppid: 501 }),
    // Not a checker, and not the supervisor's business.
    task({ command: "ps", pgid: 500, pid: 505, ppid: 900 }),
    task({ command: "node", pgid: 500, pid: 900, ppid: 1 }),
  ];
  const survivors = guardedSurvivors(records, { pgid: 500, rootPid: 900 });
  assert.deepEqual(
    survivors.map((record) => `${record.command}:${record.pid}`),
    ["node:501", "tsgo:502", "vize:503", "corsa:504"],
    "group membership and direct descent are both checked, and the root is excluded",
  );
  assert.equal(describeSurvivor(survivors[1]!), "zombie tsgo pid=502 ppid=1 pgid=500, threads=1");
  assert.deepEqual(
    guardedSurvivors(records, { ignorePids: [501, 502, 503, 504], pgid: 500, rootPid: 900 }),
    [],
  );
  assert.deepEqual([...GUARDED_COMMANDS], ["node", "vize", "tsgo", "corsa", "tsc"]);
});

// The recorded tree has to reach a checker that escaped both the supervisor's
// parentage and its process group, or a leak could hide in the one place the
// artifact does not look. Unrelated system processes stay out, because 35
// phases times three samples of a whole runner is an artifact nobody opens.
test("the recorded tree keeps every checker and drops unrelated system processes", () => {
  const records = [
    task({ command: "supervisor", pgid: 1, pid: 900, ppid: 1 }),
    task({ command: "node", pgid: 500, pid: 501, ppid: 900 }),
    task({ command: "sh", pgid: 500, pid: 502, ppid: 501 }),
    // Escaped both relations: different group, reparented to init.
    task({ command: "tsgo", pgid: 777, pid: 503, ppid: 1 }),
    task({ command: "sshd", pgid: 300, pid: 300, ppid: 1 }),
  ];
  assert.deepEqual(
    relevantTasks(records, { pgid: 500, rootPid: 900 }).map((record) => record.pid),
    [501, 502, 503, 900],
  );
  assert.deepEqual(
    relevantTasks(records, { pgid: null, rootPid: 900 }).map((record) => record.pid),
    [501, 502, 503, 900],
  );
});

// Kept apart from the filtering test above: this one reads the live process
// table, so a failure here is about the running runner rather than about how
// recorded tasks are selected.
test("a sample taken without a group still roots the tree at the supervisor", () => {
  const sample = sampleTopology("before", { pgid: null, startedAt: performance.now() });
  assert.equal(sample.label, "before");
  assert.ok(
    sample.tasks.some((record) => record.pid === process.pid),
    "the sample must root the tree at the supervisor",
  );
  assert.ok(sample.liveTasks >= sample.tasks.length);
  assert.deepEqual(sample.group, { liveTasks: 0, pgid: null, processes: [] });
});

test("ulimit and cgroup readings distinguish unavailable from unlimited", () => {
  assert.equal(parseUlimitProcesses("63812\n"), 63812);
  assert.equal(parseUlimitProcesses(" unlimited \n"), "unlimited");
  assert.equal(parseUlimitProcesses("not-a-number"), null);
  assert.equal(parseCgroupPath("0::/user.slice/user-1000.slice\n"), "/user.slice/user-1000.slice");
  assert.equal(
    parseCgroupPath("12:memory:/docker/abc\n4:pids:/docker/abc\n0::/init.scope\n"),
    "/docker/abc",
  );
  assert.equal(parseCgroupPath("garbage"), null);
});

test("runner facts and the live table are readable on this platform", () => {
  const facts = readRunnerFacts();
  assert.ok(facts.cpuCount > 0);
  assert.match(facts.platform, /^[a-z0-9]+-[a-z0-9]+$/);
  assert.ok(
    facts.ulimitProcesses === null ||
      facts.ulimitProcesses === "unlimited" ||
      facts.ulimitProcesses > 0,
  );
  const table = readProcessTable();
  assert.ok(table.length > 0, "the live process table must not be empty");
  assert.ok(
    table.some((record) => record.pid === process.pid),
    "the reader must see itself",
  );
});
