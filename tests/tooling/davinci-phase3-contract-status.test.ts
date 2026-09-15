import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const phasePath = path.join(repoRoot, "davinci-road", "plan", "phase-3.md");
const recordsRoot = path.join(repoRoot, "davinci-road", "plan", "phase-3-records");

const phase = fs.readFileSync(phasePath, "utf8");

function phaseTaskIndex(source: string = phase): Map<string, boolean> {
  const todoIndex = sectionBetween(source, /^## TODO index$/mu, /^---$/mu, "Phase 3 TODO index");
  const entries = [...todoIndex.matchAll(/^- \[(?<checked>[ x])\] (?<id>P3-\d+)\b/gmu)].map(
    (match) => [match.groups!.id, match.groups!.checked === "x"] as const,
  );
  assert.ok(entries.length >= 16, "Phase 3 TODO index must cover the task ladder");
  assert.equal(new Set(entries.map(([id]) => id)).size, entries.length, "duplicate P3 task id");
  return new Map(entries);
}

function phaseSection(id: string, source: string = phase): string {
  const start = new RegExp(`^\\*\\*${id} `, "mu").exec(source)?.index;
  assert.notEqual(start, undefined, `missing ${id} section`);
  const tail = source.slice(start);
  const next = /^\*\*P3-\d+ /mu.exec(tail.slice(1))?.index;
  return next == null ? tail : tail.slice(0, next + 1);
}

function recordIds(): string[] {
  return fs
    .readdirSync(recordsRoot)
    .filter((file) => /^p3-\d+\.md$/u.test(file))
    .map((file) => file.replace(/\.md$/u, "").toUpperCase())
    .sort((a, b) => numericId(a) - numericId(b));
}

function recordFiles(): string[] {
  return fs
    .readdirSync(recordsRoot)
    .filter((file) => /^p3-\d+\.md$/u.test(file))
    .sort((a, b) => numericId(recordFileId(a)) - numericId(recordFileId(b)));
}

function recordFileId(file: string): string {
  return file.replace(/\.md$/u, "").toUpperCase();
}

function numericId(id: string): number {
  return Number(/^P3-(?<number>\d+)$/u.exec(id)?.groups?.number);
}

function lineMentionsTask(line: string, id: string): boolean {
  const escaped = id.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&");
  return new RegExp(`(^|[^A-Z0-9-])${escaped}(?!\\d)`, "u").test(line);
}

function recordPath(id: string): string {
  return path.join(recordsRoot, `${id.toLowerCase()}.md`);
}

function recordLink(id: string): RegExp {
  return new RegExp(`phase-3-records/${id.toLowerCase()}\\.md`, "u");
}

function sectionBetween(source: string, start: RegExp, end: RegExp, label: string): string {
  const startMatch = start.exec(source);
  assert.ok(startMatch, `missing ${label}`);
  const tail = source.slice(startMatch.index);
  const endMatch = end.exec(tail.slice(startMatch[0].length));
  assert.ok(endMatch, `missing end marker for ${label}`);
  return tail.slice(0, startMatch[0].length + endMatch.index);
}

test("Phase 3 task index and record slices stay in sync", () => {
  const tasks = phaseTaskIndex();
  const records = new Set(recordIds());

  for (const id of records) {
    assert.ok(tasks.has(id), `${id} has a record file but no Phase 3 TODO entry`);
  }

  for (const [id, checked] of tasks) {
    const section = phaseSection(id);
    const hasRecord = records.has(id);

    if (checked) {
      assert.ok(hasRecord, `${id} is complete but has no record file`);
      assert.match(section, recordLink(id), `${id} must link its completion record`);
      assert.match(
        section,
        /(?:^|[^A-Za-z])(?:Landed|Closed)(?:[^A-Za-z]|$)/u,
        `${id} must state its terminal status`,
      );
      continue;
    }

    if (!hasRecord) {
      assert.doesNotMatch(section, recordLink(id), `${id} must not link a missing record`);
      continue;
    }

    const record = fs.readFileSync(recordPath(id), "utf8");
    assert.match(section, recordLink(id), `${id} slice must be linked from phase-3.md`);
    assert.match(
      `${section}\n${record}`,
      /\b(?:remain|remaining|remains|open)\b/iu,
      `${id} has a slice record but must still name its remaining work`,
    );
  }
});

test("completed Phase 3 tasks are not described as unfinished in sibling records", () => {
  const completedTasks = [...phaseTaskIndex()].filter(([, checked]) => checked).map(([id]) => id);
  const unfinishedTerms = [
    "does not close",
    "still needs",
    "still remains open",
    "remains open",
    "remaining work",
  ];

  for (const file of recordFiles()) {
    const fileId = recordFileId(file);
    const source = fs.readFileSync(path.join(recordsRoot, file), "utf8");
    const lines = source.split("\n");

    for (const completedId of completedTasks) {
      if (completedId === fileId) continue;

      for (const [lineIndex, line] of lines.entries()) {
        if (!lineMentionsTask(line, completedId)) continue;

        const saysUnfinished = unfinishedTerms.some((term) => line.toLowerCase().includes(term));
        assert.equal(
          saysUnfinished,
          false,
          `${file}:${lineIndex + 1} describes completed ${completedId} as unfinished`,
        );
      }
    }
  }
});
