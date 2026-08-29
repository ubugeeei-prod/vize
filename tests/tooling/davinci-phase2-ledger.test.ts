import assert from "node:assert/strict";
import fs from "node:fs";
import { fileURLToPath } from "node:url";
import { test } from "node:test";

import {
  createCompatibilityContext,
  readCompatibilityLedger,
  validateCompatibilityLedger,
} from "../../tools/fixtures/fixture-compatibility-ledger.mjs";
import {
  p2_11CurrentRecordEvidence,
  recordsTaskRow,
  requiredLine,
  requiredSection,
} from "./support/davinci-phase2-ledger.ts";

function p2_11Installment(number: number): URL {
  return new URL(
    `../../davinci-road/plan/phase-2-records/p2-11/installment-${number}.md`,
    import.meta.url,
  );
}

const docs = {
  roadmap: new URL("../../davinci-road/roadmap.md", import.meta.url),
  readme: new URL("../../davinci-road/plan/README.md", import.meta.url),
  phase: new URL("../../davinci-road/plan/phase-2.md", import.meta.url),
  tasks: new URL("../../davinci-road/plan/phase-2-tasks.md", import.meta.url),
  tasksLater: new URL("../../davinci-road/plan/phase-2-tasks-later.md", import.meta.url),
  records: new URL("../../davinci-road/plan/phase-2-records.md", import.meta.url),
  p2_11: new URL("../../davinci-road/plan/phase-2-records/p2-11.md", import.meta.url),
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
const p2_11InstallmentNumbers = [
  20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38,
] as const;
const p2_11Installments = new Map(
  p2_11InstallmentNumbers.map((number) => [number, read(p2_11Installment(number))]),
);

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

function currentGroup(source: string, label: string) {
  const match = new RegExp(
    `^- \\*\\*${label}: (?<count>\\d+) of (?<total>\\d+) — (?<ids>P2-[\\s\\S]*?)\\.\\*\\*`,
    "mu",
  ).exec(source);
  assert.ok(match?.groups, `missing ${label} current-ledger group`);
  const ids = match.groups.ids.match(/P2-\d+(?:[ab])?/gu) ?? [];
  assert.equal(new Set(ids).size, ids.length, `${label} contains a duplicate task`);
  const declaredCount = Number(match.groups.count);
  assert.equal(ids.length, declaredCount, `${label} count does not match its exact set`);
  return { declaredCount, ids, total: Number(match.groups.total) };
}

function taskSection(source: string, id: string): string {
  const start = new RegExp(`^## ${id} —`, "mu").exec(source)?.index;
  if (start == null) throw new Error(`missing ${id} contract`);
  const tail = source.slice(start);
  const next = /^## P2-/mu.exec(tail.slice(1))?.index;
  return next == null ? tail : tail.slice(0, next + 1);
}

function dependencySet(source: string, id: string, taskIds: string[]): string[] {
  const section = taskSection(source, id);
  const raw = /\*\*Deps:\*\* (?<deps>[\s\S]*?) \*\*Non-goals:\*\*/u.exec(section)?.groups?.deps;
  assert.ok(raw, `missing ${id} dependency clause`);
  if (raw === "all of P2-1..P2-19.") return taskIds.filter((task) => task !== "P2-20");
  return raw.match(/P2-\d+(?:[ab])?/gu) ?? [];
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

test("Phase 2 current classification is exact, disjoint, and exhaustive", () => {
  const tasks = taskIndex(text.phase);
  const complete = currentGroup(text.phase, "Complete");
  const active = currentGroup(text.phase, "Active and blocked");
  const untouched = currentGroup(text.phase, "Untouched and dependency-blocked");
  const groups = [complete, active, untouched];
  assert.equal(tasks.size, 22);
  assert.deepEqual(complete.ids, completedTasks);
  assert.deepEqual(active.ids, activeTasks);
  assert.deepEqual(untouched.ids, untouchedTasks);
  for (const group of groups) assert.equal(group.total, tasks.size);

  for (let left = 0; left < groups.length; left += 1) {
    for (let right = left + 1; right < groups.length; right += 1) {
      assert.deepEqual(
        groups[left].ids.filter((id) => groups[right].ids.includes(id)),
        [],
        "current-ledger groups must be disjoint",
      );
    }
  }
  const covered = groups.flatMap((group) => group.ids);
  assert.equal(new Set(covered).size, tasks.size, "current-ledger coverage must not duplicate ids");
  assert.deepEqual(
    [...covered].sort(),
    [...tasks.keys()].sort(),
    "current-ledger groups must cover all 22 TODO ids",
  );
  for (const [id, checked] of tasks) assert.equal(complete.ids.includes(id), checked, id);

  for (const source of [text.roadmap, text.readme, text.phase, text.records]) {
    assertCurrentCount(source, completedTasks.length, tasks.size);
  }
});

test("dependency edges explain every untouched classification", () => {
  const taskIds = [...taskIndex(text.phase).keys()];
  const sources = new Map([
    ["P2-12b", text.tasks],
    ["P2-16", text.tasksLater],
    ["P2-17", text.tasksLater],
    ["P2-20", text.tasksLater],
  ]);
  const expected = new Map([
    ["P2-12b", ["P2-12a", "P2-11", "P2-3"]],
    ["P2-16", ["P2-11"]],
    ["P2-17", ["P2-11", "P2-12b", "P2-13"]],
    ["P2-20", taskIds.filter((id) => id !== "P2-20")],
  ]);
  const open = new Set([...activeTasks, ...untouchedTasks]);
  for (const id of untouchedTasks) {
    const dependencies = dependencySet(sources.get(id)!, id, taskIds);
    assert.deepEqual(dependencies, expected.get(id), `${id} dependency set drifted`);
    assert.ok(dependencies.every((dependency) => taskIds.includes(dependency)));
    assert.ok(!dependencies.includes(id), `${id} must not depend on itself`);
    assert.ok(
      dependencies.some((dependency) => open.has(dependency)),
      `${id} is not dependency-blocked by an open task`,
    );
  }
});

test("every completion joins a merged PR to honest current evidence", () => {
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
  assert.match(text.records, /current evidence/);
  const p2_19 = /^\| P2-19\s+\|[^\n]+$/mu.exec(text.records)?.[0] ?? "";
  assert.match(p2_19, /Review evidence/);
  assert.match(p2_19, /p2-19\.md/);
  assert.match(p2_19, /not a transport implementation witness/);
  assert.doesNotMatch(p2_19, /davinci-phase2-ledger/);
});

test("P2-11 records current installments without presenting stale remainders", () => {
  const currentEvidence = {
    roadmap: requiredSection(
      text.roadmap,
      /^\*\*Current execution ledger/mu,
      /^\*\*Exit gate:/mu,
      "roadmap current execution ledger",
    ),
    readme: requiredLine(text.readme, /^\| \[phase-2\.md\][^\n]+$/mu, "plan README phase 2 row"),
    tasks: requiredSection(
      text.tasks,
      /^\*\*Current series evidence/mu,
      /^\*\*Steps:\*\*/mu,
      "P2-11 current series evidence",
    ),
    records: recordsTaskRow(text.records, "P2-11"),
    p2_11: p2_11CurrentRecordEvidence(text.p2_11),
  };
  for (const [label, source] of Object.entries(currentEvidence)) {
    assert.match(source, /#5210/, `${label} must cite the current installment-38 PR`);
    assert.match(
      source,
      /38 (?:landed\s+)?installments|installment 38|\| 38\s+\|/i,
      `${label} must cite current installment 38 evidence`,
    );
    assert.doesNotMatch(source, /pending installment 38|\| 38\s+\| pending/i);
  }
  assert.match(text.p2_11, /#4933/);
  assert.match(text.p2_11, /#5011/);
  assert.match(text.p2_11, /#5178/);
  assert.match(text.p2_11, /#5183/);
  assert.match(text.p2_11, /#5198/);
  assert.match(text.p2_11, /#5200/);
  assert.match(text.p2_11, /#5203/);
  assert.match(text.p2_11, /#5205/);
  assert.match(text.p2_11, /#5207/);
  assert.match(text.p2_11, /#5210/);
  assert.match(text.p2_11, /#4919/);
  assert.match(text.p2_11, /#4921/);
  assert.match(text.p2_11, /#4924/);
  assert.match(text.p2_11, /#4927/);
  assert.match(text.p2_11, /#4929/);
  assert.match(text.p2_11, /Current named remainder \(after #5210\)/);
  assert.doesNotMatch(text.p2_11, /dynamic-argument bind names \/ modifiers/);
  assert.doesNotMatch(text.p2_11, /\*\*malformed slot fact gaps\*\*/);
  assert.match(p2_11Installments.get(20)!, /14-fixture S2-vs-shipped byte-for-byte battery/);
  assert.match(p2_11Installments.get(20)!, /does not tick P2-11/);
  assert.match(p2_11Installments.get(21)!, /Vue 2 pipe filters/);
  assert.match(p2_11Installments.get(22)!, /Vue 2 filter helper order/);
  assert.match(p2_11Installments.get(23)!, /Slot outlet same-name names/);
  assert.match(p2_11Installments.get(24)!, /Patch-flag matrix expansion/);
  assert.match(p2_11Installments.get(25)!, /Dynamic component patch flags/);
  assert.match(p2_11Installments.get(26)!, /Model listener patch order/);
  assert.match(p2_11Installments.get(27)!, /Dynamic component model arguments/);
  assert.match(p2_11Installments.get(28)!, /SFC style carriers are DOM-inert/);
  assert.match(p2_11Installments.get(29)!, /Bare Template Default Slots/);
  assert.match(p2_11Installments.get(30)!, /Inert Slot-Template Bindings/);
  assert.match(p2_11Installments.get(31)!, /Inline Slot-Template Carriers/);
  assert.match(p2_11Installments.get(31)!, /f5aa60553/);
  assert.match(p2_11Installments.get(32)!, /V-show Runtime Directives/);
  assert.match(p2_11Installments.get(32)!, /2be66b0f0/);
  assert.match(p2_11Installments.get(33)!, /V-html Raw HTML Props/);
  assert.match(p2_11Installments.get(33)!, /13cff4d99/);
  assert.match(p2_11Installments.get(34)!, /V-text Text-Content Props/);
  assert.match(p2_11Installments.get(34)!, /11750115a/);
  assert.match(p2_11Installments.get(35)!, /V-cloak DOM Cloak Markers/);
  assert.match(p2_11Installments.get(35)!, /02c4eb1a7/);
  assert.match(p2_11Installments.get(36)!, /Slot Outlet V-on Props/);
  assert.match(p2_11Installments.get(36)!, /cf7fc9a22/);
  assert.match(p2_11Installments.get(37)!, /Object V-bind Modifiers/);
  assert.match(p2_11Installments.get(37)!, /4e577b62/);
  assert.match(p2_11Installments.get(38)!, /Object V-on Modifiers/);
  assert.match(p2_11Installments.get(38)!, /f3959e7e3/);
});

test("suite registry debt and the TS-52 transport decision stay resolved", () => {
  const maximum = suiteMaximum(text.suites);
  assert.equal(maximum, 52);
  assertSuiteRange(text.readme, maximum);
  assert.match(text.suites, /^\| TS-25 \|[^\n]*P2-9[^\n]*P2-11[^\n]*P2-16/mu);
  assert.match(text.suites, /^\| TS-52 \|[^\n]*Spolvero feed payload/mu);
  assert.match(text.phase, /\*\*Registry maintenance this phase owes\*\*/);
  assert.match(text.phase, /P2-18 must add the entry in its own PR/);
  assert.match(text.phase, /Current resolution \(2026-08-25\): registry maintenance is resolved/);
  assert.match(text.phase, /recorded deviation from the re-cut's\s+own-PR condition/);
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
  for (const key of Object.keys(docs) as Array<keyof typeof docs>) {
    for (const match of text[key].matchAll(/\]\((?<target>[^)]+)\)/gu)) {
      const target = match.groups!.target.split("#", 1)[0];
      if (target === "" || /^[a-z]+:/u.test(target)) continue;
      const resolved = fileURLToPath(new URL(target, docs[key]));
      assert.ok(fs.existsSync(resolved), `${key} has a missing link: ${match.groups!.target}`);
    }
  }
});
