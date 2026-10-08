import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { frozenCorpus, root, sha256 } from "./css-external-target-reference.mjs";

export { root, sha256 };
export const fixture = path.join(root, "crates/vize_patina/tests/fixtures/issue-7976-global");

export function globalCorpus() {
  const historical = frozenCorpus();
  const source = JSON.parse(fs.readFileSync(path.join(fixture, "source.json"), "utf8"));
  assert.equal(source.issue, 7976);
  assert.deepEqual(source.author, historical.source.author);
  assert.equal(source.historyCaseCount, historical.cases.length);
  assert.equal(source.historyCliObservationCount, 218);
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
  assert.equal(source.caseCount, 66);
  assert.equal(source.initialCaseCount, 47);
  assert.equal(
    sha256(JSON.stringify(cases.slice(0, source.initialCaseCount))),
    source.initialCasesSha256,
  );
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
