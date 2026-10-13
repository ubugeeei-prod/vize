import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { gunzipSync } from "node:zlib";
import { sha256, wholeJson } from "./lint-default-correctness-reference.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const fixture = path.join(root, "tests/_fixtures/differential/lint/reserved-prefix-8090");
// Frozen source-authored authority; this test never invokes either lint observer.
const frozen = {
  "cases.json": "016061653fcd7781d87c2d542ee4d982a013e7059e1858ca9405cd589cae5d77",
  "originals.json": "65d7e2cbbdc5552f1d28338163d88506716fc8ebd6a2a88a0c5eab5670283144",
  "bare-parent-successors.json": "24f0a6e69dab5ae91929d0d2b4022de46d099048ba3782f8fd8c48480485ec84",
  "default-route-companions.json":
    "07267827ba3f3fc6cf789e6939a2b58f61758198be2527fa59ece5cae2470800",
  "bare-parent-route-successors.json":
    "54610b0976da6e8571d4e170cb4593bfdca4b2d8650025fa882b795f50b3aed8",
  "original-sources.json": "9fe2a2cbaf134f6896bd497befe5996e67383884d16d8650ba5752fc8df4d08e",
  "primary-sources.json": "4f63098b509aebb4bbc761002d1fb314574cfeb822e6ae2b58fbbd76f22a650a",
  "observe-before.mjs": "a8e7d018cc6a2f0cd166d5091a79326b4911261f5f1718047bbf531a7525579a",
  "original-public-before.json": "99033a79fa52c9b47e9def815d9367a25107acb4f41ee99bd4bbc89fa7dcdc6c",
  "official-provider.json": "25339fe5144eb1ba6e8d2a620af9912ea7882a6538c0d8743fb88424c18f7aee",
  "provider-supplement.json": "80f44968e6e4f799c65a6b5ad152838d33caab29e5d8eb28a4d85733f1ca034e",
  "initial-provider-expectations.json.gz":
    "2dac6c04fd0ab3f3490275f329bbee7a4286b4d07d173b353750515a40b2251d",
  "initial-observer.mjs.fixture":
    "b35bbb9994ed9999d484bdeb54c6d98b3f132d3944d2ad5eaf08175ead7ac0fe",
  "historical-computed-underscore.json":
    "47e34166163903438b37f7eaa2ea978bf7c03da24325af74bc555bc84f70a1fe",
  "historical-computed-underscore.snap.fixture":
    "0a8da7c3ca5c01a349be45f96c8a610b61f3b2287c229db5f3ce1eb54e5eb7fe",
  "historical-computed-underscore.ts.fixture":
    "d37d7479c65bddefaa55cce918e5976f6300271ebc0efb79d9c0fe0c2913d9db",
};
const read = (name) => fs.readFileSync(path.join(fixture, name));
const json = (name) => JSON.parse(read(name));
const eq = assert.deepEqual;
const is = assert.equal;
const idsOf = (entries) => entries.map(({ id }) => id);
const slice = (source, start, end) => Buffer.from(source).subarray(start, end).toString();
const counts = (entries, fields) => fields.split(" ").map((field) => count(entries, field));
const count = (entries, field) => entries.reduce((n, entry) => n + entry[field].length, 0);
function bytesMatch(bytes, metadata, hash = "sha256", size = "bytes") {
  is(sha256(bytes), metadata[hash]);
  if (metadata[size] !== undefined) is(bytes.length, metadata[size]);
}
function unpack(metadata) {
  is(metadata.encoding, "gzip-base64");
  const bytes = gunzipSync(Buffer.from(metadata.completePacket, "base64"));
  bytesMatch(bytes, metadata, "sha256", "rawBytes");
  return bytes;
}
function inputPacket(run, entry) {
  is(run.id, entry.id);
  is(run.kind, entry.id.startsWith("original-") ? "immutable-original" : "authored");
  eq(run.input, {
    source: entry.source,
    sourceSha256: entry.sourceSha256,
    bytes: Buffer.byteLength(entry.source),
  });
  is(run.inputAfterSha256, entry.sourceSha256);
}
function nativePacket(run, entry) {
  inputPacket(run, entry);
  const expected = wholeJson(entry, entry.publicBeforeDiagnostics);
  const argv = "lint -c vize.config.json --format json --locale en --help-level none";
  eq(run.argv, [...argv.split(" "), entry.filename]);
  is(run.matched, true);
  is(run.error, null);
  is(run.signal, null);
  is(run.stderr, "");
  is(run.expectedStatus, entry.publicBeforeDiagnostics.length ? 1 : 0);
  is(run.status, run.expectedStatus);
  eq(run.expected, expected, entry.id);
  eq(run.actual, expected, entry.id);
  eq(JSON.parse(run.stdout), expected, entry.id);
}
function providerExpected(entry, messages, directory) {
  const findings = (severity) => messages.filter((finding) => finding.severity === severity);
  return [
    {
      filePath: path.posix.join(directory, entry.filename),
      messages,
      suppressedMessages: entry.providerSuppressedDiagnostics ?? [],
      errorCount: findings(2).length,
      fatalErrorCount: 0,
      warningCount: findings(1).length,
      fixableErrorCount: findings(2).filter((finding) => finding.fix).length,
      fixableWarningCount: findings(1).filter((finding) => finding.fix).length,
      ...(messages.length ? { source: entry.source } : {}),
      usedDeprecatedRules: [],
    },
  ];
}

await test("reserved membership authority retains full references, initial failures and original Matrix bytes", () => {
  for (const [name, digest] of Object.entries(frozen)) is(sha256(read(name)), digest, name);
  const primary = json("primary-sources.json"),
    corpus = json("cases.json"),
    originals = json("originals.json");
  const index = json("original-sources.json"),
    successors = json("bare-parent-successors.json");
  const supplement = json("provider-supplement.json"),
    native = json("original-public-before.json"),
    provider = json("official-provider.json");
  const artifacts =
    "corpus originals successors originalSources providerSupplement initialProviderExpectations initialObserver";
  for (const key of artifacts.split(" ")) {
    bytesMatch(read(primary[key].path), primary[key]);
    is(primary[key].sha256, frozen[primary[key].path]);
  }
  const whole = primary.observer.wholeJson;
  bytesMatch(fs.readFileSync(path.join(root, whole.path)), whole);
  eq(primary.sourceParent, corpus.sourceParent);
  is(primary.sourceParent.commit, "30637987a17857bdb69a7a0f33b72d5cc36422d2");
  const reserved =
    "$data $props $el $options $parent $root $children $slots $scopedSlots $refs $isServer $attrs $listeners $watch $set $delete $on $once $off $emit $mount $forceUpdate $nextTick $destroy".split(
      " ",
    );
  eq(corpus.reservedDollarNames, reserved);
  eq(corpus.underscoreGroups, ["data"]);
  eq(corpus.excludedGroups, ["inject"]);
  eq(primary.referenceContract.unfinishedProviderCoverage, corpus.unfinishedProviderCoverage);
  const lock = JSON.parse(unpack(primary.provider.packageLock));
  is(lock.packages["node_modules/eslint-plugin-vue"].version, "10.9.2");
  for (const entry of primary.provider.packages) {
    const installed = lock.packages[`node_modules/${entry.name}`];
    for (const key of ["version", "resolved", "integrity"]) is(installed[key], entry[key]);
    const archive = primary.freshPackageArchives.find((item) => item.name === entry.name);
    is(archive.treeSha256, entry.installedFileTreeSha256);
    is(archive.files, entry.installedFileCount);
    for (const key of ["resolved", "integrity"]) is(archive[key], entry[key]);
  }
  const verified = {
    packages: primary.provider.packages.map((entry) => ({
      name: entry.name,
      files: entry.installedFileCount,
      treeSha256: entry.installedFileTreeSha256,
    })),
    lockSha256: primary.provider.packageLockSha256,
  };
  eq(provider.verifiedProviderBefore, verified);
  eq(provider.verifiedProviderAfter, verified);
  eq(provider.provider, primary.provider);
  eq(provider.configuration, primary.observer.providerConfig);
  bytesMatch(Buffer.from(native.config.text), native.config);
  eq(JSON.parse(native.config.text), primary.observer.nativeConfig);
  eq(primary.publicBefore, corpus.publicBeforeBaseline);
  for (const [key, value] of Object.entries(primary.publicBefore)) is(native.baseline[key], value);
  is(native.baseline.verifiedBinarySha256, primary.publicBefore.binarySha256);
  is(native.verifiedBinaryAfterSha256, primary.publicBefore.binarySha256);
  is(primary.inheritedBareCorpus.files.length, 27);
  for (const entry of primary.inheritedBareCorpus.files) bytesMatch(read(entry.path), entry);

  const entries = [...corpus.cases, ...originals.cases];
  const ids = idsOf(entries);
  eq(idsOf(native.runs), ids);
  eq(idsOf(provider.runs), ids);
  for (const entry of entries) {
    const bytes = read(entry.sourceFile ?? entry.filename + ".fixture");
    bytesMatch(bytes, entry, "sourceSha256");
    is(bytes.toString("utf8"), entry.source);
    for (const member of entry.members)
      is(bytes.subarray(member.start, member.end).toString(), member.rawKey);
    const spans = entry.members.filter(
      (member) =>
        member.descriptorSupported &&
        corpus.supportedGroups.includes(member.group) &&
        (reserved.includes(member.name) ||
          (member.group === "data" && member.name.startsWith("_"))),
    );
    const retained = entry.currentParentDiagnostics.filter((finding) =>
      spans.some((member) => member.start === finding.start && member.end === finding.end),
    );
    // Every retained finding preserves the whole original producer packet.
    eq(entry.diagnostics, retained, entry.id);
    is(entry.diagnostics.length, spans.length, entry.id);
  }
  eq(
    counts(entries, "currentParentDiagnostics publicBeforeDiagnostics diagnostics"),
    [264, 269, 153],
  );
  eq([count(corpus.cases, "diagnostics"), count(originals.cases, "diagnostics")], [152, 1]);
  const custody = {
    corpusSha256: frozen["cases.json"],
    originalsSha256: frozen["originals.json"],
    originalSourcesSha256: frozen["original-sources.json"],
    providerSupplementSha256: frozen["provider-supplement.json"],
    initialProviderExpectationsSha256: frozen["initial-provider-expectations.json.gz"],
    primarySourcesSha256: frozen["primary-sources.json"],
    helperSha256: frozen["observe-before.mjs"],
    wholeJsonSha256: whole.sha256,
  };
  eq(native.custody, custody);
  eq(provider.custody, custody);
  const corrected = new Map(supplement.cases.map((entry) => [entry.id, entry.providerDiagnostics]));
  const directory = path.posix.dirname(provider.runs[0].expected[0].filePath);
  assert.match(path.posix.basename(directory), /^vize-reserved-prefix-reference-/);
  entries.forEach((entry, position) => {
    nativePacket(native.runs[position], entry);
    const run = provider.runs[position],
      messages = corrected.get(entry.id) ?? entry.providerDiagnostics;
    inputPacket(run, entry);
    is(run.matched, true);
    is(run.nativeAfterDiagnosticCount, entry.diagnostics.length);
    is(run.providerParity, entry.providerParity);
    is(
      run.providerExpectationSource,
      corrected.has(entry.id) ? "provider-supplement.json" : "initial-authored-vector",
    );
    const expected = providerExpected(entry, messages, directory);
    eq(run.expected, expected, entry.id);
    eq(run.actual, expected, entry.id);
  });
  const summary = { cases: 41, matchedCases: 41, authoredCases: 20, originalCases: 21 };
  eq(native.summary, { ...summary, diagnosticCount: 269, suppressedDiagnosticCount: 0 });
  eq(provider.summary, { ...summary, diagnosticCount: 162, suppressedDiagnosticCount: 1 });

  eq(Object.values(supplement.correctionCounts), [1, 3, 7, 7, 0, 0]);
  for (const basis of supplement.sourceBasis) {
    const installed = primary.provider.files.find((entry) => entry.path === basis.path);
    is(basis.sha256, installed.sha256);
  }
  for (const change of supplement.cases) {
    const entry = entries.find((item) => item.id === change.id);
    for (const source of change.sourceDerivation) {
      is(source.sourceSha256, entry.sourceSha256);
      if (source.commentRange) {
        is(entry.source.slice(...source.commentRange), source.comment);
        const finding = change.providerDiagnostics.find(
          (item) => item.fix?.range[0] === source.commentRange[0],
        );
        eq(finding.fix, { range: source.commentRange, text: " " });
        is(finding.line, source.line);
        is(finding.ruleId, null);
        is(finding.severity, 1);
      } else {
        const text = slice(entry.source, source.propertyStartByte, source.propertyEndByte);
        is(text, source.propertyText);
        is(change.providerDiagnostics[0].nodeType, "Property");
      }
    }
  }

  const initialBytes = gunzipSync(read(primary.initialProviderExpectations.path));
  const archive = primary.initialProviderExpectations;
  bytesMatch(initialBytes, archive, "uncompressedSha256", "uncompressedBytes");
  const initial = JSON.parse(initialBytes);
  const records =
    "cases originals firstProviderReceipt firstNativeReceipt firstPrimary firstHelper";
  for (const key of records.split(" "))
    bytesMatch(Buffer.from(initial[key].completeBytes), initial[key]);
  is(initial.cases.completeBytes, read("cases.json").toString());
  is(initial.originals.completeBytes, read("originals.json").toString());
  is(initial.firstHelper.completeBytes, read("initial-observer.mjs.fixture").toString());
  for (const [receipt, key] of [
    [native, "firstNativeReceipt"],
    [provider, "firstProviderReceipt"],
  ]) {
    is(receipt.priorAttempts.length, 1);
    is(unpack(receipt.priorAttempts[0]).toString(), initial[key].completeBytes);
    is(receipt.priorAttempts[0].sha256, initial[key].sha256);
  }
  is(supplement.firstProviderReceiptSha256, initial.firstProviderReceipt.sha256);
  const firstPrimary = JSON.parse(initial.firstPrimary.completeBytes);
  is(firstPrimary.observer.sha256, initial.firstHelper.sha256);
  const firstNative = JSON.parse(initial.firstNativeReceipt.completeBytes),
    firstProvider = JSON.parse(initial.firstProviderReceipt.completeBytes);
  eq(idsOf(firstNative.runs), ids);
  eq(idsOf(firstProvider.runs), ids);
  const firstDirectory = path.posix.dirname(firstProvider.runs[0].expected[0].filePath);
  entries.forEach((entry, position) => {
    nativePacket(firstNative.runs[position], entry);
    const run = firstProvider.runs[position];
    inputPacket(run, entry);
    eq(run.expected, providerExpected(entry, entry.providerDiagnostics, firstDirectory), entry.id);
    const messages = corrected.get(entry.id) ?? entry.providerDiagnostics;
    const actual = providerExpected(entry, messages, firstDirectory);
    eq(run.actual, actual, entry.id);
    is(run.matched, !corrected.has(entry.id));
  });
  eq(firstNative.summary, native.summary);
  eq(firstProvider.summary, { ...provider.summary, matchedCases: 37 });
  eq(idsOf(firstProvider.runs.filter((entry) => !entry.matched)), idsOf(supplement.cases));

  for (const metadata of index.matrices) {
    const compressed = read(metadata.path);
    bytesMatch(compressed, metadata);
    const bytes = gunzipSync(compressed);
    bytesMatch(bytes, metadata, "uncompressedSha256", "uncompressedBytes");
    const matrix = JSON.parse(bytes);
    is(matrix.project, metadata.project);
    is(matrix.revision, metadata.revision);
    is(matrix.evidence.commitSha, metadata.sourceCommit);
    eq(matrix.budget, metadata.budget);
    is(matrix.divergence.falsePositives.length, metadata.falsePositiveCount);
    is(matrix.divergence.falseNegatives.length, metadata.falseNegativeCount);
    for (const source of index.sources.filter((entry) => entry.matrixPath === metadata.path)) {
      const entry = originals.cases.find((item) => item.id === source.id);
      for (const [key, value] of Object.entries(source)) eq(entry[key], value);
      bytesMatch(read(source.sourceFile), source, "sourceSha256");
      const repository = source.repository.replace("github.com", "raw.githubusercontent.com");
      is(source.url, `${repository}/${source.revision}/${source.path}`);
      const findings = matrix.divergence.falsePositives.filter(
        (finding) => finding.file === source.path && finding.ruleId === corpus.rule,
      );
      eq(source.matrixAllReservedFindings, findings);
      for (const range of source.originalRanges)
        is(slice(entry.source, range.start, range.end), range.token);
    }
  }
  eq(counts(index.sources, "matrixPrefixFindings matrixAllReservedFindings"), [37, 39]);
  const parent = json(successors.parentCorpus.path);
  bytesMatch(read(successors.parentCorpus.path), successors.parentCorpus);
  is(parent.cases.length, 16);
  eq(counts(parent.cases, "beforeDiagnostics diagnostics"), [137, 17]);
  eq([successors.cases.length, count(successors.cases, "diagnostics")], [16, 9]);
  const names = {
    "mixed-bare-and-prefix-data": ["$data", "$props", "_private"],
    "prefix-props-policy-control": ["$props"],
    "prefix-computed-methods-control": ["$attrs", "$emit"],
    "prefix-setup-inject-control": ["$slots"],
    "quoted-escaped-utf8-crlf": ["$data", "_private"],
  };
  for (const [position, entry] of successors.cases.entries()) {
    const original = parent.cases[position];
    for (const key of ["id", "filename", "sourceSha256"]) is(entry[key], original[key]);
    const expected = (names[entry.id] ?? []).map((name) =>
      original.diagnostics.find((finding) => finding.message === `Reserved key '${name}'`),
    );
    eq(entry.diagnostics, expected);
  }
  const historical = json("historical-computed-underscore.json"),
    snapshot = read(historical.snapshotFile),
    source = read(historical.sourceFile);
  bytesMatch(snapshot, historical, "snapshotSha256");
  bytesMatch(source, historical, "sourceSha256");
  const blob = createHash("sha1")
    .update(`blob ${snapshot.length}\0`)
    .update(snapshot)
    .digest("hex");
  is(blob, historical.originalGitBlob);
  is(historical.sourceCommit, primary.sourceParent.commit);
  is(slice(source, historical.sourceRange.start, historical.sourceRange.end), "_foo");
  eq(historical.diagnostics, []);
  eq(historical.providerDiagnostics, []);
});
