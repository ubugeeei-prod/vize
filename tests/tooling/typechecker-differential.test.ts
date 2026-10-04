import assert from "node:assert/strict";
import fs from "node:fs";
import { execFileSync } from "node:child_process";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import { sha256 } from "../differential/harness.mjs";
import { summarizeNativeAcceptance } from "../differential/acceptance-rates.mjs";
import {
  TYPECHECKER_TESTS,
  loadTypecheckerManifest,
  typecheckerReport,
  validateTypecheckerCapture,
  type Capture,
} from "../differential/typechecker.ts";
import {
  aggregateTypecheckerWorkers,
  passedFixtureTests,
} from "../differential/typechecker-shards.ts";

const root = fileURLToPath(new URL("../../", import.meta.url));
const manifest = path.join(root, "tests/_fixtures/differential/typechecker/manifest.json");
const loaded = loadTypecheckerManifest(manifest);
const receipt = Buffer.from("synthetic unit-test receipt, not runtime evidence");
const binarySha256 = sha256(Buffer.from("synthetic unit-test executable identity"));
const sourceRevision = "1".repeat(40);

// These synthetic transport objects test rejection/accounting. Actual execution
// is required separately through all registered full-worker JUnit cases and receipts.
function capture(pack: keyof typeof TYPECHECKER_TESTS): Capture {
  const fixtures = loaded.cases.filter((fixture) => fixture.pack === pack);
  return {
    schema: "vize.typechecker-fixture-observation",
    version: 1,
    pack,
    test: TYPECHECKER_TESTS[pack],
    binaryPath: "/synthetic/unit-test-binary",
    fixturePackSha256: fixtures[0].packReference.sha256,
    archiveReceiptSha256: sha256(receipt),
    binarySha256,
    missingFields: ["end", "relatedInformation", "raw backend diagnostics"],
    matchedContract: "batch-start-diagnostics-v1",
    unbaselinedFields: ["diagnostics[].blockType", "exitCode", "success"],
    cases: fixtures.map((fixture) => ({
      id: fixture.case,
      projectRoot: "/synthetic/project",
      inputs: fixture.inputs.map((input) => ({ file: input.file, sha256: input.sha256 })),
      diagnostics: structuredClone(fixture.diagnostics),
      publicResult: {
        exitCode: 0,
        success: true,
        diagnostics: fixture.diagnostics.map((diagnostic) => ({
          ...diagnostic,
          file: `/synthetic/project/${diagnostic.file}`,
          line: diagnostic.line - 1,
          column: diagnostic.column - 1,
          blockType: null,
        })),
      },
    })),
  };
}

test("the shared registry retains all existing production projects and exact input bytes", () => {
  assert.equal(loaded.cases.length, 28);
  assert.equal(
    loaded.cases.reduce((count, fixture) => count + fixture.inputs.length, 0),
    100,
  );
  assert.deepEqual(
    [...new Set(loaded.cases.map((fixture) => fixture.pack))],
    Object.keys(TYPECHECKER_TESTS),
  );
  assert.deepEqual(
    [...new Set(loaded.cases.map((fixture) => fixture.adapters.legacy))],
    ["batch-typechecker-fixture-v1"],
  );
  for (const pack of Object.keys(TYPECHECKER_TESTS) as Array<keyof typeof TYPECHECKER_TESTS>) {
    validateTypecheckerCapture(loaded, capture(pack), { receipt, binarySha256 });
  }
});

test("missing, duplicated and reordered projects cannot confer legacy coverage", () => {
  const original = capture("component-event-tuples");
  for (const change of [
    (value: Capture) => {
      value.cases.pop();
    },
    (value: Capture) => {
      value.cases.push(value.cases[0]);
    },
    (value: Capture) => {
      value.cases.reverse();
    },
  ]) {
    const value = structuredClone(original);
    change(value);
    assert.throws(() => validateTypecheckerCapture(loaded, value, { receipt, binarySha256 }));
  }
  const captures = (Object.keys(TYPECHECKER_TESTS) as Array<keyof typeof TYPECHECKER_TESTS>).map(
    capture,
  );
  assert.throws(() => typecheckerReport(loaded, captures.slice(1), sourceRevision));
  assert.throws(() => typecheckerReport(loaded, [...captures, captures[0]], sourceRevision));
});

test("every exact diagnostic field and unexpected child row remains part of the contract", () => {
  const original = capture("event-handler-narrowing");
  const mutations = [
    ["file", "src/child.vue"],
    ["line", 8],
    ["column", 34],
    ["severity", 2],
    ["code", 9999],
    ["message", "changed diagnostic"],
  ] as const;
  for (const [key, replacement] of mutations) {
    const value = structuredClone(original);
    Object.assign(value.cases[2].diagnostics[0], { [key]: replacement });
    assert.throws(() => validateTypecheckerCapture(loaded, value, { receipt, binarySha256 }), key);
  }
  const value = structuredClone(original);
  value.cases[0].diagnostics.push({ ...value.cases[2].diagnostics[0], file: "src/child.vue" });
  assert.throws(() => validateTypecheckerCapture(loaded, value, { receipt, binarySha256 }));
});

test("input, fixture, executable and transferred build identity changes all fail", () => {
  const original = capture("event-handler-narrowing");
  for (const key of ["fixturePackSha256", "archiveReceiptSha256", "binarySha256"] as const) {
    const value = structuredClone(original);
    value[key] = "0".repeat(64);
    assert.throws(() => validateTypecheckerCapture(loaded, value, { receipt, binarySha256 }), key);
  }
  const changed = structuredClone(original);
  changed.cases[0].inputs[0].sha256 = "0".repeat(64);
  assert.throws(() => validateTypecheckerCapture(loaded, changed, { receipt, binarySha256 }));
});

test("all available public result fields are retained without inventing their baselines", () => {
  const original = capture("event-handler-narrowing");
  for (const key of ["exitCode", "success", "diagnostics"] as const) {
    const value = structuredClone(original);
    Reflect.deleteProperty(value.cases[2].publicResult, key);
    assert.throws(() => validateTypecheckerCapture(loaded, value, { receipt, binarySha256 }), key);
  }
  const missingBlock = structuredClone(original);
  Reflect.deleteProperty(missingBlock.cases[2].publicResult.diagnostics[0], "blockType");
  assert.throws(() => validateTypecheckerCapture(loaded, missingBlock, { receipt, binarySha256 }));
  const missingCaptures = (
    Object.keys(TYPECHECKER_TESTS) as Array<keyof typeof TYPECHECKER_TESTS>
  ).map(capture);
  missingCaptures[0] = missingBlock;
  assert.throws(() => typecheckerReport(loaded, missingCaptures, sourceRevision));
  const droppedRow = structuredClone(original);
  droppedRow.cases[2].publicResult.diagnostics.pop();
  assert.throws(() => validateTypecheckerCapture(loaded, droppedRow, { receipt, binarySha256 }));
  const unbaselined = structuredClone(original);
  unbaselined.cases[2].publicResult.exitCode = 17;
  unbaselined.cases[2].publicResult.success = false;
  unbaselined.cases[2].publicResult.diagnostics[0].blockType = "template";
  validateTypecheckerCapture(loaded, unbaselined, { receipt, binarySha256 });
  const captures = (Object.keys(TYPECHECKER_TESTS) as Array<keyof typeof TYPECHECKER_TESTS>).map(
    capture,
  );
  captures[0] = unbaselined;
  const row = typecheckerReport(loaded, captures, sourceRevision).rows[2];
  assert.equal(row.legacy.matchedContract, "batch-start-diagnostics-v1");
  assert.deepEqual(row.legacy.unbaselinedFields, [
    "diagnostics[].blockType",
    "exitCode",
    "success",
  ]);
  assert.deepEqual(row.legacy.observation.publicResult, unbaselined.cases[2].publicResult);
});

const junit = (name: string, body = "") =>
  `<testcase name="${name}" classname="vize_canon::fix_history_diagnostics" time="0.001">${body}</testcase>`;

test("successful JUnit identities are required; skips, failures, duplicates and unknown bodies fail", () => {
  const name = TYPECHECKER_TESTS["event-handler-narrowing"];
  assert.deepEqual(passedFixtureTests(junit(name)), [name]);
  assert.deepEqual(passedFixtureTests(`<testcase classname="foreign" name="${name}"/>`), []);
  for (const body of ["<skipped/>", "<failure/>", "<error/>"]) {
    assert.throws(() => passedFixtureTests(junit(name, body)));
  }
  assert.throws(() => passedFixtureTests(junit(name) + junit(name)));
  assert.throws(() => passedFixtureTests(junit("unregistered_test")));
  assert.throws(() => passedFixtureTests("<!DOCTYPE external>" + junit(name)));
});

test("complete legacy-only observations stay in the denominator with zero native acceptance", () => {
  const captures = (Object.keys(TYPECHECKER_TESTS) as Array<keyof typeof TYPECHECKER_TESTS>).map(
    capture,
  );
  const report = typecheckerReport(loaded, captures, sourceRevision);
  assert.equal(report.summary.legacyMatches, 28);
  const acceptance = summarizeNativeAcceptance(loaded, report, {
    sourceRevision,
    buildReceiptSha256: sha256(receipt),
    requiredStages: ["parse", "facts", "emit", "check"],
  });
  assert.deepEqual(acceptance.total, {
    planned: 28,
    nativeHandled: 0,
    nativeEquivalent: 0,
    unsupported: 28,
    legacyBacked: 0,
    unverified: 0,
  });
});

test("four-worker reconciliation rejects missing workers, stale sources and duplicate execution", (t) => {
  const directory = fs.mkdtempSync(path.join(tmpdir(), "vize-typechecker-accounting-"));
  t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
  const artifacts = path.join(directory, "artifacts");
  const outputDir = path.join(directory, "output");
  const git = (...args: string[]) =>
    execFileSync("git", args, { cwd: root, encoding: "utf8" }).trim();
  const sha = git("rev-parse", "HEAD");
  const tree = git("rev-parse", "HEAD^{tree}");
  const actualReceipt = Buffer.from(
    JSON.stringify({
      schemaVersion: 3,
      sha,
      tree,
      cargoProfile: "ci",
      nextestVersion: "0.9.146",
      rustcVersion: "rustc 1.98.0 synthetic-unit-test",
      requireTsgo: "1",
      disableTsgo: null,
      nuxtIterations: "100",
      archiveSha256: "0".repeat(64),
    }),
  );
  const assignment = (Object.keys(TYPECHECKER_TESTS) as Array<keyof typeof TYPECHECKER_TESTS>).map(
    capture,
  );
  for (const item of assignment) item.archiveReceiptSha256 = sha256(actualReceipt);
  const write = (shard: number, captures: Capture[]) => {
    const destination = path.join(artifacts, `shard-${shard}`);
    fs.mkdirSync(path.join(destination, "typechecker-fixtures"), { recursive: true });
    const tests = captures.map((item) => item.test);
    const xml = Buffer.from(tests.map((name) => junit(name)).join("\n"));
    fs.writeFileSync(path.join(destination, "junit.xml"), xml);
    fs.writeFileSync(
      path.join(destination, "typechecker-fixtures/worker.json"),
      JSON.stringify({
        schema: "vize.typechecker-worker-observations",
        version: 1,
        shard,
        sourceRevision: sha,
        sourceTree: tree,
        manifestSha256: loaded.manifestSha256,
        receiptBase64: actualReceipt.toString("base64"),
        junitSha256: sha256(xml),
        tests,
        captures,
      }),
    );
  };
  const run = () =>
    aggregateTypecheckerWorkers({ repoRoot: root, artifactRoot: artifacts, outputDir });
  write(1, assignment);
  write(2, []);
  write(3, []);
  assert.throws(run, /four complete worker artifacts/);
  write(4, []);
  assert.equal(run().report.summary.legacyMatches, 28);
  write(4, [assignment[0]]);
  assert.throws(run, /duplicate captured test body/);
  write(4, []);
  const fourth = path.join(artifacts, "shard-4/typechecker-fixtures/worker.json");
  const stale = JSON.parse(fs.readFileSync(fourth, "utf8"));
  stale.sourceRevision = "0".repeat(40);
  fs.writeFileSync(fourth, JSON.stringify(stale));
  assert.throws(run);
});
