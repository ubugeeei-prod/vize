import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { gunzipSync } from "node:zlib";
import { sha256, wholeJson } from "./lint-default-correctness-reference.mjs";
import {
  validateProviderEnvelope,
  validatePublicExportTable,
} from "./support/public-vue-imports-reference-contract.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const fixture = path.join(root, "tests/_fixtures/differential/lint/public-vue-imports-8090");
const read = (name) => fs.readFileSync(path.join(fixture, name));
const json = (name) => JSON.parse(read(name));
const primary = json("primary-sources.json");
const corpus = json("cases.json");
const supplement = json("provider-envelope-supplement.json");
const authenticate = (entry, directory = fixture) => {
  const bytes = fs.readFileSync(path.join(directory, entry.path));
  assert.equal(sha256(bytes), entry.sha256, entry.path);
  if (entry.bytes !== undefined) assert.equal(bytes.length, entry.bytes, entry.path);
  return bytes;
};

await test("pinned whole carriers, public exports and legacy producers retain source custody", () => {
  assert.equal(corpus.cases.length, 29);
  assert.equal(
    sha256(read("cases.json")),
    "d52480f474c2a6dba9681aa620a0c5e6dd2597accd91ad286000230a1b9d2389",
  );
  assert.equal(
    sha256(read("default-route-controls.json")),
    "412dc8a553f58e3feda1d9ffc572cc89e5f08929d134c8943ee11a8689db888f",
  );
  for (const entry of [
    primary.corpus,
    primary.routes,
    primary.original,
    primary.providerEnvelope,
    primary.initialReference,
    primary.releasedFixContract,
    ...primary.provider.primarySources,
    ...primary.references,
    ...primary.referenceExecution,
  ])
    authenticate(entry);
  authenticate(primary.observer);
  // Live observers and consumers are edited by other work; require them, never pin their bytes.
  for (const entry of [
    primary.observer.wholeJson,
    primary.observer.envelopeContract,
    ...primary.consumers,
  ])
    assert(fs.existsSync(path.join(root, entry.path)), entry.path);
  for (const entry of corpus.cases) {
    const bytes = read(entry.sourceFile);
    assert.equal(bytes.toString("utf8"), entry.source);
    assert.equal(sha256(bytes), entry.sourceSha256);
    assert.equal(sha256(entry.fixedSource), entry.fixedSourceSha256);
    let fixed = Buffer.from(entry.source);
    const edits = entry.diagnostics
      .flatMap((d) => d.fix?.edits ?? [])
      .sort((a, b) => b.start - a.start);
    for (const edit of edits)
      fixed = Buffer.concat([
        fixed.subarray(0, edit.start),
        Buffer.from(edit.new_text),
        fixed.subarray(edit.end),
      ]);
    assert.equal(fixed.toString("utf8"), entry.fixedSource);
    for (const findings of [
      entry.currentParentDiagnostics,
      entry.diagnostics,
      entry.fixedDiagnostics,
    ]) {
      for (const d of findings) {
        assert.deepEqual(
          Object.keys(d).sort(),
          ["rule_name", "severity", "message", "start", "end", "help", "labels", "fix"].sort(),
        );
        assert.ok(d.start < d.end && d.end <= bytes.length);
        assert.equal(d.rule_name, corpus.rule);
        assert.equal(d.severity, "warning");
        assert.equal(d.help, "Import from 'vue' directly for better compatibility");
        assert.deepEqual(d.labels, []);
      }
    }
  }
  assert.equal(primary.original.bytes, 452);
  const packetBytes = gunzipSync(authenticate(primary.matrix));
  assert.equal(packetBytes.length, 9123302);
  assert.equal(sha256(packetBytes), primary.matrix.rawSha256);
  const packet = JSON.parse(packetBytes);
  assert.equal(packet.project, "element-plus");
  assert.equal(packet.revision, primary.matrix.revision);
  assert.deepEqual(packet.budget, {
    falseNegativePassed: false,
    falsePositivePassed: false,
    maxFalseNegativeCount: 0,
    maxFalsePositiveCount: 0,
    passed: false,
    unusableReason: null,
    verdict: "breached",
  });
  assert.equal(primary.matrix.source, "fd4c30a1c6195c4a9e569c17e99d9942b84b7833");
  assert.equal(primary.matrix.artifactId, 11709971103);
  assert.equal(
    primary.matrix.archiveSha256,
    "7bb9ae76824189884d7a9c59aa66672c9e3b8cdf9dfcd33bf95b41697da766e7",
  );
  const lock = gunzipSync(authenticate(primary.provider.lock));
  assert.equal(sha256(lock), primary.provider.lock.rawSha256);
  assert.equal(lock.length, primary.provider.lock.rawBytes);
  assert.deepEqual(
    primary.provider.packages.map((p) => [p.name, p.version, p.fileCount]),
    [
      ["eslint", "10.4.1", 419],
      ["eslint-plugin-vue", "10.9.2", 340],
      ["vue-eslint-parser", "10.4.1", 6],
      ["@typescript-eslint/parser", "8.65.0", 7],
      ["typescript", "6.0.3", 140],
    ],
  );
  for (const p of primary.provider.packages) {
    assert.equal(p.files.length, p.fileCount);
    assert.equal(sha256(p.files.map((f) => `${f.sha256}  ${f.path}\n`).join("")), p.fileTreeSha256);
    assert.equal(p.files.find((f) => f.path === "package.json").sha256, p.packageJsonSha256);
  }
  const before = authenticate(primary.sourceProducer.beforeFixture);
  assert.deepEqual(authenticate(primary.sourceProducer.releasedFixture), before);
  // The live producer and export table are shipped code: the archived producer
  // stays pinned, and the live table is checked by its exported names below.
  assert(fs.existsSync(path.join(root, primary.sourceProducer.path)), primary.sourceProducer.path);
  const table = fs.readFileSync(path.join(root, primary.compileTimeTable.path), "utf8");
  const names = [
    ...table.slice(table.indexOf("= &["), table.indexOf("];")).matchAll(/"([^"]+)"/g),
  ].map((m) => m[1]);
  const pinned = json("vue3-export-names.json");
  validatePublicExportTable(names, pinned);
  assert.ok(names.includes("ref") && names.includes("camelize") && !names.includes("isString"));
  for (const mutated of [
    names.slice(1),
    [...names, "newUnpinnedExport"],
    [names[0], ...names.slice(0, -1)],
    [...names].reverse(),
  ]) {
    assert.throws(() => validatePublicExportTable(mutated, pinned));
  }
});

await test("ESLint10 supplement rejects every semantic or unrelated envelope change", () => {
  validateProviderEnvelope(corpus, supplement);
  assert.equal(supplement.initialCorpusSha256, sha256(read("cases.json")));
  assert.equal(supplement.initialRoutesSha256, sha256(read("default-route-controls.json")));
  const serializer = read(supplement.serializer.fixture).toString("utf8");
  assert.equal(sha256(serializer), supplement.serializer.sha256);
  const body = serializer.slice(
    serializer.indexOf("function createProblem(options) {"),
    serializer.indexOf("\n}\n", serializer.indexOf("function createProblem(options) {")) + 3,
  );
  assert.equal(body, supplement.serializer.createProblem.source);
  assert.equal(sha256(body), supplement.serializer.createProblem.sha256);
  assert.ok(!body.includes("nodeType"));
  assert.ok(serializer.includes("loc: descriptor.loc"));
  assert.ok(serializer.includes("this.#sourceCode.getLoc(descriptor.node)"));
  const p = primary.provider.packages.find((p) => p.name === "eslint");
  assert.equal(p.fileTreeSha256, supplement.serializer.packageTreeSha256);
  assert.equal(p.packageJsonSha256, supplement.serializer.packageJsonSha256);
  assert.equal(
    p.files.find((f) => f.path === "lib/linter/file-report.js").sha256,
    supplement.serializer.sha256,
  );
  const mutations = [
    (s) => s.cases.pop(),
    (s) => s.cases.reverse(),
    (s) => (s.cases[0].sourceSha256 = "changed"),
    (s) => (s.serializer.version = "9.39.1"),
    ...[
      "ruleId",
      "severity",
      "message",
      "line",
      "column",
      "endLine",
      "endColumn",
      "messageId",
    ].flatMap((key) => [
      (s) => (s.cases.find((c) => c.diagnostics.length).diagnostics[0][key] = "changed"),
      (s) => delete s.cases.find((c) => c.diagnostics.length).diagnostics[0][key],
    ]),
    (s) => (s.cases.find((c) => c.diagnostics.length).diagnostics[0].extra = true),
    (s) => (s.cases.find((c) => c.diagnostics.length).diagnostics[0].nodeType = "Literal"),
    (s) =>
      (s.cases
        .find((c) => c.diagnostics.some((d) => d.fix))
        .diagnostics.find((d) => d.fix).fix.text = "changed"),
    (s) =>
      s.cases.find((c) => c.diagnostics.some((d) => d.fix)).diagnostics.find((d) => d.fix).fix
        .range[0]++,
    (s) =>
      delete s.cases.find((c) => c.diagnostics.some((d) => d.fix)).diagnostics.find((d) => d.fix)
        .fix,
  ];
  for (const mutate of mutations) {
    const changed = structuredClone(supplement);
    mutate(changed);
    assert.throws(() => validateProviderEnvelope(corpus, changed));
  }
});

await test("complete initial failure and corrected reference packets remain distinct", () => {
  const initialRaw = gunzipSync(authenticate(supplement.initialReference));
  assert.equal(sha256(initialRaw), supplement.initialReference.rawSha256);
  assert.equal(initialRaw.length, supplement.initialReference.rawBytes);
  const initial = JSON.parse(initialRaw);
  assert.equal(initial.files.length, 35);
  const initialJson = (name) => {
    const entry = initial.files.find((f) => f.path === name);
    const bytes = Buffer.from(entry.content, "base64");
    assert.equal(sha256(bytes), entry.sha256);
    assert.equal(bytes.length, entry.bytes);
    return JSON.parse(bytes);
  };
  for (const entry of initial.files) {
    const bytes = Buffer.from(entry.content, "base64");
    assert.equal(sha256(bytes), entry.sha256);
    assert.equal(bytes.length, entry.bytes);
    if (
      [
        "cases.json",
        "default-route-controls.json",
        ...corpus.cases.map((c) => c.sourceFile),
      ].includes(entry.path)
    )
      assert.deepEqual(read(entry.path), bytes);
  }
  const oldProvider = initialJson("official-provider.json");
  const oldReleased = initialJson("released-before.json");
  assert.deepEqual(oldProvider.summary, { cases: 29, matchedCases: 10, diagnostics: 19 });
  assert.deepEqual(oldReleased.summary, { cases: 29, matchedCases: 29, diagnostics: 24 });
  for (const row of oldProvider.runs) {
    const messages = row.expected[0].messages.map(({ nodeType: _nodeType, ...rest }) => rest);
    assert.deepEqual(row.actual, [{ ...row.expected[0], messages }]);
    assert.equal(row.matched, row.expected[0].messages.length === 0);
  }
  const released = json("released-before.json");
  const official = json("official-provider.json");
  const fixed = json("released-fix-before.json");
  assert.deepEqual(released.summary, { cases: 29, matchedCases: 29, diagnostics: 24 });
  assert.deepEqual(official.summary, { cases: 29, matchedCases: 29, diagnostics: 19 });
  assert.deepEqual(fixed.summary, { cases: 6, matchedCases: 6, diagnostics: 0 });
  assert.equal(released.baseline.verifiedBinaryBeforeSha256, primary.publicBefore.binarySha256);
  assert.equal(released.baseline.verifiedBinaryAfterSha256, primary.publicBefore.binarySha256);
  assert.deepEqual(official.verifiedPackagesBefore, official.verifiedPackagesAfter);
  const execution = json("reference-execution-primary.json.fixture");
  assert.equal(execution.observer.sha256, sha256(read("reference-execution-observer.mjs.fixture")));
  for (const receipt of [released, official, fixed]) {
    assert.equal(
      receipt.custody.primarySourcesSha256,
      sha256(read("reference-execution-primary.json.fixture")),
    );
    assert.equal(receipt.custody.helperSha256, execution.observer.sha256);
    assert.equal(receipt.custody.corpusSha256, sha256(read("cases.json")));
  }
  assert.equal(
    execution.observer.envelopeContract.sha256,
    sha256(read("reference-execution-envelope-contract.mjs.fixture")),
  );
  assert.equal(
    execution.providerEnvelope.sha256,
    sha256(read("provider-envelope-supplement.json")),
  );
  assert.deepEqual(
    official.verifiedPackagesBefore,
    execution.provider.packages.map((p) => ({
      name: p.name,
      version: p.version,
      treeSha256: p.fileTreeSha256,
    })),
  );

  for (const [receipt, name] of [
    [released, "released-before.json"],
    [official, "official-provider.json"],
  ]) {
    assert.equal(receipt.priorAttempts.length, 1);
    const prior = gunzipSync(Buffer.from(receipt.priorAttempts[0].completePacket, "base64"));
    assert.equal(sha256(prior), initial.files.find((f) => f.path === name).sha256);
  }
  for (const receipt of [released, official, fixed])
    for (const row of receipt.runs) {
      assert.equal(row.matched, true);
      assert.deepEqual(row.actual, row.expected);
      const entry = corpus.cases.find((c) => c.id === row.id);
      assert.equal(row.input.source, entry.source);
      assert.equal(row.input.sha256, entry.sourceSha256);
      if (receipt === official) {
        assert.equal(path.basename(row.actual[0].filePath), entry.filename);
        assert.deepEqual(
          row.actual[0].messages,
          supplement.cases.find((c) => c.id === row.id).diagnostics,
        );
        assert.deepEqual(row.actual[0].suppressedMessages, []);
        assert.deepEqual(row.actual[0].usedDeprecatedRules, []);
        assert.equal(row.inputAfterSha256, entry.sourceSha256);
      } else {
        assert.equal(row.status, 0);
        assert.equal(row.signal, null);
        assert.equal(row.error, null);
        assert.equal(row.stderrBase64, "");
        assert.equal(row.stderr, "");
        assert.equal(Buffer.from(row.stdoutBase64, "base64").toString("utf8"), row.stdout);
        assert.deepEqual(JSON.parse(row.stdout), row.expected);
        if (receipt === released)
          assert.deepEqual(row.expected, wholeJson(entry, entry.currentParentDiagnostics));
        else {
          const contract = json("released-fix-contract.json").cases.find((c) => c.id === row.id);
          assert.equal(row.finalSource, contract.fixedSource);
          assert.equal(row.finalSourceSha256, contract.fixedSourceSha256);
          assert.deepEqual(
            row.expected,
            wholeJson({ ...entry, source: contract.fixedSource }, contract.diagnostics),
          );
        }
      }
    }
  const controls = json("default-route-controls.json");
  assert.equal(controls.corpusSha256, sha256(read("cases.json")));
  const successor = json("default-route-controls-successor.json");
  assert.deepEqual(successor.predecessor, {
    path: "default-route-controls.json",
    sha256: sha256(read("default-route-controls.json")),
  });
  for (const [route, authored] of Object.entries(successor.routes)) {
    for (const c of controls.cases) {
      assert.ok(c.routes.some((r) => r.id === route) && c.routes.some((r) => r.id === authored));
    }
  }
  assert.equal(controls.cases.length, 4);
  for (const c of controls.cases) assert.equal(c.routes.length, 7);
  assert.equal(
    corpus.cases.reduce((n, c) => n + c.diagnostics.length, 0),
    17,
  );
  assert.equal(
    corpus.cases.reduce((n, c) => n + c.currentParentDiagnostics.length, 0),
    24,
  );
});
