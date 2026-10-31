import assert from "node:assert/strict";
import { test } from "node:test";
import { contract, verifyProfileOutputs } from "./support/jsx-child-profile-contract.ts";

function fixtures() {
  const before = Array.from({ length: 16 }, (_, index) => ({
    source:
      index === 12 || index === 13
        ? "const Child={}; original static experiment"
        : `unchanged${index}`,
    ssr: false,
    map: false,
    output: { code: "unchanged", diagnostics: [], map: null },
  }));
  const after = structuredClone(before);
  for (const row of [...contract.rows, ...contract.existingStaticRows]) {
    before[row.index] = {
      source: row.source,
      ssr: row.ssr,
      map: row.map,
      output: structuredClone(row.before) as (typeof before)[number]["output"],
    };
    after[row.index] = {
      source: row.source,
      ssr: row.ssr,
      map: row.map,
      output: structuredClone(row.after) as (typeof after)[number]["output"],
    };
  }
  return { before, after };
}

test("child profile guard admits only the four complete observed output transitions", () => {
  const { before, after } = fixtures();
  assert.deepEqual(verifyProfileOutputs(before, after), [0, 1, 8, 9]);
});
test("child profile guard refuses unrelated outputs, diagnostics, inputs and broader populations", () => {
  for (const mutate of [
    (rows: ReturnType<typeof fixtures>["after"]) => {
      rows[0].output.code += "foreign";
    },
    (rows: ReturnType<typeof fixtures>["after"]) => {
      rows[2].output.code += "foreign";
    },
    (rows: ReturnType<typeof fixtures>["after"]) => {
      rows[12].output.code += "foreign";
    },
    (rows: ReturnType<typeof fixtures>["after"]) => {
      rows[13].output.diagnostics.length = 1;
    },
    (rows: ReturnType<typeof fixtures>["after"]) => {
      rows[0].output.diagnostics.length = 1;
    },
    (rows: ReturnType<typeof fixtures>["after"]) => {
      rows[8].source += "edited";
    },
    (rows: ReturnType<typeof fixtures>["after"]) => {
      rows.pop();
    },
    (rows: ReturnType<typeof fixtures>["after"]) => {
      rows[0].ssr = true;
    },
  ]) {
    const { before, after } = fixtures();
    mutate(after);
    assert.throws(() => verifyProfileOutputs(before, after), assert.AssertionError);
  }
});
