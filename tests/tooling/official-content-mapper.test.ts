import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import {
  assertOfficialDiagnostics,
  loadOfficialFixture,
  officialCases,
  officialRunOptions,
  resolveOfficialNativeImage,
  officialActorInputs,
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

test("published subject H and trusted tooling actor F remain independent exact identities", () => {
  const values = {
    PUBLIC_CUT: "a".repeat(40),
    PUBLIC_HEAD: "b".repeat(40),
    PUBLIC_TAG: "v0.470.0",
    PUBLIC_SOURCE_PR: "9503",
    PUBLIC_RELEASE_RUN: "123",
    PUBLIC_ACTOR: "c".repeat(40),
  };
  const selected = officialActorInputs(values);
  assert.equal(selected.actor, values.PUBLIC_ACTOR);
  assert.equal(selected.subject.head, values.PUBLIC_HEAD);
  assert.notEqual(selected.actor, selected.subject.head);
  assert.equal(
    officialActorInputs({ ...values, PUBLIC_ACTOR: values.PUBLIC_HEAD }).actor,
    values.PUBLIC_HEAD,
  );
  for (const invalid of [undefined, "", "main", "c".repeat(39)])
    assert.throws(() => officialActorInputs({ ...values, PUBLIC_ACTOR: invalid }));
  assert.throws(() => officialActorInputs({ ...values, PUBLIC_HEAD: "main" }));
});

test("9499 published adapter preserves all fourteen H-authored fixtures without inventing negative streams", () => {
  const source = fs.readFileSync(
    new URL("../_fixtures/content-mapper/with-defaults-surplus.json", import.meta.url),
    "utf8",
  );
  const owned = JSON.parse(source) as {
    issue: number;
    cases: Pick<Case, "name" | "files" | "codes">[];
  };
  const fixture = loadOfficialFixture("unread-live-path.json", source);
  const projected = officialCases(fixture);
  assert.equal(projected.length, 14);
  assert.equal(owned.cases.length, projected.length);
  for (let index = 0; index < owned.cases.length; index++) {
    const authored = owned.cases[index],
      row = projected[index];
    for (const [file, text] of Object.entries(authored.files)) assert.equal(row.files[file], text);
    assert.deepEqual(
      Object.keys(row.files).sort(),
      ["tsconfig.json", ...Object.keys(authored.files)].sort(),
    );
    assert.deepEqual(row.codes, authored.codes);
    assert.equal(row.status, authored.codes.length ? 2 : 0);
    assert.equal(row.stderr, "");
    assert.equal(row.stdout, authored.codes.length ? undefined : "");
    assert.deepEqual(JSON.parse(row.files["tsconfig.json"]), {
      compilerOptions: {
        strict: true,
        module: "ESNext",
        moduleResolution: "bundler",
        skipLibCheck: true,
      },
      contentMappers: [{ package: "vize", extensions: [".vue"] }],
      include: ["*.vue"],
    });
  }
  assert.equal(projected.filter((row) => row.codes.length === 0).length, 10);
  assert.equal(
    projected.filter((row) => row.codes.length === 1 && row.codes[0] === 2322).length,
    4,
  );
  assert.throws(() =>
    loadOfficialFixture("unread-live-path.json", JSON.stringify({ ...owned, issue: 9504 })),
  );
});

test("public-only selection requires exact explicit selectors and never falls back to old fixtures", () => {
  const root = path.join(os.tmpdir(), "official-owned");
  const defaults = path.join(root, "css.json");
  const legacy = officialRunOptions(
    ["--public-version", "0.467.0", "--fixture", "events.json", "--fixture", "props.json"],
    root,
    defaults,
  );
  assert.deepEqual(legacy.fixturePaths, [
    defaults,
    path.join(root, "events.json"),
    path.join(root, "props.json"),
  ]);
  assert.equal(legacy.candidateOnly, false);
  const selected = [
    "--candidate-only",
    "--explicit-fixtures-only",
    "--public-version",
    "0.470.0",
    "--public-inputs",
    "/authority/tuple.json",
    "--fixture",
    "new.json",
  ];
  const admitted = officialRunOptions(selected, root, defaults);
  assert.deepEqual(admitted.fixturePaths, [path.join(root, "new.json")]);
  assert.equal(admitted.candidateOnly, true);
  for (const invalid of [
    selected.filter((arg) => arg !== "--candidate-only"),
    selected.filter((arg) => arg !== "--explicit-fixtures-only"),
    selected.slice(0, -2),
    selected.slice(0, 4),
    [...selected, "--baseline-only"],
    [...selected, "--candidate-ony"],
    [...selected, "--fixture"],
    [...selected, "--fixture", "new.json"],
    [...selected, "--fixture", "../foreign.json"],
    [...selected, "--public-version", "0.470.0"],
    selected.map((arg) => (arg === "0.470.0" ? "latest" : arg)),
    selected.map((arg) => (arg === "--public-inputs" ? "--unknown-inputs" : arg)),
  ])
    assert.throws(() => officialRunOptions(invalid, root, defaults));
});

test("all TypeScript error headers require authored mappings even without a whole-stream golden", () => {
  const legacy = { ...row, stdout: undefined };
  assertOfficialDiagnostics(legacy, original, false, result());
  for (const output of [
    stdout + "error TS2344: Unmapped synthetic constraint.\n",
    "error TS2353: Unmapped virtual property.\n" + stdout,
  ])
    assert.throws(
      () => assertOfficialDiagnostics(legacy, original, false, result(output)),
      /every TypeScript error must map to authored source/u,
    );
});

test("public identity resolves an inert native archive without evaluating its wrapper or image", () => {
  for (const scenario of ["none", "one", "ambiguous", "foreign"] as const) {
    const directory = fs.mkdtempSync(path.join(os.tmpdir(), "official-native-resolution-"));
    const outside = fs.mkdtempSync(path.join(os.tmpdir(), "official-foreign-native-"));
    const native = path.join(directory, "node_modules/@vizejs/native");
    fs.mkdirSync(native, { recursive: true });
    const names = ["@vizejs/native-darwin-arm64", "@vizejs/native-linux-x64-gnu"];
    fs.writeFileSync(
      path.join(native, "package.json"),
      JSON.stringify({
        name: "@vizejs/native",
        main: "index.js",
        optionalDependencies: Object.fromEntries(names.map((name) => [name, "1.0.0"])),
      }),
    );
    fs.writeFileSync(
      path.join(native, "index.js"),
      "throw new Error('must never evaluate native wrapper')",
    );
    const image = "vize-vitrine.darwin-arm64.node";
    const install = (name: string) => {
      const target = path.join(directory, "node_modules", name);
      fs.mkdirSync(target, { recursive: true });
      fs.writeFileSync(path.join(target, "package.json"), JSON.stringify({ name, main: image }));
      fs.writeFileSync(path.join(target, image), "not a native executable");
      return target;
    };
    try {
      let first: string | undefined;
      if (scenario === "one" || scenario === "ambiguous") first = install(names[0]);
      if (scenario === "ambiguous") install(names[1]);
      if (scenario === "foreign") {
        fs.writeFileSync(
          path.join(outside, "package.json"),
          JSON.stringify({ name: names[0], main: image }),
        );
        fs.writeFileSync(path.join(outside, image), "not a native executable");
        fs.symlinkSync(
          outside,
          path.join(directory, "node_modules", names[0]),
          process.platform === "win32" ? "junction" : "dir",
        );
      }
      if (scenario === "one")
        assert.equal(
          resolveOfficialNativeImage(directory),
          fs.realpathSync(path.join(first!, image)),
        );
      else assert.throws(() => resolveOfficialNativeImage(directory));
    } finally {
      fs.rmSync(directory, { recursive: true, force: true });
      fs.rmSync(outside, { recursive: true, force: true });
    }
  }
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
