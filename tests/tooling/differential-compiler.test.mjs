import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { summarizeNativeAcceptance } from "../differential/acceptance-rates.mjs";
import { BUILD_RECIPE } from "../differential/build-receipt.ts";
import {
  COMPILER_ARGV,
  compilerAttempt,
  loadCompilerManifest,
  runCompilerPack,
  validateCompilerReport,
} from "../differential/compiler.mjs";
import { sha256 } from "../differential/harness.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const pack = path.join(root, "tests/_fixtures/differential/compiler");
const manifestPath = path.join(pack, "manifest.json");
const loaded = loadCompilerManifest(manifestPath);
const fixture = loaded.cases[0];
const sourceRevision = loaded.manifest.baseRevision;
const build = {
  sourceRevision,
  binaryPath: "target/ci/vize",
  binarySha256: "0".repeat(64),
  cliVersion: "vize 0.429.0-dev",
};

function syntheticReport(code = fixture.expected) {
  const rows = loaded.cases.map((current, index) => {
    const output = Buffer.from(
      JSON.stringify({
        filename: "Layout.vue",
        code: (index === 0 ? code : current.expected).toString(),
        css: current.expectedCss?.toString() ?? null,
        errors: [],
        warnings: [],
        script_lang: current.comparison.scriptLang ?? "js",
        macro_artifacts: [],
      }),
    );
    const observation = compilerAttempt(
      { status: 0, signal: null, stdout: Buffer.alloc(0), stderr: Buffer.alloc(0) },
      output,
      current.expected,
      current.comparison.scriptLang,
      current.expectedCss,
    );
    const verdict =
      observation.referenceComparison.state === "equal" &&
      (observation.cssReferenceComparison === null ||
        observation.cssReferenceComparison.state === "equal")
        ? "matched-reference"
        : "baseline-drift";
    return {
      id: current.id,
      target: "ssr",
      legacy: {
        state: "completed",
        verdict,
        argv: COMPILER_ARGV,
        inputSha256: sha256(current.input),
        observation,
      },
      native: { state: "unsupported", reason: current.adapters.reasons.native },
      comparison: {
        state: "not-compared",
        reason: "whole-product native compiler path unavailable",
      },
    };
  });
  const baselineDrift = rows.filter((row) => row.legacy.verdict === "baseline-drift").length;
  return {
    schema: "vize.differential.result",
    version: 1,
    product: "compiler",
    manifestSha256: loaded.manifestSha256,
    sourceRevision,
    argv: COMPILER_ARGV,
    buildReceipt: { schema: "vize.differential.build", version: 1, recipe: BUILD_RECIPE, ...build },
    binary: { path: build.binaryPath, sha256: build.binarySha256, version: build.cliVersion },
    rows,
    summary: {
      plannedCases: rows.length,
      legacyMatches: rows.length - baselineDrift,
      legacyFailures: 0,
      baselineDrift,
      nativeUnsupported: rows.length,
      pairedComparisons: 0,
      nativeHandled: 0,
      nativeEquivalent: 0,
    },
  };
}

void test("compiler fixture pins a real full-module reference without native credit", () => {
  assert.deepEqual(
    loaded.cases.map((current) => current.id),
    [
      "compiler/sfc/ssr-slot-scope",
      "compiler/sfc/slot-fallback-compound-text-9889",
      "compiler/sfc/slot-fallback-adjacent-text-10087",
      "compiler/sfc/production-v-if-comment-9920",
      "compiler/sfc/template-asset-urls-9895",
      "compiler/sfc/v-model-maybe-ref-9897",
      "compiler/sfc/maybe-ref-model-parity-10044",
      "compiler/sfc/maybe-ref-authored-handlers-10164",
      "compiler/sfc/vue2-preserve-boundaries-9886",
      "compiler/sfc/vue2-preserve-comment-edges-10065",
      "compiler/sfc/package-import-prop-type-9894",
      "compiler/sfc/scoped-where-nesting-9830",
      "compiler/sfc/scoped-keyframes-nesting-9829",
      "compiler/sfc/scoped-checkout-stability-10067",
      "compiler/sfc/jsx-vapor-runtime-slots-6889",
      "compiler/sfc/root-text-patch-flag-10168",
    ],
  );
  const report = syntheticReport();
  assert.deepEqual(validateCompilerReport(loaded, report, build), report.summary);
  assert.deepEqual(summarizeNativeAcceptance(loaded, report, { sourceRevision }).total, {
    planned: 15,
    nativeHandled: 0,
    nativeEquivalent: 0,
    unsupported: 15,
    legacyBacked: 0,
    unverified: 0,
  });
  assert.equal(
    loaded.cases.find((current) => current.id === "compiler/sfc/package-import-prop-type-9894")
      .comparison.scriptLang,
    "ts",
  );
  assert(
    loaded.cases
      .filter((current) => current.id.includes("scoped-"))
      .every((current) => Buffer.isBuffer(current.expectedCss)),
  );
});

void test("compiler comparison checks every code byte and rejects forged native completion", () => {
  const changed = syntheticReport(Buffer.concat([fixture.expected, Buffer.from("\n")]));
  assert.deepEqual(validateCompilerReport(loaded, changed, build), changed.summary);
  assert.equal(changed.rows[0].legacy.observation.referenceComparison.state, "different");
  const forged = syntheticReport();
  forged.rows[0].native = { state: "completed" };
  assert.throws(() => validateCompilerReport(loaded, forged, build));
  forged.rows[0].native = { state: "unsupported", reason: fixture.adapters.reasons.native };
  forged.rows[0].comparison.state = "equal";
  assert.throws(() => validateCompilerReport(loaded, forged, build));
  const tampered = syntheticReport();
  tampered.rows[0].legacy.observation.codeSha256 = "f".repeat(64);
  assert.throws(() => validateCompilerReport(loaded, tampered, build), /forged/);

  const cssFixture = loaded.cases.find((current) => current.expectedCss !== null);
  assert(cssFixture);
  const cssOutput = Buffer.from(
    JSON.stringify({
      filename: "Layout.vue",
      code: cssFixture.expected.toString(),
      css: `${cssFixture.expectedCss.toString()}\n`,
      errors: [],
      warnings: [],
      script_lang: "js",
      macro_artifacts: [],
    }),
  );
  const cssChanged = compilerAttempt(
    { status: 0, signal: null, stdout: Buffer.alloc(0), stderr: Buffer.alloc(0) },
    cssOutput,
    cssFixture.expected,
    "js",
    cssFixture.expectedCss,
  );
  assert.equal(cssChanged.referenceComparison.state, "equal");
  assert.equal(cssChanged.cssReferenceComparison.state, "different");
});

void test("compiler input and reference digests fail closed", () => {
  for (const relative of loaded.cases.flatMap((current) => [
    `${current.inputs.root}/Layout.vue.txt`,
    `${current.inputs.root}/module.expected.txt`,
    ...(current.reference.cssPath ? [current.reference.cssPath] : []),
  ])) {
    const copy = fs.mkdtempSync(path.join(os.tmpdir(), "vize-compiler-manifest-"));
    try {
      fs.cpSync(pack, copy, { recursive: true });
      fs.appendFileSync(path.join(copy, relative), "drift");
      assert.throws(
        () => loadCompilerManifest(path.join(copy, "manifest.json")),
        /SHA256 mismatch/,
      );
    } finally {
      fs.rmSync(copy, { recursive: true, force: true });
    }
  }
});

void test("compiler plans and byte-compares every registered SSR case", () => {
  const copy = fs.mkdtempSync(path.join(os.tmpdir(), "vize-compiler-multiple-"));
  try {
    fs.cpSync(pack, copy, { recursive: true });
    fs.cpSync(path.join(copy, "ssr-slot-scope"), path.join(copy, "second-slot-scope"), {
      recursive: true,
    });
    const manifest = JSON.parse(fs.readFileSync(path.join(copy, "manifest.json"), "utf8"));
    const second = structuredClone(manifest.cases[0]);
    second.id = "compiler/sfc/second-slot-scope";
    second.inputs.root = "second-slot-scope";
    second.reference.path = "second-slot-scope/module.expected.txt";
    manifest.cases.push(second);
    fs.writeFileSync(path.join(copy, "manifest.json"), JSON.stringify(manifest));
    const multiple = loadCompilerManifest(path.join(copy, "manifest.json"));
    assert.equal(multiple.cases.length, loaded.cases.length + 1);

    const report = syntheticReport();
    report.manifestSha256 = multiple.manifestSha256;
    report.rows.push({ ...structuredClone(report.rows[0]), id: second.id });
    report.summary = {
      ...report.summary,
      plannedCases: report.summary.plannedCases + 1,
      legacyMatches: report.summary.legacyMatches + 1,
      nativeUnsupported: report.summary.nativeUnsupported + 1,
    };
    assert.deepEqual(validateCompilerReport(multiple, report, build), report.summary);

    const changed = structuredClone(report);
    const output = JSON.parse(
      Buffer.from(changed.rows[1].legacy.observation.jsonBase64, "base64").toString("utf8"),
    );
    output.code += "\n";
    changed.rows[1].legacy.observation = compilerAttempt(
      { status: 0, signal: null, stdout: Buffer.alloc(0), stderr: Buffer.alloc(0) },
      Buffer.from(JSON.stringify(output)),
      multiple.cases[1].expected,
    );
    changed.rows[1].legacy.verdict = "baseline-drift";
    changed.summary.legacyMatches -= 1;
    changed.summary.baselineDrift += 1;
    assert.deepEqual(validateCompilerReport(multiple, changed, build), changed.summary);
    assert.throws(() =>
      validateCompilerReport(multiple, { ...report, rows: report.rows.slice(0, -1) }, build),
    );

    const unavailable = runCompilerPack({
      manifestPath: path.join(copy, "manifest.json"),
      binaryPath: path.join(root, "target/ci/missing-vize"),
      sourceRevision,
      repoRoot: root,
    });
    assert.equal(unavailable.rows.length, loaded.cases.length + 1);
    assert.equal(unavailable.summary.legacyFailures, loaded.cases.length + 1);
  } finally {
    fs.rmSync(copy, { recursive: true, force: true });
  }
});

void test("malformed compiler output retains raw evidence and cannot complete", () => {
  const raw = Buffer.from("{broken JSON");
  const observation = compilerAttempt(
    { status: 0, signal: null, stdout: Buffer.from("log"), stderr: Buffer.from("warning") },
    raw,
    fixture.expected,
  );
  assert.equal(observation.jsonBase64, raw.toString("base64"));
  assert.equal(observation.jsonSha256, sha256(raw));
  assert.equal(observation.stdoutBase64, Buffer.from("log").toString("base64"));
  assert.equal(typeof observation.facetError, "string");
  const report = syntheticReport();
  report.rows[0].legacy = {
    ...report.rows[0].legacy,
    state: "failed",
    verdict: "failed",
    error: observation.facetError,
    observation,
  };
  report.summary = {
    ...report.summary,
    legacyMatches: loaded.cases.length - 1,
    legacyFailures: 1,
  };
  assert.deepEqual(validateCompilerReport(loaded, report, build), report.summary);
  const forged = structuredClone(report);
  forged.rows[0].legacy.state = "completed";
  assert.throws(() => validateCompilerReport(loaded, forged, build));
  const arrayCode = Buffer.from(
    JSON.stringify({
      filename: "Layout.vue",
      code: [65, 66],
      css: null,
      errors: [],
      warnings: [],
      script_lang: "js",
      macro_artifacts: [],
    }),
  );
  const malformed = compilerAttempt(
    { status: 0, signal: null, stdout: Buffer.alloc(0), stderr: Buffer.alloc(0) },
    arrayCode,
    fixture.expected,
  );
  assert.match(malformed.facetError, /compiler output code must be a string/);
  assert.equal(malformed.jsonBase64, arrayCode.toString("base64"));
  assert.equal(malformed.referenceComparison, null);
});

void test("missing source-built CLI retains the planned failed row", () => {
  const report = runCompilerPack({
    manifestPath,
    binaryPath: path.join(root, "target/ci/missing-vize"),
    sourceRevision,
    repoRoot: root,
  });
  assert.equal(report.rows.length, loaded.cases.length);
  assert.equal(report.rows[0].legacy.state, "failed");
  assert.equal(report.rows[0].native.state, "unsupported");
  assert.equal(report.summary.nativeHandled, 0);
});
