import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

export const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
export const fixture = path.join(root, "crates/vize_patina/tests/fixtures/issue-7976");
export const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");

export function frozenCorpus() {
  const source = JSON.parse(fs.readFileSync(path.join(fixture, "source.json"), "utf8"));
  assert.deepEqual(source.author, {
    login: "ubugeeei",
    id: 71201308,
    coauthor: "ubugeeei <71201308+ubugeeei@users.noreply.github.com>",
  });
  for (const row of [...source.inputs, ...source.authored]) {
    const bytes = fs.readFileSync(path.join(fixture, row.file));
    assert.equal(bytes.length, row.bytes);
    assert.equal(sha256(bytes), row.sha256);
  }
  const corpus = JSON.parse(fs.readFileSync(path.join(fixture, "cases.json"), "utf8"));
  assert.equal(corpus.cases.length, source.caseCount);
  assert.equal(source.caseCount, 47);
  const ids = new Set();
  for (const row of corpus.cases) {
    assert(!ids.has(row.id));
    ids.add(row.id);
    const bytes = Buffer.from(row.source);
    assert.equal(bytes.subarray(row.styleStart, row.styleEnd).toString().includes("display"), true);
    for (const start of row.expectedStarts)
      assert.equal(bytes.subarray(start, start + 7).toString(), "display");
    assert.equal(row.expectedCli[0].file, row.filename);
    assert.equal(row.expectedCli[0].warningCount, row.expectedStarts.length);
    assert.equal(row.expectedCli[0].messages.length, row.expectedStarts.length);
    assert.equal(row.expectedCli[0].errorCount, 0);
  }
  for (const issue of source.issues) {
    const bytes = fs.readFileSync(path.join(fixture, String(issue.number), "original-issue.md"));
    assert.equal(sha256(bytes), issue.bodySha256);
    const blocks = [...bytes.toString().matchAll(/```[^\n]*\n([\s\S]*?)```/g)];
    const inputs = source.inputs.filter((row) => row.issue === issue.number);
    assert.equal(inputs.length, 3);
    for (const [index, row] of inputs.entries())
      assert.equal(fs.readFileSync(path.join(fixture, row.file), "utf8"), blocks[index][1]);
    const original = corpus.cases.find((row) => row.id === `original-${issue.number}`);
    assert.equal(original.source, fs.readFileSync(path.join(fixture, inputs[1].file), "utf8"));
  }
  return { source, cases: corpus.cases };
}
