import assert from "node:assert/strict";
import fs from "node:fs";
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
import { passedFixtureTests } from "../differential/typechecker-shards.ts";

const root = fileURLToPath(new URL("../../", import.meta.url));
const manifest = path.join(root, "tests/_fixtures/differential/typechecker/manifest.json");
const historyLedger = path.join(root, "docs/davinci/plan/typechecker-fix-history.tsv");
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
  assert.equal(loaded.cases.length, 74);
  assert.equal(
    loaded.cases.reduce((count, fixture) => count + fixture.inputs.length, 0),
    262,
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

test("registered original-history fixtures stay admitted in the requirement ledger", () => {
  const [header, ...lines] = fs.readFileSync(historyLedger, "utf8").trimEnd().split("\n");
  assert.equal(
    header,
    "commit\tclassification\tfocused_requirement\tdiagnostic_fixture_state\tremaining_scope\tsubject",
  );
  assert.equal(lines.length, 271);
  const rows = new Map(
    lines.map((line) => {
      const fields = line.split("\t");
      assert.equal(fields.length, 6);
      return [fields[0], fields] as const;
    }),
  );
  assert.equal(rows.size, 271);

  const registered = loaded.cases.filter((fixture) => rows.has(fixture.regressionCommit));
  assert.equal(new Set(registered.map((fixture) => fixture.regressionCommit)).size, 13);
  assert.equal(registered.length, 32);
  for (const fixture of registered) {
    assert.equal(
      rows.get(fixture.regressionCommit)?.[3],
      "batch-start-actions-verified",
      `${fixture.pack} is registered but absent from the historical admission ledger`,
    );
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
  assert.equal(report.summary.legacyMatches, 74);
  const acceptance = summarizeNativeAcceptance(loaded, report, {
    sourceRevision,
    buildReceiptSha256: sha256(receipt),
    requiredStages: ["parse", "facts", "emit", "check"],
  });
  assert.deepEqual(acceptance.total, {
    planned: 74,
    nativeHandled: 0,
    nativeEquivalent: 0,
    unsupported: 74,
    legacyBacked: 0,
    unverified: 0,
  });
});
