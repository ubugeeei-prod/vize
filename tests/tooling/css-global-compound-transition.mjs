import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { root, sha256 } from "./css-external-target-reference.mjs";

export function compoundTransition(cases) {
  const fixture = path.join(root, "crates/vize_patina/tests/fixtures");
  const source = JSON.parse(
    fs.readFileSync(path.join(fixture, "issue-7976-compound/semantic-transitions.json"), "utf8"),
  );
  assert.equal(source.schema, "vize.css.global-compound.semantic-expectations");
  assert.equal(source.version, 1);
  assert.equal(
    sha256(fs.readFileSync(path.join(fixture, "issue-7976-global/cases.json"))),
    source.baselineCasesSha256,
  );
  assert.deepEqual(source.transitions.map((row) => row.before.id).sort(), [
    "global-compound-unknown",
    "outer-compound-unknown",
  ]);
  const overrides = new Map();
  for (const { before, after } of source.transitions) {
    assert.deepEqual(
      before,
      cases.find((row) => row.id === before.id),
      before.id,
    );
    assert.deepEqual(
      after,
      {
        ...before,
        expectedStarts: [],
        expectedCli: [{ file: before.filename, messages: [], errorCount: 0, warningCount: 0 }],
        expectedPlain: "Patina lint report: No problems found in 1 file(s)\n",
      },
      before.id,
    );
    overrides.set(before.id, after);
  }
  return cases.map((row) => overrides.get(row.id) ?? row);
}
