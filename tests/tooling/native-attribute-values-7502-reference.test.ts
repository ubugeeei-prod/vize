// Pure contract laws. Actual native acceptance is mandatory in the hosted judge.
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";
import {
  rawProcess7502,
  validateProcess7502,
} from "./support/native-attribute-values-7502-build.ts";
import {
  envelopes7502,
  validateEnvelopes7502,
} from "./support/native-attribute-values-7502-envelopes.ts";
import {
  exactKeys7502,
  archivedCapture7502,
  archiveOutputSha7502,
  archiveOutputUrl7502,
  baseline7502,
  disposition7502,
  fixtureSha7502,
  fixtureUrl7502,
  hash7502,
  loadInputs7502,
  outputUrl7502,
  requireReviewed7502,
  transition7502,
  unchangedRows7502,
} from "./support/native-attribute-values-7502-inputs.ts";
import { mandatory7502Paths } from "./support/native-attribute-values-7502-judge.ts";
import { coordinate7502, maps7502 } from "./support/native-attribute-values-7502-maps.ts";
import {
  historyCommands7502,
  validateHistory7502,
} from "./support/native-attribute-values-7502-history.ts";
import { validateCapture7502 } from "./support/native-attribute-values-7502-oracle.ts";

test("verbatim original fourteen pins retain original HTML/options and precise two lower controls", () => {
  const bytes = readFileSync(fixtureUrl7502),
    pack = loadInputs7502(bytes);
  assert.equal(hash7502(bytes), fixtureSha7502);
  assert.equal(pack.fixtures.length, 14);
  assert.deepEqual(
    pack.fixtures
      .filter((row: any) => row.disposition === "lower-refusal")
      .map((row: any) => [row.id, row.source]),
    [
      ["original-regression-11", '<div class="a&amp;amp;b">x</div>'],
      ["original-regression-12", '<div title="a &amp;lt; b">&amp;lt;</div>'],
    ],
  );
  assert.equal(pack.fixtures[0].source, '<div title="a &amp;lt; b">x</div>');
  assert.equal(pack.fixtures[5].source, '<div title="a&b">x</div>');
  assert.equal(pack.fixtures[5].existingExpectedTemplateHtml, '<div title="a&b">x</div>');
  assert.equal(pack.fixtures[8].source, "<div title=a&b>x</div>");
  assert.equal(pack.fixtures[8].existingExpectedTemplateHtml, '<div title="a&amp;b">x</div>');
  for (const mutate of [
    (value: any) => (value.fixtures[0].source += "\n"),
    (value: any) => (value.fixtures[0].existingOptions = "selected custom options"),
    (value: any) => (value.fixtures[11].disposition = "positive"),
  ]) {
    const changed = structuredClone(pack);
    mutate(changed);
    assert.throws(
      () => loadInputs7502(Buffer.from(JSON.stringify(changed))),
      /original input pack drift/,
    );
  }
});

test("unfrozen/null, missing fields and incomplete rows cannot grant whole native output acceptance", () => {
  const stored = JSON.parse(readFileSync(outputUrl7502, "utf8"));
  exactKeys7502(stored, ["schema", "version", "state", "capture"]);
  assert.equal(stored.schema, "vize.native-attribute-values-7502.reviewed-output");
  assert.equal(stored.version, 2);
  assert(["unfrozen", "reviewed"].includes(stored.state));
  if (stored.state === "unfrozen") assert.equal(stored.capture, null);
  else validateCapture7502(stored.capture); // Schema only; never hosted runtime credit.
  const unfrozen = {
    schema: "vize.native-attribute-values-7502.reviewed-output",
    version: 2,
    state: "unfrozen",
    capture: null,
  };
  const synthetic = { syntheticContractOnly: true };
  assert.throws(() => requireReviewed7502(synthetic, unfrozen), /remain unfrozen/);
  assert.throws(
    () => requireReviewed7502(synthetic, { ...unfrozen, state: "reviewed" }),
    /complete reviewed capture/,
  );
  assert.throws(() =>
    requireReviewed7502(synthetic, {
      ...unfrozen,
      state: "reviewed",
      capture: {},
      unexpected: true,
    }),
  );
  assert.throws(() =>
    requireReviewed7502(synthetic, { ...unfrozen, state: "reviewed", capture: { other: true } }),
  );
  assert.throws(() => validateCapture7502({ rows: [] }));
  assert.throws(() => validateEnvelopes7502({ rows: [] }));
  assert.throws(() => exactKeys7502({ present: null }, ["present", "missing"]));
  assert.throws(() => exactKeys7502({ present: null, extra: null }, ["present"]));
});

test("mandatory hosted paths reject absent capture instead of using fixture native modules", () => {
  assert.throws(() => mandatory7502Paths({}), /actual hosted evidence path required/);
  assert.throws(() =>
    mandatory7502Paths({ VIZE_NATIVE_ATTRIBUTE_VALUES_7502_CAPTURE: "fixture-code.json" }),
  );
  assert.deepEqual(
    mandatory7502Paths({
      VIZE_NATIVE_ATTRIBUTE_VALUES_7502_EVIDENCE_DIR: "/evidence",
      VIZE_NATIVE_ATTRIBUTE_VALUES_7502_CAPTURE: "/evidence/first.capture.json",
      VIZE_NATIVE_ATTRIBUTE_VALUES_7502_BUILD_RECEIPT: "/evidence/build-receipt.json",
      VIZE_NATIVE_ATTRIBUTE_VALUES_7502_HISTORY_RECEIPT: "/evidence/history-receipt.json",
    }),
    {
      directory: "/evidence",
      capture: "/evidence/first.capture.json",
      build: "/evidence/build-receipt.json",
      history: "/evidence/history-receipt.json",
    },
  );
});

test("the named successor retains every original byte and every unaffected whole row", () => {
  assert.equal(hash7502(readFileSync(archiveOutputUrl7502)), archiveOutputSha7502);
  const archived = archivedCapture7502();
  const packet = { rows: structuredClone(archived.rows) };
  unchangedRows7502(packet);
  const classRows = packet.rows.filter((row: any) => row.id === transition7502.input);
  assert.deepEqual(classRows.map(disposition7502), [
    "positive",
    "positive",
    "positive",
    "positive",
    "target-refusal",
    "target-refusal",
  ]);
  assert(classRows.every((row: any) => row.disposition === "lower-refusal"));
  packet.rows[0].observation.file.nodeCount++;
  assert.throws(() => unchangedRows7502(packet), /unaffected original whole row/);
  assert.throws(() => validateCapture7502(archived));
  assert.throws(() => validateHistory7502({}, {}, "/missing-history"));
  const relabelled = structuredClone(archived);
  relabelled.version = 2;
  relabelled.transition = transition7502;
  relabelled.summary = {
    fixtures: 14,
    outcomes: 84,
    positive: 76,
    lowerRefusals: 6,
    targetRefusals: 2,
  };
  assert.throws(() => validateCapture7502(relabelled));
  const commands = historyCommands7502("/baseline", "/history");
  assert.deepEqual(commands[0], [
    "git",
    ["fetch", "--no-tags", "origin", "815d9342ed252cad5802e58931b15166f25bf746"],
  ]);
  assert.deepEqual(commands.slice(-2), [
    ["vp", ["node", "tests/tooling/support/native-attribute-values-7502-build.ts", "/history"]],
    ["vp", ["node", "tests/tooling/support/native-attribute-values-7502-judge.ts"]],
  ]);
});

test("removing a genuine class segment anchor is rejected by the map judge", () => {
  // A pure map counterfactual supplies no native code or runtime acceptance.
  const source = "class",
    mappings = JSON.stringify([[[0, 0, 0, 0]]]);
  const row = {
    id: transition7502.input,
    target: "dom",
    disposition: "lower-refusal",
    sourceMap: true,
    filename: "Class.vue",
    nativeSource: source,
    result: {
      code: source,
      map: {
        file: "Class.vue",
        sources: ["Class.vue"],
        sourcesContent: [source],
        names: [],
        mappings,
      },
      links: [
        {
          authored: { start: 0, end: 5 },
          generated: { start: 0, end: 5 },
          name: null,
          segment: true,
        },
      ],
    },
  };
  const codec = { decode: JSON.parse, encode: JSON.stringify };
  maps7502({ rows: [row] }, codec);
  row.result.links.pop();
  assert.throws(() => maps7502({ rows: [row] }, codec), /complete exact start anchor/);
});

test("historical source and command substitutions fail before evidence can grant credit", () => {
  const receipt = {
    schema: "vize.native-attribute-values-7502.history",
    version: 1,
    state: "qualified",
    currentSource: { current: true },
    baseline: baseline7502,
    baselineSource: {
      sourceRevision: baseline7502.revision,
      sourceTree: baseline7502.tree,
      fixtureSourceTree: baseline7502.fixtureTree,
      inputPackSha256: fixtureSha7502,
    },
    originalInputSha256: fixtureSha7502,
    originalReviewedOutputSha256: archiveOutputSha7502,
    worktree: "/native-attribute-values-7502-baseline",
    attempts: [] as any[],
    buildReceiptSha256: null,
    qualificationSha256: null,
    captureSha256: null,
    envelopesSha256: null,
    failure: null,
  };
  for (const mutate of [
    (row: any) => (row.state = "unqualified"),
    (row: any) => (row.currentSource = { stale: true }),
    (row: any) => (row.baselineSource.sourceRevision = "wrong"),
    (row: any) => (row.baselineSource.fixtureSourceTree = "wrong"),
    (row: any) => (row.originalReviewedOutputSha256 = "wrong"),
  ]) {
    const changed = structuredClone(receipt);
    mutate(changed);
    assert.throws(() => validateHistory7502(changed, receipt.currentSource, "/evidence"));
  }
  const commands = historyCommands7502(receipt.worktree, "/evidence/history");
  receipt.attempts = ["fetch", "checkout", "install", "runtime", "build", "judge"].map(
    (step, index) => ({
      step,
      executable: commands[index][0],
      argv: commands[index][1],
      cwd: "/wrong",
      ...rawProcess7502({
        status: 0,
        signal: null,
        stdout: Buffer.alloc(0),
        stderr: Buffer.alloc(0),
      }),
    }),
  );
  receipt.attempts[0].argv = ["fetch", "origin", "main"];
  assert.throws(() => validateHistory7502(receipt, receipt.currentSource, "/evidence"));
});

test("raw failure streams retain exact bytes even on throw and reject tampering", () => {
  // Synthetic schema controls confer no source-built or native runtime credit.
  const raw = rawProcess7502({
    status: null,
    signal: "SIGTERM",
    error: new Error("synthetic failure"),
    stdout: Buffer.from([0, 255, 10]),
    stderr: Buffer.from("synthetic error\n"),
  });
  validateProcess7502(raw);
  assert.equal(raw.exitStatus, null);
  assert.equal(raw.signal, "SIGTERM");
  assert.equal(raw.processError, "synthetic failure");
  assert(Buffer.from(raw.stdoutBase64, "base64").equals(Buffer.from([0, 255, 10])));
  for (const changed of [
    { ...raw, stdoutSha256: "0".repeat(64) },
    { ...raw, stderrBase64: "!" },
    { ...raw, hidden: true },
    { ...raw, processError: 3 },
  ])
    assert.throws(() => validateProcess7502(changed));
});

test("byte-map endpoints preserve UTF-8 and independent envelope coverage remains complete", () => {
  assert.deepEqual(coordinate7502("雪\n🌸", 3), [0, 1]);
  assert.deepEqual(coordinate7502("雪\n🌸", 4), [1, 0]);
  assert.deepEqual(coordinate7502("雪\n🌸", 8), [1, 2]);
  for (const offset of [-1, 1, 2, 5, 9]) assert.throws(() => coordinate7502("雪\n🌸", offset));
  assert.deepEqual(
    envelopes7502.map((entry) => entry.issue),
    ["Script(Ordinary)", "Script(Setup)", "Style", "Style"],
  );
  assert.equal(loadInputs7502().fixtures.length * envelopes7502.length * 3 * 2, 336);
});
