import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import { gunzipSync } from "node:zlib";
import { sha256, wholeJson } from "./lint-default-correctness-reference.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const fixture = path.join(root, "tests/_fixtures/differential/lint/reserved-suppression-8090");
const read = (name) => fs.readFileSync(path.join(fixture, name));
const json = (name) => JSON.parse(read(name));

await test("registered alias preserves complete before packets and the original suppression oracle", () => {
  assert.equal(
    sha256(read("source-authority.json")),
    "c7ebe0b963ee0098354a0b6f87feda9accf43945e6a1f815f882ba75b2139593",
  );
  const authority = json("source-authority.json");
  for (const entry of authority.files) {
    assert.equal(sha256(read(entry.path)), entry.sha256, entry.path);
    assert.equal(read(entry.path).length, entry.bytes, entry.path);
  }
  assert.deepEqual(authority.scope, {
    coreCases: 24,
    beforeDiagnostics: 22,
    afterDiagnostics: 12,
    sourceRoutes: 7,
    sourceCliPackets: 169,
    changedOriginalCases: ["original-vue-select-src-components-select"],
    otherOriginalCases: 20,
    providerCoreRuns: 0,
    newPipelineStages: 0,
    otherRuleAliasesAdded: 0,
  });
  const producer = authority.producer;
  assert.equal(sha256(read(producer.before.fixture)), producer.before.sha256);
  assert.equal(read(producer.before.fixture).length, producer.before.bytes);
  // The live producer is shipped code; only the archived prior copy is pinned.
  assert(fs.existsSync(path.join(root, producer.path)), producer.path);

  const corpus = json("cases.json");
  const receipt = json("original-public-before.json");
  assert.equal(corpus.cases.length, 24);
  assert.deepEqual(receipt.baseline, corpus.beforeBaseline);
  assert.equal(receipt.runs.length, 24);
  assert.equal(receipt.baseline.tag, "v0.443.0");
  assert.equal(receipt.baseline.publicReleaseRun, 38081199501);
  assert.equal(
    receipt.baseline.binarySha256,
    "9ad7cf956351265a152a9bbfa08b8c50c85dadacef66529474bbb6170267e254",
  );
  for (const entry of receipt.custody) assert.equal(sha256(read(entry.path)), entry.sha256);
  let before = 0,
    after = 0;
  corpus.cases.forEach((entry, index) => {
    const bytes = read(entry.filename + ".fixture");
    assert.equal(bytes.toString(), entry.source);
    assert.equal(sha256(bytes), entry.sourceSha256);
    checkPublicPacket(receipt.runs[index], entry, entry.beforeDiagnostics);
    before += entry.beforeDiagnostics.length;
    after += entry.diagnostics.length;
    for (const finding of [...entry.beforeDiagnostics, ...entry.diagnostics]) {
      assert.equal(finding.rule_name, corpus.rule);
      assert.equal(finding.severity, "error");
      assert.equal(finding.fix, null);
      assert.equal(finding.labels.length, 1);
      assert.equal(finding.start, finding.labels[0].start);
      assert.equal(finding.end, finding.labels[0].end);
      const token = bytes.subarray(finding.start, finding.end).toString();
      assert.match(token, /^(?:_private|_hidden|_shown|\$emit)$/);
      assert.equal(finding.message, `Reserved key '${token}'`);
    }
  });
  assert.deepEqual({ before, after }, { before: 22, after: 12 });
  const initial = JSON.parse(gunzipSync(read("initial-twenty-two-before.json.gz")));
  assert.equal(initial.cases.cases.length, 22);
  assert.equal(initial.receipt.runs.length, 22);
  assert.deepEqual(initial.cases.cases, corpus.cases.slice(0, 22));
  initial.cases.cases.forEach((entry, index) => {
    checkPublicPacket(initial.receipt.runs[index], entry, entry.beforeDiagnostics);
  });
  assert.equal(
    initial.receipt.custody.find(({ path }) => path === "observe-public-before.mjs").sha256,
    sha256(Buffer.from(initial.observer)),
  );
  const byId = Object.fromEntries(corpus.cases.map((entry) => [entry.id, entry]));
  assert.equal(byId["alias-next-line"].beforeDiagnostics.length, 1);
  assert.deepEqual(byId["alias-next-line"].diagnostics, []);
  assert.equal(byId["canonical-disable-alias-enable"].beforeDiagnostics.length, 0);
  assert.equal(byId["canonical-disable-alias-enable"].diagnostics.length, 1);

  const oracle = json("original-suppression-oracle.json");
  assert.deepEqual(oracle.references, authority.originalReferences);
  for (const entry of oracle.references) {
    assert.equal(sha256(fs.readFileSync(path.join(root, entry.path))), entry.sha256);
  }
  const provider = json("../reserved-prefix-8090/official-provider.json");
  const publicBefore = json("../reserved-prefix-8090/original-public-before.json");
  const original = json("original-vue-select-successor.json");
  assert.deepEqual(
    oracle.providerRun,
    provider.runs.find(({ id }) => id === original.id),
  );
  assert.deepEqual(
    oracle.publicBeforeRun,
    publicBefore.runs.find(({ id }) => id === original.id),
  );
  assert.deepEqual(oracle.providerRun.actual, oracle.providerRun.expected);
  const official = oracle.providerRun.actual[0];
  assert.equal(official.messages.length, 2);
  assert.equal(official.warningCount, 2);
  assert.equal(official.fixableWarningCount, 2);
  assert.equal(official.errorCount, 0);
  assert.equal(official.suppressedMessages.length, 1);
  assert.deepEqual(official.suppressedMessages, original.providerSuppressedDiagnostics);
  assert.deepEqual(official.suppressedMessages[0].suppressions, [
    { kind: "directive", justification: "" },
  ]);
  const originalBytes = read(original.sourceFile);
  assert.equal(sha256(originalBytes), original.sourceSha256);
  assert.equal(originalBytes.length, 35649);
  assert.equal(official.source, originalBytes.toString());
  assert.deepEqual(oracle.publicBeforeRun.actual, oracle.publicBeforeRun.expected);
  assert.deepEqual(JSON.parse(oracle.publicBeforeRun.stdout), oracle.publicBeforeRun.expected);
  assert.equal(oracle.publicBeforeRun.status, 1);
  assert.equal(oracle.publicBeforeRun.stderr, "");
  assert.equal(oracle.publicBeforeRun.signal, null);
  assert.equal(oracle.publicBeforeRun.error, null);
  assert.equal(oracle.publicBeforeRun.actual[0].messages.length, 2);
  assert.equal(original.beforeDiagnostics.length, 1);
  assert.deepEqual(original.diagnostics, []);

  const inherited = json("../reserved-prefix-8090/originals.json");
  const successor = json("original-prefix-successors.json");
  assert.equal(successor.cases.length, 21);
  assert.equal(
    sha256(read("../reserved-prefix-8090/originals.json")),
    successor.originalCorpus.sha256,
  );
  const changed = [];
  successor.cases.forEach((entry, index) => {
    const old = inherited.cases[index];
    for (const key of ["id", "sourceSha256"]) assert.equal(entry[key], old[key]);
    if (entry.id === original.id) assert.deepEqual(entry.diagnostics, original.diagnostics);
    else assert.deepEqual(entry.diagnostics, old.diagnostics);
    if (JSON.stringify(entry.diagnostics) !== JSON.stringify(old.diagnostics))
      changed.push(entry.id);
  });
  assert.deepEqual(changed, [original.id]);
});

function checkPublicPacket(run, entry, diagnostics) {
  assert.equal(run.id, entry.id);
  assert.deepEqual(run.input, { text: entry.source, sha256: entry.sourceSha256 });
  assert.deepEqual(run.expected, wholeJson(entry, diagnostics));
  assert.deepEqual(run.actual, run.expected);
  assert.deepEqual(JSON.parse(run.stdout), run.expected);
  assert.equal(run.expectedStatus, diagnostics.length ? 1 : 0);
  assert.equal(run.status, run.expectedStatus);
  assert.equal(run.stderr, "");
  assert.equal(run.error, null);
  assert.equal(run.signal, null);
}
