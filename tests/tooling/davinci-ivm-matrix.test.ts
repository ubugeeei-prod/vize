import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const fixtureRoot = path.join(repoRoot, "formal/impeto/fixtures");

type Snapshot = { tree: unknown[]; events: string[]; identities: [string, number][] };
type Case = {
  name: string;
  template: string;
  scenario: { context: Record<string, unknown>; steps: Record<string, unknown>[] };
};

function jsonLines<T>(name: string): T[] {
  return fs
    .readFileSync(path.join(fixtureRoot, name), "utf8")
    .split("\n")
    .filter((line) => line.length > 0)
    .map((line) => JSON.parse(line) as T);
}

const cases = jsonLines<Case>("ivm-matrix.cases.jsonl");
const lowered = jsonLines<{ name: string; graph: string; values: string }>(
  "ivm-matrix.lowered.jsonl",
);
const behaviors = jsonLines<{ name: string; trace: Snapshot[] }>("ivm-matrix.behavior.jsonl");
const gaps = jsonLines<{ name: string; trace: Snapshot[] }>("ivm-matrix.vapor-gaps.jsonl");

test("TS-29 matrix is the full generated product with aligned artifacts", () => {
  const sources = ["array", "object", "range"].flatMap((kind) =>
    ["keyed", "positional"].map((identity) => `${kind}-${identity}`),
  );
  const expected = sources.flatMap((source) =>
    (source.startsWith("range") ? ["text", "toggle"] : ["text", "toggle", "button"]).flatMap(
      (body) => ["open", "guarded"].map((wrapper) => `${source}-${body}-${wrapper}`),
    ),
  );
  assert.deepEqual(
    cases.map((entry) => entry.name),
    expected,
  );
  assert.deepEqual(
    lowered.map((entry) => entry.name),
    expected,
  );
  assert.deepEqual(
    behaviors.map((entry) => entry.name),
    expected,
  );
  for (const entry of lowered) {
    assert.match(entry.graph, /^\[s3-folio\]\nphase=built\n/u);
    assert.match(entry.values, /^\[s3-values-folio\]\n/u);
  }
});

test("TS-29 reference traces are complete and exercise identity policies", () => {
  for (const [index, entry] of cases.entries()) {
    const trace = behaviors[index].trace;
    assert.equal(trace.length, entry.scenario.steps.length + 2, entry.name);
    assert.deepEqual(trace.at(-1)?.tree, [], `${entry.name} must unmount`);
    assert.deepEqual(trace.at(-1)?.identities, [], `${entry.name} must unmount`);
    const keyed = entry.name.includes("-keyed-");
    const identities = trace.map(
      (snapshot) => new Map(snapshot.identities.filter(([name]) => !name.startsWith("btn-"))),
    );
    // Step 2 reorders arrays, patches an object payload or grows a range.
    const [before, after] = [identities[1], identities[2]];
    for (const [name, lifetime] of before) {
      if (keyed && after.has(name)) assert.equal(after.get(name), lifetime, entry.name);
    }
    const positions = (snapshot: Map<string, number>) => [...snapshot.values()];
    if (!keyed && entry.name.startsWith("array")) {
      assert.deepEqual(positions(after), positions(before), `${entry.name} keeps positions`);
      assert.notDeepEqual([...after.keys()], [...before.keys()], `${entry.name} reorders`);
    }
    const hidden = trace.findIndex((snapshot) => snapshot.identities.length === 0);
    if (entry.name.endsWith("-guarded")) {
      assert.ok(hidden > 0, `${entry.name} must hide the guarded list`);
      const shown = trace[hidden + 1].identities.map(([, lifetime]) => lifetime);
      const earlier = trace.slice(0, hidden).flatMap((s) => s.identities.map(([, l]) => l));
      assert.ok(
        shown.every((lifetime) => lifetime > Math.max(...earlier)),
        `${entry.name} must recreate every element after the guard reopens`,
      );
    }
  }
});

test("TS-29 Vapor gaps are confined to the pinned upstream class", () => {
  assert.deepEqual(
    gaps.map((entry) => entry.name),
    cases.map((entry) => entry.name).filter((name) => name.startsWith("object-positional-")),
  );
  for (const gap of gaps) {
    const reference = behaviors.find((entry) => entry.name === gap.name)!.trace;
    assert.notDeepEqual(gap.trace, reference);
    const strip = (trace: Snapshot[]) =>
      JSON.parse(
        JSON.stringify(trace, (key, value) =>
          key === "data-id"
            ? "*"
            : key === "identities"
              ? value.map(([, l]: [string, number]) => l)
              : value,
        ),
      );
    assert.deepEqual(strip(gap.trace), strip(reference), `${gap.name} exceeds the gap class`);
  }
});
