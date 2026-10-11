import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { gunzipSync } from "node:zlib";
import { sha256, wholeJson } from "./lint-default-correctness-reference.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const fixture = path.join(root, "tests/_fixtures/differential/lint/bare-reserved-8090");
// These are the final frozen bytes after the source-confirmed help projection
// correction. Changing a receipt or oracle requires an explicit new adjudication.
const frozen = {
  "primary-sources.json": "104b87c2b4a31f36e666fb57dff5c29d6e762a8d4d64f80a58bde149b9d078f0",
  "observe-before.mjs": "e99f90b8276748938160b997a58f67015744652656efbdad6e7a9134426bc80a",
  "cases.json": "498ecc06838197424a3f59113a48c8bb45f9acd70cd7542f837607d5cd2a9131",
  "original-public-before.json": "7825b1445de7b064c759e354259af701854da9c2f8d4f9c41e1be16b49599b0e",
  "official-provider.json": "a49b9be27f4e67be45eb0d517f510c94631fd3cc5141dfa063cc3c4e11e57ed3",
  "original-matrix-packet.json.gz":
    "8dd2c90853351ff15bc180098bdcdf4e711276e59ee1c99a25fe7588b89b8cd2",
  "original-wave-ui-accordion.vue.fixture":
    "3312b54575de99e7026851bcbac68c7edef605374718ecd8a727f656e7d903d6",
  "original-wave-ui-accordion.json":
    "8e5e4ab869cacd0a746cb1907e6cb1f9ae170bb5dd65c6f59fe8c37d1420010a",
  "historical-method-bare.snap.fixture":
    "41d984e28428729c827b64c9d25f3acb9cf47b73289126aa7e2d341b68e36055",
  "historical-method-bare.ts.fixture":
    "11cda92fb09893f3dd3d67357273b04f30f9e6eb9c1d76d831349bb3e6a20410",
};
const read = (filename) => fs.readFileSync(path.join(fixture, filename));
const json = (filename) => JSON.parse(read(filename));

function nativePacket(run, entry) {
  const expected = wholeJson(entry, entry.beforeDiagnostics);
  assert.equal(run.id, entry.id);
  assert.equal(run.sourceSha256, entry.sourceSha256);
  assert.equal(run.matched, true);
  assert.equal(run.error, null);
  assert.equal(run.signal, null);
  assert.equal(run.stderr, "");
  assert.deepEqual(run.argv, [
    "lint",
    "-c",
    "vize.config.json",
    "--format",
    "json",
    "--locale",
    "en",
    "--help-level",
    "none",
    entry.filename,
  ]);
  const status = entry.beforeDiagnostics.length ? 1 : 0;
  assert.equal(run.expectedStatus, status);
  assert.equal(run.status, status);
  // Keep all fields, including help, counts, locations and message ordering.
  assert.deepEqual(run.expected, expected, entry.id);
  assert.deepEqual(run.actual, expected, entry.id);
  assert.deepEqual(JSON.parse(run.stdout), expected, entry.id);
}

function providerPacket(run, entry, directory) {
  const messages = entry.providerDiagnostics;
  const expected = [
    {
      filePath: path.posix.join(directory, entry.filename),
      messages,
      suppressedMessages: [],
      errorCount: messages.filter((finding) => finding.severity === 2).length,
      fatalErrorCount: 0,
      warningCount: messages.filter((finding) => finding.severity === 1).length,
      fixableErrorCount: 0,
      fixableWarningCount: 0,
      ...(messages.length ? { source: entry.source } : {}),
      usedDeprecatedRules: [],
    },
  ];
  assert.equal(run.id, entry.id);
  assert.equal(run.sourceSha256, entry.sourceSha256);
  assert.equal(run.matched, true);
  assert.equal(run.nativeAfterDiagnosticCount, entry.diagnostics.length);
  assert.equal(run.retainedProviderPolicyDifference, entry.diagnostics.length !== messages.length);
  assert.deepEqual(run.expected, expected, entry.id);
  assert.deepEqual(run.actual, expected, entry.id);
}

await test("bare reserved-name authority retains complete frozen references and prefix packets", () => {
  for (const [filename, digest] of Object.entries(frozen)) {
    assert.equal(sha256(read(filename)), digest, filename);
  }
  const primary = json("primary-sources.json");
  const corpus = json("cases.json");
  const native = json("original-public-before.json");
  const provider = json("official-provider.json");
  assert.equal(primary.schema, "vize.lint.bare-reserved-primary-sources");
  assert.equal(primary.version, 1);
  assert.equal(corpus.cases.length, 16);
  assert.deepEqual(corpus.removedBareNames, [
    "el",
    "data",
    "props",
    "attrs",
    "slots",
    "refs",
    "parent",
    "root",
    "options",
    "emit",
    "nextTick",
    "watch",
    "forceUpdate",
    "isMounted",
  ]);
  assert.equal(primary.provider.repositoryCommit, "9aa463a4aa4ad3c37d7facacda2e40ad7880ea92");
  assert.equal(primary.provider.npmGitHeadVerified, true);
  assert.equal(primary.primarySources.vueCommit, "8be32d654f5c99de2801d6815e0dc798fb1f7d00");
  assert.equal(primary.referenceContract.bareNamesReserved, false);
  assert.ok(
    corpus.removedBareNames.every((name) => !primary.referenceContract.reservedKeys.includes(name)),
  );
  assert.deepEqual(primary.referenceContract.underscoreGroups, ["data", "asyncData"]);
  assert.equal(primary.referenceContract.injectIncluded, false);
  assert.deepEqual(
    primary.referenceContract.retainedProviderDifferences,
    corpus.retainedProviderDifferences,
  );
  assert.deepEqual(provider.retainedProviderDifferences, corpus.retainedProviderDifferences);

  assert.equal(primary.observer.path, "observe-before.mjs");
  assert.equal(primary.observer.sha256, frozen[primary.observer.path]);
  assert.equal(
    primary.observer.wholeJson.path,
    "tests/tooling/lint-default-correctness-reference.mjs",
  );
  assert.equal(
    primary.observer.wholeJson.sha256,
    "d0fe46840f15d3aee0cad8b0394b4c67dd1f738a9ddaba39f55073fefd9df6b7",
  );
  assert.equal(
    sha256(fs.readFileSync(path.join(root, primary.observer.wholeJson.path))),
    primary.observer.wholeJson.sha256,
  );
  assert.equal(primary.corpus.path, "cases.json");
  assert.equal(primary.corpus.sha256, frozen[primary.corpus.path]);
  const custody = {
    casesSha256: frozen["cases.json"],
    primarySourcesSha256: frozen["primary-sources.json"],
    helperSha256: frozen["observe-before.mjs"],
    wholeJsonSha256: primary.observer.wholeJson.sha256,
    originalExpectationsSha256: frozen["original-wave-ui-accordion.json"],
  };
  assert.deepEqual(native.custody, custody);
  assert.deepEqual(provider.custody, custody);
  assert.deepEqual(provider.provider, primary.provider);
  assert.deepEqual(provider.configuration, primary.observer.providerConfig);
  assert.deepEqual(JSON.parse(native.config), primary.observer.nativeConfig);
  assert.deepEqual(primary.publicBefore, corpus.beforeBaseline);
  for (const [field, value] of Object.entries(corpus.beforeBaseline)) {
    assert.equal(native.baseline[field], value, field);
  }
  assert.equal(native.baseline.verifiedBinarySha256, corpus.beforeBaseline.binarySha256);

  const lock = primary.provider.packageLock;
  assert.equal(lock.encoding, "gzip-base64");
  const lockBytes = gunzipSync(Buffer.from(lock.completePacket, "base64"));
  assert.equal(lockBytes.length, lock.rawBytes);
  assert.equal(sha256(lockBytes), lock.sha256);
  assert.equal(lock.sha256, primary.provider.packageLockSha256);
  const packages = JSON.parse(lockBytes).packages;
  assert.deepEqual(
    primary.provider.packages.map(({ name, version }) => [name, version]),
    [
      ["eslint", "9.39.1"],
      ["eslint-plugin-vue", "10.9.2"],
      ["vue-eslint-parser", "10.4.1"],
    ],
  );
  for (const entry of primary.provider.packages) {
    const installed = packages[`node_modules/${entry.name}`];
    assert.equal(installed.version, entry.version);
    assert.equal(installed.resolved, entry.resolved);
    assert.equal(installed.integrity, entry.integrity);
    assert.match(entry.installedFileTreeSha256, /^[a-f0-9]{64}$/);
    assert.ok(entry.installedFileCount > 0);
  }

  assert.deepEqual(
    native.runs.map(({ id }) => id),
    corpus.cases.map(({ id }) => id),
  );
  assert.deepEqual(
    provider.runs.map(({ id }) => id),
    corpus.cases.map(({ id }) => id),
  );
  const directory = path.posix.dirname(provider.runs[0].expected[0].filePath);
  assert.match(path.posix.basename(directory), /^vize-bare-reserved-observation-/);
  for (const [index, entry] of corpus.cases.entries()) {
    const bytes = read(entry.filename + ".fixture");
    assert.equal(bytes.toString("utf8"), entry.source);
    assert.equal(sha256(bytes), entry.sourceSha256);
    assert.deepEqual(primary.corpus.cases[index], {
      id: entry.id,
      filename: entry.filename,
      sourceSha256: entry.sourceSha256,
    });
    for (const member of entry.members) {
      assert.equal(bytes.subarray(member.start, member.end).toString("utf8"), member.rawKey);
    }
    const prefixSpans = entry.members
      .filter((member) => member.name.startsWith("$") || member.name.startsWith("_"))
      .map((member) => [member.start, member.end]);
    const retained = entry.beforeDiagnostics.filter((finding) =>
      prefixSpans.some(([start, end]) => finding.start === start && finding.end === end),
    );
    // This compares entire API diagnostics: help, labels, fixes and byte ranges
    // remain identical to the before packet for every retained prefix member.
    assert.deepEqual(entry.diagnostics, retained, entry.id);
    assert.equal(retained.length, prefixSpans.length, entry.id);
    nativePacket(native.runs[index], entry);
    providerPacket(provider.runs[index], entry, directory);
  }
  assert.equal(
    corpus.cases.reduce((n, entry) => n + entry.beforeDiagnostics.length, 0),
    137,
  );
  assert.equal(
    corpus.cases.reduce((n, entry) => n + entry.diagnostics.length, 0),
    17,
  );
  assert.equal(
    corpus.cases.reduce((n, entry) => n + entry.providerDiagnostics.length, 0),
    9,
  );
  assert.deepEqual(native.summary, {
    cases: 16,
    diagnosticCount: 137,
    matchedCases: 16,
    realFileMatched: true,
    realFileDiagnosticCount: 2,
  });
  assert.deepEqual(provider.summary, {
    cases: 16,
    diagnosticCount: 9,
    matchedCases: 16,
    realFileMatched: true,
    realFileDiagnosticCount: 0,
    retainedProviderPolicyDifferenceCases: [
      "mixed-bare-and-prefix-data",
      "prefix-props-policy-control",
      "prefix-computed-methods-control",
      "prefix-setup-inject-control",
    ],
  });

  const original = json("original-wave-ui-accordion.json");
  original.source = read(original.sourceFile).toString("utf8");
  assert.equal(sha256(Buffer.from(original.source)), original.sourceSha256);
  assert.equal(original.sourceSha256, primary.realFile.sha256);
  assert.equal(primary.realFile.path, original.sourceFile);
  assert.equal(primary.realFile.bytes, Buffer.byteLength(original.source));
  assert.equal(primary.realFile.expectationsSha256, frozen[primary.realFile.expectationsPath]);
  assert.equal(original.revision, primary.realFile.revision);
  assert.equal(original.originalPath, primary.realFile.originalPath);
  assert.equal(original.immutableUrl, primary.realFile.url);
  assert.deepEqual(
    original.beforeDiagnostics.map(({ message, start, end }) => [message, start, end]),
    [
      ["Reserved key 'slots'", 10625, 10630],
      ["Reserved key 'props'", 10870, 10875],
    ],
  );
  assert.deepEqual(original.diagnostics, []);
  assert.deepEqual(original.providerDiagnostics, []);
  for (const receipt of [native, provider]) {
    const { observation, ...metadata } = receipt.realFile;
    assert.deepEqual(metadata, primary.realFile);
    if (receipt === native) nativePacket(observation, original);
    else providerPacket(observation, original, directory);
  }

  const matrixBytes = read(primary.matrix.path);
  assert.equal(sha256(matrixBytes), primary.matrix.sha256);
  assert.equal(matrixBytes.length, primary.matrix.bytes);
  const unpacked = gunzipSync(matrixBytes);
  assert.equal(sha256(unpacked), primary.matrix.uncompressedSha256);
  assert.equal(unpacked.length, primary.matrix.uncompressedBytes);
  const matrix = JSON.parse(unpacked);
  assert.equal(matrix.project, "wave-ui");
  assert.equal(matrix.revision, "a95a3bb1d65369de9befed6341bdc56bb5de075a");
  assert.equal(matrix.evidence.commitSha, "9466fdbb90bfae8803d5e57ae267b656be051721");
  assert.equal(matrix.files.comparedCount, 219);
  assert.equal(primary.matrix.comparedFiles, matrix.files.comparedCount);
  assert.equal(matrix.baseline.package, "eslint-plugin-vue");
  assert.equal(matrix.baseline.version, "10.9.2");
  assert.equal(matrix.divergence.falsePositives.length, 83);
  assert.equal(matrix.divergence.falseNegatives.length, 0);
  assert.ok(matrix.divergence.falsePositives.every(({ ruleId }) => ruleId === corpus.rule));
  assert.equal(primary.matrix.project, matrix.project);
  assert.equal(primary.matrix.revision, matrix.revision);
  assert.equal(primary.matrix.sourceCommit, matrix.evidence.commitSha);
  assert.equal(primary.matrix.falsePositiveCount, matrix.divergence.falsePositives.length);
  assert.equal(primary.matrix.falseNegativeCount, matrix.divergence.falseNegatives.length);

  const historical = json("historical-snapshot.json");
  const snapshot = read(historical.snapshotFile);
  const source = read(historical.sourceFile);
  assert.equal(sha256(snapshot), historical.snapshotSha256);
  assert.equal(sha256(source), historical.sourceSha256);
  const gitBlob = createHash("sha1")
    .update(`blob ${snapshot.length}\0`)
    .update(snapshot)
    .digest("hex");
  assert.equal(gitBlob, historical.originalGitBlob);
  assert.equal(gitBlob, "719d3074cb03a14071d24f0f1fa8f12808707e14");
  assert.equal(historical.sourceCommit, corpus.producer.sourceCommit);
  assert.equal(
    source.subarray(historical.sourceRange.start, historical.sourceRange.end).toString(),
    "emit",
  );
  assert.equal(historical.beforeMessage, "Reserved key 'emit'");
  assert.deepEqual(historical.afterDiagnostics, []);

  assert.equal(native.priorAttempts.length, 1);
  const prior = native.priorAttempts[0];
  const priorBytes = gunzipSync(Buffer.from(prior.completePacket, "base64"));
  assert.equal(prior.encoding, "gzip-base64");
  assert.equal(priorBytes.length, prior.rawBytes);
  assert.equal(
    sha256(priorBytes),
    primary.observer.projectionCorrection.initialNativeReceiptSha256,
  );
  assert.equal(sha256(priorBytes), prior.sha256);
  assert.deepEqual(JSON.parse(priorBytes).summary, prior.summary);
});
