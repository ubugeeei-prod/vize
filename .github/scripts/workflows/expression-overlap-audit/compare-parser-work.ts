import assert from "node:assert/strict";
import { join } from "node:path";
import { readJson, textFile } from "../shared/files.ts";

import { globSync, readFileSync } from "node:fs";
function findDirectory(role: string) {
  const matches = globSync("evidence/expression-overlap-" + role + "-*");
  assert.equal(matches.length, 1);
  return matches[0]!;
}
const base = findDirectory("baseline"),
  head = findDirectory("candidate");
function same(file: string) {
  assert.deepEqual(readFileSync(join(base, file)), readFileSync(join(head, file)), file);
}
same("cases/manifest.json");
same("private-source-facts.txt");
const cases = readJson<{ name: string; require_admission: boolean }[]>(
  join(base, "cases/manifest.json"),
);
for (const { name, require_admission } of cases) {
  same(name + ".packets.txt");
  if (require_admission) same(name + ".admission.json");
  else
    assert.equal(
      readJson<{ safe: boolean }>(join(head, name + ".admission.json")).safe,
      false,
      name,
    );
}
function instructions(dir: string, name: string) {
  const summary = /^summary: (\d+)/mu.exec(textFile(join(dir, name + ".callgrind")));
  assert.ok(summary, name);
  return Number(summary[1]);
}
function peak(dir: string, name: string) {
  return readJson<{ peak_arena_used_bytes: number }[]>(join(dir, name + ".timings.json"))[0]!
    .peak_arena_used_bytes;
}
for (const name of ["issue-10845.input", "10401-append-greater.input"]) {
  const before = instructions(base, name),
    after = instructions(head, name);
  assert.ok(before > after * 8, name);
  console.log(name, before, "->", after, "instructions");
  const baseMemory = peak(base, name),
    headMemory = peak(head, name);
  assert.ok(baseMemory > headMemory * 8, name);
  console.log(name, baseMemory, "->", headMemory, "arena bytes");
}
assert.ok(
  instructions(head, "minimal-overlap-256.input") <
    3 * instructions(head, "minimal-overlap-128.input"),
  "shared parser work is still superlinear",
);
for (const name of [
  "issue-10845.input",
  "issue-10845.min.input",
  "issue-10845.slow-1.input",
  "issue-10845.slow-2.input",
  "10401-append-greater.input",
])
  assert.equal(textFile(join(head, name + ".asan.exit")).trim(), "0", name);
console.log(
  "All",
  cases.length * 40,
  "complete packets match; required admission, rejected inputs and sanitizer budgets hold.",
);
