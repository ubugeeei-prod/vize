import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import { validateLoopScenario } from "./support/davinci-mounted-trace.mjs";

function fixture(name: string, extension: string) {
  return JSON.parse(
    fs.readFileSync(
      new URL(
        `../formal/impeto/fixtures/rust-lowered-loop-${name}.${extension}.json`,
        import.meta.url,
      ),
      "utf8",
    ),
  );
}

test("keyed and positional fixtures share updates but require different actual node lifetimes", () => {
  const keyed = fixture("keyed", "behavior");
  const unkeyed = fixture("unkeyed", "behavior");
  assert.deepEqual(fixture("keyed", "scenario"), fixture("unkeyed", "scenario"));
  assert.deepEqual(
    keyed.map(({ tree, events }) => ({ tree, events })),
    unkeyed.map(({ tree, events }) => ({ tree, events })),
  );
  assert.deepEqual(keyed[2].identities, [
    ["b", 2],
    ["a", 1],
  ]);
  assert.deepEqual(unkeyed[2].identities, [
    ["b", 1],
    ["a", 2],
  ]);
  assert.deepEqual(keyed[8].events, ["A", "A2", "A2", "C", "A3"]);
  for (const trace of [keyed, unkeyed]) {
    assert.deepEqual(trace[12].identities, [["a", 5]], "refill must not resurrect an old lifetime");
  }
});

test("every loop scenario has full observations and terminal cleanup", () => {
  for (const name of ["keyed", "unkeyed", "nested"]) {
    const scenario = fixture(name, "scenario");
    validateLoopScenario(scenario.context, scenario.steps);
    const trace = fixture(name, "behavior");
    assert.equal(trace.length, scenario.steps.length + 2);
    assert.deepEqual(trace.at(-1), { tree: [], events: trace.at(-2).events, identities: [] });
  }
  const nested = fixture("nested", "behavior");
  assert.deepEqual(
    nested[5].identities,
    [
      ["a", 9],
      ["c", 7],
      ["b", 3],
    ],
    "reparenting creates a lifetime",
  );
  assert.deepEqual(
    nested[8].identities,
    [
      ["b", 3],
      ["a", 11],
    ],
    "reintroduced parent creates descendants",
  );
});

test("mounted loop scripts reject fields the reference does not execute", () => {
  for (const context of [null, [], { record: null }, { $event: {} }, { constructor: "bad" }]) {
    assert.throws(() => validateLoopScenario(context, []), assert.AssertionError);
  }
  for (const step of [
    null,
    [],
    {},
    { ignored: true },
    { click: 1 },
    { click: "a", patch: {} },
    { patch: null },
    { patch: [] },
    { patch: { record: null } },
  ]) {
    assert.throws(() => validateLoopScenario({}, [step]), assert.AssertionError);
  }
});
