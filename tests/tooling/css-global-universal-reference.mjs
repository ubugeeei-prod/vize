import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { root, sha256 } from "./css-external-target-reference.mjs";

export const fixture = path.join(root, "crates/vize_patina/tests/fixtures/issue-7976-universal");

export function universalCorpus() {
  const source = JSON.parse(fs.readFileSync(path.join(fixture, "source.json"), "utf8"));
  assert.equal(source.issue, 7976);
  assert.equal(source.actualParent, "807970136d3c0b45d173a525b0bdc8c86b7df291");
  assert.equal(source.historicalCaseCount, 182);
  assert.equal(source.parentNativeCaseCount, 16);
  assert.equal(source.historicalCliObservationCount, 858);
  assert.equal(source.compoundBaselineCaseCount, 62);
  assert.equal(source.semanticTransitionCount, 2);
  for (const row of source.history) {
    const bytes = fs.readFileSync(path.join(root, row.path));
    assert.equal(bytes.length, row.bytes, row.path);
    assert.equal(sha256(bytes), row.sha256, row.path);
  }
  for (const row of source.authored) {
    const bytes = fs.readFileSync(path.join(fixture, row.file));
    assert.equal(bytes.length, row.bytes, row.file);
    assert.equal(sha256(bytes), row.sha256, row.file);
  }
  const { cases } = JSON.parse(fs.readFileSync(path.join(fixture, "cases.json"), "utf8"));
  assert.equal(cases.length, source.caseCount);
  assert.equal(source.caseCount, 43);
  assert.equal(source.cliObservationCount, 215);
  assert.equal(new Set(cases.map((row) => row.id)).size, cases.length);
  for (const row of cases) {
    const bytes = Buffer.from(row.source);
    for (const start of [...row.expectedStarts, ...row.expectedCssStarts])
      assert.equal(bytes.subarray(start, start + 7).toString(), "display", row.id);
    assert.equal(row.expectedCli[0].file, row.filename);
    assert.equal(row.expectedCli[0].errorCount, 0);
    assert.equal(row.expectedCli[0].warningCount, row.expectedStarts.length);
    assert.equal(row.expectedCli[0].messages.length, row.expectedStarts.length);
    assert(row.expectedCssStarts.length >= row.expectedStarts.length, row.id);
  }
  return { source, cases };
}
