import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import os from "node:os";
import { gunzipSync } from "node:zlib";
import { test } from "node:test";
import {
  captureOracle,
  assertOracle,
  corpusRoot,
  readCorpus,
  sha256,
} from "./support/bound-slot-attribute-oracle.mjs";

await test("whole full51 official Vue-base packets retain every independent finding and metadata", async () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-bound-slot-oracle-"));
  const rawPath = path.join(dir, "raw.jsonl");
  const capture = await captureOracle({
    benchmarkManifest: process.env.VIZE_TEST_BOUND_SLOT_PROVIDER_MANIFEST,
    onRecorded(packet) {
      fs.appendFileSync(rawPath, JSON.stringify(packet) + "\n");
    },
  });
  fs.writeFileSync(path.join(dir, "capture.json"), JSON.stringify(capture, null, 2) + "\n");
  const expected = JSON.parse(fs.readFileSync(path.join(corpusRoot, "oracle.json"), "utf8"));
  assertOracle(capture, expected);
  assert.equal(capture.captures.length, 92);
});

await test("the actual before/after packets add exactly the declared bound finding", () => {
  const before = JSON.parse(fs.readFileSync(path.join(corpusRoot, "native-before.json"), "utf8"));
  const after = JSON.parse(fs.readFileSync(path.join(corpusRoot, "native-after.json"), "utf8"));
  assert.equal(before.sourceRevision, "26e56ac6a0de3f9f838db55bbdb71757317f2435");
  assert.equal(after.sourceRevision, `${before.sourceRevision}+bound-slot-working-tree`);
  assert.equal(before.executionStatus, "completed");
  assert.equal(after.executionStatus, "completed");
  assert.equal(before.completedCases, 46);
  assert.equal(after.completedCases, 46);
  let added = 0;
  for (const input of readCorpus().cases) {
    assert.equal(sha256(input.source), input.sourceSha256, input.id);
    const b = before.cases.find((c) => c.id === input.id);
    const a = after.cases.find((c) => c.id === input.id);
    assert.equal(b.source, input.source);
    assert.equal(a.source, input.source);
    assert.equal(b.filename, input.filename);
    assert.equal(a.filename, input.filename);
    assert.equal(a.observations.length, 2);
    assert.equal(b.observations.length, 2);
    assert.deepEqual(a.observations[0], a.observations[1]);
    assert.deepEqual(b.observations[0], b.observations[1]);
    assert.deepEqual(a.observations[0], input.native);
    if (input.addedTargets === 0) {
      assert.deepEqual(a.observations[0], b.observations[0], `${input.id}: unchanged whole packet`);
      continue;
    }
    assert.equal(input.addedTargets, 1);
    const namedDelta = structuredClone(a.observations[0]);
    const index = namedDelta.diagnostics.findIndex(
      (d) =>
        d.rule_name === "vue/no-deprecated-slot-attribute" &&
        input.source.slice(d.start, d.end) === input.newBoundKey,
    );
    assert.notEqual(index, -1, input.id);
    namedDelta.diagnostics.splice(index, 1);
    namedDelta.error_count -= 1;
    assert.deepEqual(namedDelta, b.observations[0], `${input.id}: only the one named addition`);
    added += 1;
  }
  assert.equal(added, 22);
});

await test("the original 40 packets and fresh baseline custody remain intact", () => {
  const read = (name) => JSON.parse(fs.readFileSync(path.join(corpusRoot, name), "utf8"));
  const original = read("original-40/cases.json");
  assert.deepEqual(readCorpus().cases.slice(0, 40), original.cases);
  for (const phase of ["before", "after"]) {
    assert.deepEqual(
      read(`native-${phase}.json`).cases.slice(0, 40),
      read(`original-40/native-${phase}.json`).cases,
    );
  }
  const oracle = read("oracle.json");
  assert.deepEqual(
    { ...oracle, captures: oracle.captures.slice(0, 80) },
    read("original-40/oracle.json"),
  );
  const custody = read("before-custody.json");
  const inventory = gunzipSync(
    fs.readFileSync(path.join(corpusRoot, "before-source-inventory.json.gz")),
  );
  assert.equal(sha256(inventory), custody.beforeSourceInventorySha256);
  assert.equal(
    sha256(fs.readFileSync(path.join(corpusRoot, "before-observer.rs"))),
    custody.driverSha256,
  );
  assert.equal(
    sha256(gunzipSync(fs.readFileSync(path.join(corpusRoot, "before-build.txt.gz")))),
    custody.buildLogSha256,
  );
  assert.equal(read("native-before.receipt.json").binarySha256, custody.beforeReceipt.binarySha256);
  for (const [name, entry] of Object.entries(read("manifest.json").files)) {
    const bytes = fs.readFileSync(path.join(corpusRoot, name));
    assert.equal(bytes.length, entry.bytes, name);
    assert.equal(sha256(bytes), entry.sha256, name);
  }
});

await test("the complete executed explain assertion keeps its actual right packet", () => {
  const read = (name) => gunzipSync(fs.readFileSync(path.join(corpusRoot, name))).toString("utf8");
  const log = read("explain-hosted-failure-log.txt.gz");
  const receipt = JSON.parse(
    fs.readFileSync(path.join(corpusRoot, "explain-assertion-correction-receipt.json")),
  );
  assert.equal(sha256(log), receipt.logSha256);
  const executed = (side) => {
    const line = log.split("\n").find((row) => row.includes(`      ${side}: `));
    assert.ok(line, side);
    return JSON.parse(line.slice(line.indexOf(`${side}: `) + side.length + 2));
  };
  const left = read("explain-executed-left-en.txt.gz");
  const right = read("explain-executed-right-en.txt.gz");
  assert.equal(left, executed("left"));
  assert.equal(right, executed("right"));
  assert.equal(sha256(left), receipt.leftSha256);
  assert.equal(sha256(right), receipt.rightSha256);
  const pages = (text) => text.split(/(?=^=== )/m).filter(Boolean);
  const a = pages(left),
    b = pages(right);
  assert.equal(a.length, 399);
  assert.equal(b.length, 399);
  assert.deepEqual(
    a.flatMap((page, index) => (page === b[index] ? [] : [page.split("\n")[0]])),
    ["=== vue/no-deprecated-slot-attribute"],
  );
});
