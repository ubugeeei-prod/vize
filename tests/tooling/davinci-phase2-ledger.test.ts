import assert from "node:assert/strict";
import fs from "node:fs";
import { fileURLToPath } from "node:url";
import { test } from "node:test";

import {
  createCompatibilityContext,
  readCompatibilityLedger,
  validateCompatibilityLedger,
} from "../../tools/fixtures/fixture-compatibility-ledger.mjs";

const docs = {
  roadmap: new URL("../../davinci-road/roadmap.md", import.meta.url),
  readme: new URL("../../davinci-road/plan/README.md", import.meta.url),
  phase: new URL("../../davinci-road/plan/phase-2.md", import.meta.url),
  tasks: new URL("../../davinci-road/plan/phase-2-tasks.md", import.meta.url),
  records: new URL("../../davinci-road/plan/phase-2-records.md", import.meta.url),
  p2_11: new URL("../../davinci-road/plan/phase-2-records/p2-11.md", import.meta.url),
  installment20: new URL(
    "../../davinci-road/plan/phase-2-records/p2-11/installment-20.md",
    import.meta.url,
  ),
  suites: new URL("../../davinci-road/plan/test-suites.md", import.meta.url),
  devtool: new URL("../../davinci-road/devtool.md", import.meta.url),
  questions: new URL("../../davinci-road/open-questions.md", import.meta.url),
} as const;

function read(url: URL): string {
  return fs.readFileSync(url, "utf8");
}

const text = Object.fromEntries(Object.entries(docs).map(([name, url]) => [name, read(url)])) as {
  [K in keyof typeof docs]: string;
};

const completedTasks = [
  "P2-1",
  "P2-2",
  "P2-3",
  "P2-4",
  "P2-5a",
  "P2-5b",
  "P2-6",
  "P2-7",
  "P2-8",
  "P2-10",
  "P2-12a",
  "P2-13",
  "P2-14",
  "P2-15",
  "P2-18",
  "P2-19",
];
const activeTasks = ["P2-9", "P2-11"];
const untouchedTasks = ["P2-12b", "P2-16", "P2-17", "P2-20"];

function taskIndex(source: string): Map<string, boolean> {
  const entries = [
    ...source.matchAll(/^- \[(?<checked>[ x])\] \[(?<id>P2-[^\]]+)\]\([^\n]+\)/gmu),
  ].map((match) => [match.groups!.id, match.groups!.checked === "x"] as const);
  assert.equal(new Set(entries.map(([id]) => id)).size, entries.length, "duplicate P2 task id");
  return new Map(entries);
}

function suiteMaximum(source: string): number {
  const ids = [...source.matchAll(/^\| TS-(?<id>\d+) \|/gmu)].map((match) =>
    Number(match.groups!.id),
  );
  assert.ok(ids.length > 0, "suite registry must not be empty");
  return Math.max(...ids);
}

function assertCurrentCount(source: string, completed: number, total: number): void {
  const expected = `${completed} of ${total}`;
  if (!source.includes(expected)) throw new Error(`stale task count: expected ${expected}`);
}

function assertSuiteRange(source: string, maximum: number): void {
  const expected = `TS-1..${maximum}`;
  if (!source.includes(expected)) throw new Error(`stale suite range: expected ${expected}`);
}

test("Phase 2 current counts are derived from the canonical TODO index", () => {
  const tasks = taskIndex(text.phase);
  assert.equal(tasks.size, 22);
  assert.deepEqual(
    [...tasks].filter(([, complete]) => complete).map(([id]) => id),
    completedTasks,
  );
  assert.deepEqual(
    [...tasks].filter(([, complete]) => !complete).map(([id]) => id),
    [...activeTasks, ...untouchedTasks],
  );

  for (const source of [text.roadmap, text.readme, text.phase, text.records]) {
    assertCurrentCount(source, completedTasks.length, tasks.size);
  }
  const active = /- \*\*Active and blocked:[\s\S]*?(?=\n- \*\*)/u.exec(text.phase)?.[0] ?? "";
  const untouched =
    /- \*\*Untouched and dependency-blocked:[\s\S]*?(?=\n- \*\*)/u.exec(text.phase)?.[0] ?? "";
  for (const id of activeTasks) assert.match(active, new RegExp(id));
  for (const id of untouchedTasks) assert.match(untouched, new RegExp(id));
});

test("every completion joins a merged PR to current executable evidence", () => {
  const expectedPrs = new Map([
    ["P2-1", "4452"],
    ["P2-2", "4452"],
    ["P2-3", "4452"],
    ["P2-4", "4496"],
    ["P2-5a", "4509"],
    ["P2-5b", "4509"],
    ["P2-6", "4509"],
    ["P2-7", "4502"],
    ["P2-8", "4544"],
    ["P2-10", "4642"],
    ["P2-12a", "4452"],
    ["P2-13", "4509"],
    ["P2-14", "4509"],
    ["P2-15", "4547"],
    ["P2-18", "4543"],
    ["P2-19", "4543"],
  ]);
  const rows = new Map(
    [...text.records.matchAll(/^\| (?<id>P2-[^ |]+)\s+\| \[#(?<pr>\d+)\]\([^\n]+$/gmu)].map(
      (match) => [match.groups!.id, match.groups!.pr],
    ),
  );
  assert.deepEqual(rows, expectedPrs);
  assert.match(text.records, /current executable evidence/);
});

test("P2-11 records installment 20 without presenting installment 19 as current", () => {
  for (const source of [text.roadmap, text.readme, text.tasks, text.records, text.p2_11]) {
    assert.match(source, /#4811/);
    assert.match(source, /20 (?:landed\s+)?installments|installment 20|\| 20\s+\|/i);
  }
  assert.match(text.p2_11, /Current named remainder \(after #4811\)/);
  assert.doesNotMatch(text.p2_11, /dynamic-argument bind names \/ modifiers/);
  assert.match(text.installment20, /14-fixture S2-vs-shipped byte-for-byte battery/);
  assert.match(text.installment20, /does not tick P2-11/);
});

test("suite registry debt and the TS-52 transport decision stay resolved", () => {
  const maximum = suiteMaximum(text.suites);
  assert.equal(maximum, 52);
  assertSuiteRange(text.readme, maximum);
  assert.match(text.suites, /^\| TS-25 \|[^\n]*P2-9[^\n]*P2-11[^\n]*P2-16/mu);
  assert.match(text.suites, /^\| TS-52 \|[^\n]*Spolvero feed payload/mu);
  assert.match(text.phase, /Registry maintenance is resolved in the current registry/);
  assert.match(text.devtool, /Decided: document over JSON-RPC/);
  assert.match(text.questions, /DevTool protocol[^\n]*Transport/);
});

test("corpus project counts come from the executable compatibility inventory", () => {
  const validated = validateCompatibilityLedger(
    readCompatibilityLedger(),
    createCompatibilityContext(),
  );
  const ecosystem = [...validated.fixtureMap.values()].filter((fixture) =>
    fixture.memberships.includes("ecosystem"),
  ).length;
  assert.equal(validated.fixtureMap.size, 146);
  assert.equal(ecosystem, 142);
  for (const source of [text.roadmap, text.readme, text.phase, text.records]) {
    assert.match(source, /146 gitlinks/);
    assert.match(source, /142 ecosystem\s+projects/);
  }
});

test("validator rejects a stale task count or suite range", () => {
  const tasks = taskIndex(text.phase);
  const maximum = suiteMaximum(text.suites);
  assert.throws(
    () => assertCurrentCount(text.readme.replace("16 of 22", "15 of 22"), 16, tasks.size),
    /stale task count: expected 16 of 22/,
  );
  assert.throws(
    () => assertSuiteRange(text.readme.replace("TS-1..52", "TS-1..51"), maximum),
    /stale suite range: expected TS-1\.\.52/,
  );
});

test("local links in the reconciled ledger exist", () => {
  for (const key of [
    "roadmap",
    "readme",
    "phase",
    "tasks",
    "records",
    "p2_11",
    "installment20",
    "suites",
  ] as const) {
    for (const match of text[key].matchAll(/\]\((?<target>[^)]+)\)/gu)) {
      const target = match.groups!.target.split("#", 1)[0];
      if (target === "" || /^[a-z]+:/u.test(target)) continue;
      const resolved = fileURLToPath(new URL(target, docs[key]));
      assert.ok(fs.existsSync(resolved), `${key} has a missing link: ${match.groups!.target}`);
    }
  }
});
