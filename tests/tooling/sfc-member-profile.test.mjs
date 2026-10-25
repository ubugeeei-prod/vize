import assert from "node:assert/strict";
import test from "node:test";
import { comparePackets } from "../../tools/benchmarks/scripts/sfc-member-profile.mjs";
import {
  memberCases,
  controlCases,
  percentiles,
} from "../../tools/benchmarks/scripts/sfc-member-fixtures.mjs";
const packet = () => ({
  rows: memberCases().map((vector) => ({
    id: vector.id,
    source: vector.source,
    sourceSha256: "hash",
    falseProfile: vector.expected,
    falseProfileBytes: JSON.stringify(vector.expected),
  })),
  controls: controlCases().map((vector) => ({
    id: vector.id,
    source: vector.source,
    falseProfile: vector.expected,
    falseProfileBytes: JSON.stringify(vector.expected),
  })),
});
test("whole false-profile metadata retains every field, literal name and refusal control", () => {
  const base = packet(),
    head = structuredClone(base);
  comparePackets(base, head);
  for (const mutate of [
    (value) => {
      value.rows[0].falseProfile.props.pop();
    },
    (value) => {
      value.rows[0].falseProfile.emits.push("invented");
    },
    (value) => {
      value.rows[0].falseProfile.extra = { lost: null, flag: false, count: 0 };
      value.rows[0].falseProfileBytes = JSON.stringify(value.rows[0].falseProfile);
    },
    (value) => {
      value.controls.at(-1).falseProfile.props[0].name = "invented";
    },
    (value) => {
      value.rows[0].source += " ";
    },
    (value) => {
      value.rows[0].sourceSha256 = "drift";
    },
    (value) => {
      value.rows.pop();
    },
    (value) => {
      value.controls.pop();
    },
  ]) {
    const altered = structuredClone(base);
    mutate(altered);
    assert.throws(() => comparePackets(base, altered));
  }
});
test("closed matrix covers every size/shape and invalid timing vectors fail closed", () => {
  assert.equal(memberCases().length, 18);
  assert.equal(new Set(memberCases().map((value) => value.id)).size, 18);
  for (const count of [32, 128, 512])
    assert.equal(memberCases().filter((value) => value.count === count).length, 6);
  assert.deepEqual(percentiles([4, 1, 3, 2]), { p50: 2, p95: 4, min: 1, max: 4 });
  for (const invalid of [[], [NaN], [Infinity], [-1]]) assert.throws(() => percentiles(invalid));
});
