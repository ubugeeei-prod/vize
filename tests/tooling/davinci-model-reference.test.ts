import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

type Entry = {
  name: string;
  template: string;
  trace: unknown[];
  scenario: { steps: Array<Record<string, unknown>>; slots: Record<string, object> };
};

function jsonLines(name: string): Entry[] {
  return fs
    .readFileSync(path.join(repoRoot, "formal/impeto/fixtures", name), "utf8")
    .split("\n")
    .filter((line) => line.length > 0)
    .map((line) => JSON.parse(line) as Entry);
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
      cases.some((entry) => entry.scenario.steps.some((step) => step.event === event)),
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

test("P3-4 supplied-slot reference artifacts are aligned and exercise supplied content", () => {
  const slotCases = jsonLines("slot-reference.cases.jsonl");
  const slotLowered = jsonLines("slot-reference.lowered.jsonl");
  const slotBehaviors = jsonLines("slot-reference.behavior.jsonl");
  const names = slotCases.map((entry) => entry.name);
  assert.deepEqual(
    slotLowered.map((entry) => entry.name),
    names,
  );
  assert.deepEqual(
    slotBehaviors.map((entry) => entry.name),
    names,
  );
  const specs = slotCases.flatMap((entry) => Object.values(entry.scenario.slots));
  assert.ok(
    specs.some((spec) => Object.hasOwn(spec, "text")),
    "static supplied text",
  );
  assert.ok(
    specs.some((spec) => Object.hasOwn(spec, "prop")),
    "displayed slot prop",
  );
  assert.ok(slotCases.some((entry) => Object.keys(entry.scenario.slots).length === 0));
  for (const [index, entry] of slotCases.entries()) {
    const trace = slotBehaviors[index].trace;
    assert.equal(trace.length, entry.scenario.steps.length + 2, entry.name);
    assert.deepEqual(trace.at(-1), { tree: [], events: [], identities: [] }, entry.name);
  }
});

test("P3-12 keyed model interactions route in-flight commits by identity", () => {
  const loopCases = jsonLines("model-loop-reference.cases.jsonl");
  const loopBehaviors = jsonLines("model-loop-reference.behavior.jsonl");
  assert.deepEqual(
    loopBehaviors.map((entry) => entry.name),
    loopCases.map((entry) => entry.name),
  );
  type Node = { tag: string; children: Array<Node | string> };
  const spans = (name: string, index: number) => {
    const out: string[] = [];
    const walk = (nodes: Array<Node | string>) => {
      for (const node of nodes) {
        if (typeof node === "string") continue;
        if (node.tag === "span") out.push(node.children.join(""));
        walk(node.children);
      }
    };
    const trace = loopBehaviors.find((entry) => entry.name === name)!.trace;
    walk((trace[index] as { tree: Node[] }).tree);
    return out;
  };
  // The IME commit (step 6) reaches item `a` when keyed, and whichever row now
  // occupies the composing element's position when positional.
  assert.deepEqual(spans("keyed-text-ime", 6), ["\u4e9c", "B1"]);
  assert.deepEqual(spans("positional-text-ime", 6), ["A", "\u4e9c"]);
  for (const entry of loopBehaviors) {
    assert.ok(
      (entry.trace as Array<{ identities?: unknown }>).every((s) => Array.isArray(s.identities)),
      `${entry.name} must observe element identities`,
    );
  }
});
