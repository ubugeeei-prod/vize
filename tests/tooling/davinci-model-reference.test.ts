import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

function jsonLines(name: string): Array<Record<string, any>> {
  return fs
    .readFileSync(path.join(repoRoot, "formal/impeto/fixtures", name), "utf8")
    .split("\n")
    .filter((line) => line.length > 0)
    .map((line) => JSON.parse(line));
}

const cases = jsonLines("model-reference.cases.jsonl");
const lowered = jsonLines("model-reference.lowered.jsonl");
const behaviors = jsonLines("model-reference.behavior.jsonl");

test("P3-4 model reference artifacts are aligned and cover every control family", () => {
  const names = cases.map((entry) => entry.name);
  assert.deepEqual(
    lowered.map((entry) => entry.name),
    names,
  );
  assert.deepEqual(
    behaviors.map((entry) => entry.name),
    names,
  );
  for (const family of ["text-", "checkbox-", "radio-", "select-", "member-", "recreated-"]) {
    assert.ok(
      names.some((name) => name.startsWith(family)),
      `missing ${family} model case`,
    );
  }
  for (const modifier of ["v-model.lazy", "v-model.trim", ".number", 'type="radio"', "multiple"]) {
    assert.ok(
      cases.some((entry) => entry.template.includes(modifier)),
      `missing ${modifier}`,
    );
  }
  for (const event of ["compositionstart", "compositionend", "input", "change"]) {
    assert.ok(
      cases.some((entry) => entry.scenario.steps.some((step: any) => step.event === event)),
      `missing ${event} step`,
    );
  }
});

test("P3-4 model traces observe live form state through unmount", () => {
  for (const [index, entry] of cases.entries()) {
    const trace = behaviors[index].trace;
    assert.equal(trace.length, entry.scenario.steps.length + 2, entry.name);
    assert.deepEqual(trace.at(-1), { tree: [], events: [] }, entry.name);
    const text = JSON.stringify(trace[0]);
    const live = entry.template.includes("<select") ? '"selected":' : '"checked":';
    assert.ok(text.includes(live), `${entry.name} must observe live form state`);
  }
});
