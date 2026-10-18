import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import {
  assertOfficialDiagnostics,
  loadOfficialFixture,
  officialCases,
  type Case,
  type Original,
} from "../../tools/support/compat/content-mapper/official-fixtures.ts";

const stdout = "App.vue(1,7): error TS2322: Type 'string' is not assignable to type 'number'.\n";
const files = { "tsconfig.json": "{}", "App.vue": "const count: number = 'bad'" };
const original: Original = {
  issue: 9433,
  files,
  baselineCodes: [2367],
  expectedCodes: [2322],
  baselineTerms: [],
  expectedTerms: ["not assignable"],
  expectedStdout: stdout,
  expectedStderr: "",
  expectedStatus: 2,
  baselineStdout: "App.vue(1,7): error TS2367: Original public false positive.\n",
  baselineStderr: "",
  baselineStatus: 2,
};
const row: Case = {
  name: "strict-number",
  original: 9433,
  files,
  codes: [2322],
  stdout,
  stderr: "",
  status: 2,
};
const result = (out = stdout, err = "", status = 2) => ({
  status,
  signal: null,
  error: null,
  stdout: { bytes: Buffer.byteLength(out) },
  stderr: { bytes: Buffer.byteLength(err) },
  stdoutText: out,
  stderrText: err,
});

test("official fixture carries whole original and strict negative streams into candidate cases", () => {
  const fixture = {
    typescript: "7.1.0-dev.20261017.1",
    vue: "3.5.20",
    baselineVize: "0.463.0",
    originals: [original],
    controls: [row],
  };
  const cases = officialCases(fixture);
  assert.equal(cases.length, 2);
  assert.equal(cases[0].stdout, stdout);
  assert.equal(cases[0].stderr, "");
  assert.equal(cases[0].status, 2);
  assertOfficialDiagnostics(cases[0], original, false, result());
  assertOfficialDiagnostics(row, original, false, result());
  assertOfficialDiagnostics(cases[0], original, true, result(original.baselineStdout));
  cases[0].files["App.vue"] = "changed";
  assert.equal(original.files["App.vue"], files["App.vue"]);
});

test("whole diagnostics reject same-code replacement, relocated range and appended output", () => {
  for (const output of [
    stdout.replace("'number'", "'boolean'"),
    stdout.replace("(1,7)", "(1,8)"),
    stdout + "unparsed compiler output\n",
  ])
    assert.throws(
      () => assertOfficialDiagnostics(row, original, false, result(output)),
      /whole stdout/u,
    );
  assert.throws(
    () => assertOfficialDiagnostics(row, original, true, result()),
    assert.AssertionError,
  );
  assert.throws(
    () =>
      assertOfficialDiagnostics(
        row,
        original,
        true,
        result(original.baselineStdout!.replace("false positive", "different error")),
      ),
    /whole stdout/u,
  );
});

test("strict controls never accept exit success, stderr, signal or process failure", () => {
  for (const outcome of [
    result(stdout, "", 0),
    result(stdout, "warning\n"),
    { ...result(), signal: "SIGTERM" },
    { ...result(), error: { code: "ENOENT" } },
  ])
    assert.throws(() => assertOfficialDiagnostics(row, original, false, outcome));
  const positive = { ...row, codes: [], stdout: "", status: 0 };
  assertOfficialDiagnostics(positive, original, false, result("", "", 0));
  assert.throws(() =>
    assertOfficialDiagnostics(positive, original, false, result("unexpected\n", "", 0)),
  );
});

test("legacy CSS checks retain authored-range, code multiset and referenced-owner build laws", () => {
  const legacy = { ...row, stdout: undefined, status: undefined };
  assertOfficialDiagnostics(legacy, original, false, result());
  for (const output of [
    stdout.replace("App.vue", "generated.ts"),
    stdout.replace("(1,7)", "(20,7)"),
    stdout.replace("(1,7)", "(1,99)"),
    stdout + stdout,
  ])
    assert.throws(() => assertOfficialDiagnostics(legacy, original, false, result(output)));
  const build = { ...legacy, build: true };
  assert.throws(() => assertOfficialDiagnostics(build, original, false, result()));
});

test("fixture admission rejects malformed exact fields, status downgrades and escaping input files", () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "official-fixture-schema-"));
  try {
    const fixture = {
      typescript: "7.1.0-dev.20261017.1",
      vue: "3.5.20",
      baselineVize: "0.463.0",
      originals: [original],
      controls: [row],
    };
    const file = path.join(directory, "fixture.json");
    const load = (value: unknown) => {
      fs.writeFileSync(file, JSON.stringify(value));
      return loadOfficialFixture(file);
    };
    assert.equal(load(fixture).originals.length, 1);
    for (const value of [
      { ...fixture, originals: [{ ...original, expectedStdout: null }] },
      { ...fixture, originals: [{ ...original, baselineStderr: [] }] },
      { ...fixture, originals: [{ ...original, baselineStatus: 0 }] },
      { ...fixture, originals: [{ ...original, expectedStatus: 0 }] },
      { ...fixture, controls: [{ ...row, stdout: 1 }] },
      { ...fixture, controls: [{ ...row, stderr: null }] },
      { ...fixture, controls: [{ ...row, status: 0 }] },
      { ...fixture, controls: [{ ...row, files: { ...files, "../escape.vue": "" } }] },
      { ...fixture, typescript: "7.1.0-dev.other" },
    ])
      assert.throws(() => load(value));
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

test("public-prop reproduction changes only the listener or the invalid prop value", () => {
  const fixture = loadOfficialFixture(
    new URL("../_fixtures/content-mapper/escaton-public-props-9498.json", import.meta.url).pathname,
  );
  assert.equal(fixture.baselineVize, "0.467.0");
  const original = fixture.originals[0];
  assert.equal(original.issue, 9498);
  assert.deepEqual(original.baselineCodes, [2353]);
  assert.deepEqual(original.expectedCodes, []);
  const control = (name: string) => {
    const row = fixture.controls.find((row) => row.name === `public-props-9498-${name}`);
    assert.ok(row, name);
    return row;
  };
  const withoutListener = control("without-listener");
  assert.deepEqual(withoutListener.files, {
    ...original.files,
    "App.vue": original.files["App.vue"].replace(' @close="() => {}"', ""),
  });
  assert.equal(withoutListener.stdout, "");
  assert.equal(withoutListener.status, 0);
  for (const [name, owner] of [
    ["wrong-allowed-prop", withoutListener],
    ["wrong-allowed-prop-listener", original],
  ] as const) {
    const invalid = control(name);
    assert.deepEqual(invalid.files, {
      ...owner.files,
      "App.vue": owner.files["App.vue"].replace(':hidden="true"', ":hidden=\"'wrong'\""),
    });
    assert.equal(invalid.status, 2);
    assert.ok(invalid.codes.every((code) => code === 2322));
  }
  const defaultAttrs = control("unknown-prop");
  const strictAttrs = control("unknown-prop-strict");
  assert.deepEqual(defaultAttrs.codes, []);
  assert.equal(defaultAttrs.status, 0);
  assert.equal(defaultAttrs.stdout, "");
  assert.equal(defaultAttrs.stderr, "");
  const strictConfig = JSON.parse(defaultAttrs.files["tsconfig.json"]);
  strictConfig.vueCompilerOptions.checkUnknownProps = true;
  assert.deepEqual(strictAttrs.files, {
    ...defaultAttrs.files,
    "tsconfig.json": `${JSON.stringify(strictConfig, null, 2)}\n`,
  });
  assert.deepEqual(strictAttrs.codes, [2353]);
  assert.equal(strictAttrs.status, 2);
  assert.equal(officialCases(fixture).length, 11);
  const workflow = fs.readFileSync(
    new URL("../../.github/workflows/content-mapper-conformance.yml", import.meta.url),
    "utf8",
  );
  const fixturePath = "tests/_fixtures/content-mapper/escaton-public-props-9498.json";
  assert.equal(workflow.split(fixturePath).length - 1, 2);
  assert.ok(workflow.includes(`--fixture ${fixturePath}`));
});
