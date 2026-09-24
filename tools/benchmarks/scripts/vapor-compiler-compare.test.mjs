import assert from "node:assert/strict";
import { test } from "node:test";

import {
  compareThreeLanes,
  renderComparison,
  validateFixtures,
} from "./vapor-compiler-compare.mjs";

const fixtures = validateFixtures();
const exportFor = (names) => ({
  name: "head",
  benchmarks: Object.fromEntries(
    names.flatMap((name) =>
      ["s3", "legacy"].map((lane) => [
        `vapor_native_pair/${name}/${lane}`,
        { criterion_estimates_v1: { median: { point_estimate: lane === "s3" ? 9000 : 10000 } } },
      ]),
    ),
  ),
});

void test("three-compiler report requires the same complete fixture set", () => {
  const names = fixtures.map(({ name }) => name);
  const official = Object.fromEntries(names.map((name) => [name, 8000]));
  const rows = compareThreeLanes(exportFor(names), official);
  assert.equal(rows.length, 7);
  assert.equal(rows[0].s3OverRetained, 0.9);
  assert.equal(rows[0].s3OverOfficial, 1.125);
  assert.match(rows[0].sourceSha256, /^[0-9a-f]{64}$/);
  assert.match(
    renderComparison({
      headSha: "a".repeat(40),
      compilerVersion: "3.6.0-beta.10",
      runner: "test",
      rows,
    }),
    /text_runs.*0\.900x.*1\.125x/,
  );
  assert.throws(() => compareThreeLanes(exportFor(names.slice(1)), official), /fixture set/);
  delete official.text_runs;
  assert.throws(() => compareThreeLanes(exportFor(names), official), /complete shared corpus/);
});
