import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { BUILD_RECIPE, validateBuildReceipt } from "../differential/build-receipt.mjs";
import { compareBytes } from "../differential/compare.mjs";
import { FORMATTER_ARGV, loadFormatterManifest, sha256 } from "../differential/manifest.mjs";
import {
  assertProcessSucceeded,
  formatterAttempt,
  runFormatterPack,
  validateFormatterReport,
} from "../differential/formatter.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const pack = path.join(root, "tests/_fixtures/differential/formatter");
const manifestPath = path.join(pack, "manifest.json");
const loaded = loadFormatterManifest(manifestPath);
const unitBuild = {
  sourceRevision: loaded.manifest.baseRevision,
  binaryPath: "target/ci/vize",
  binarySha256: "0".repeat(64),
  cliVersion: "vize 0.429.0",
};

function withCopy(run) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-differential-contract-"));
  try {
    fs.cpSync(pack, dir, { recursive: true });
    run(dir);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
}

// Synthetic rows test the validator only. These are never CLI capture evidence.
function referenceReport() {
  return {
    schema: "vize.differential.result",
    version: 1,
    product: "formatter",
    sourceRevision: loaded.manifest.baseRevision,
    manifestSha256: loaded.manifestSha256,
    argv: FORMATTER_ARGV,
    buildReceipt: {
      schema: "vize.differential.build",
      version: 1,
      recipe: BUILD_RECIPE,
      ...unitBuild,
    },
    binary: {
      path: unitBuild.binaryPath,
      sha256: unitBuild.binarySha256,
      version: unitBuild.cliVersion,
    },
    rows: loaded.cases.map((fixture) => ({
      id: fixture.id,
      legacy: {
        state: "completed",
        verdict: "matched-reference",
        passes: [1, 2, 3].map((pass) =>
          formatterAttempt(
            pass,
            pass === 1 ? fixture.input : fixture.expected,
            fixture.expected,
            { status: 0, signal: null, stdout: Buffer.alloc(0), stderr: Buffer.alloc(0) },
            fixture.expected,
          ),
        ),
      },
      native: { state: "unsupported", reason: fixture.adapters.reasons.native },
      comparison: { state: "not-compared", reason: "native formatter adapter unavailable" },
    })),
    summary: {
      plannedCases: 2,
      legacyMatches: 2,
      legacyFailures: 0,
      baselineDrift: 0,
      nativeUnsupported: 2,
      pairedComparisons: 0,
      nativeHandled: 0,
      nativeEquivalent: 0,
    },
  };
}

test("formatter differential manifest preserves two authored inputs and candidate-captured references", () => {
  assert.equal(loaded.cases.length, 2);
  for (const fixture of loaded.cases) {
    assert.equal(fixture.expectations.legacy.state, "captured");
    assert.equal(fixture.adapters.native, null);
    assert.equal(fixture.input.at(-1), 10);
    assert.equal(fixture.expected.at(-1), 10);
  }
  assert.equal(
    fs.readdirSync(pack, { recursive: true }).filter((name) => name.endsWith(".vue")).length,
    2,
  );
});

test("formatter differential comparison preserves every output byte and Unicode offset", () => {
  const expected = Buffer.from("あ\n");
  assert.deepEqual(compareBytes(expected, Buffer.from("あ\n")), { state: "equal" });
  for (const actual of ["あ", "あ\r\n", "あ \n", "い\n"]) {
    assert.equal(compareBytes(expected, Buffer.from(actual)).state, "different");
  }
  assert.equal(compareBytes(expected, Buffer.from("い\n")).byteOffset, 2);
  assert.throws(() => compareBytes("あ", expected), /raw buffers/);
});

test("formatter differential immutable input, output and configuration corruption fails", () => {
  for (const relative of [
    "sfc-split-v-pre-indentation/App.vue",
    "sfc-split-v-pre-indentation/reference.expected.txt",
    "format-options-reference.json",
    "capture/report.json",
  ]) {
    withCopy((dir) => {
      fs.appendFileSync(path.join(dir, relative), "corrupted");
      assert.throws(
        () => loadFormatterManifest(path.join(dir, "manifest.json")),
        /SHA256 mismatch/,
      );
    });
  }
});

test("formatter differential rejects changed adapter options and duplicate planned identities", () => {
  for (const change of [
    (manifest) => manifest.adapterOptions.argv.pop(),
    (manifest) => manifest.cases.push(manifest.cases[0]),
  ]) {
    withCopy((dir) => {
      const file = path.join(dir, "manifest.json");
      const manifest = JSON.parse(fs.readFileSync(file, "utf8"));
      change(manifest);
      fs.writeFileSync(file, JSON.stringify(manifest));
      assert.throws(() => loadFormatterManifest(file));
    });
  }
});

test("formatter differential missing executable fails all planned cases without fallback", () => {
  const report = runFormatterPack({
    manifestPath,
    sourceRevision: loaded.manifest.baseRevision,
    binaryPath: path.join(pack, "missing-source-built-vize"),
  });
  assert.equal(report.binary, null);
  assert.deepEqual(report.summary, {
    plannedCases: 2,
    legacyMatches: 0,
    legacyFailures: 2,
    baselineDrift: 0,
    nativeUnsupported: 2,
    pairedComparisons: 0,
    nativeHandled: 0,
    nativeEquivalent: 0,
  });
  for (const row of report.rows) assert.match(row.legacy.error, /ENOENT/);
  const relative = runFormatterPack({
    manifestPath,
    sourceRevision: loaded.manifest.baseRevision,
    binaryPath: "vize",
  });
  assert.equal(relative.summary.legacyFailures, 2);
  assert.match(relative.rows[0].legacy.error, /absolute path/);
});

test("formatter differential process failures and signals are failures", () => {
  assert.throws(
    () => assertProcessSucceeded({ status: 2, signal: null, stderr: Buffer.from("failed") }),
    /exited 2/,
  );
  assert.throws(() => assertProcessSucceeded({ status: null, signal: "SIGTERM" }), /terminated/);
  assert.throws(() => assertProcessSucceeded({ error: new Error("timed out") }), /timed out/);
});

test("formatter differential accounting rejects omitted, duplicate and unplanned result rows", () => {
  validateFormatterReport(loaded, referenceReport(), unitBuild);
  for (const mutate of [
    (report) => report.rows.pop(),
    (report) => {
      report.rows[1] = report.rows[0];
    },
    (report) => {
      report.rows[1].id = "formatter/unplanned";
    },
    (report) => {
      report.summary.nativeEquivalent = 1;
    },
    (report) => {
      report.rows[0].native.state = "completed";
    },
    (report) => {
      report.rows[0].legacy.passes.pop();
    },
  ]) {
    const report = referenceReport();
    mutate(report);
    assert.throws(() => validateFormatterReport(loaded, report, unitBuild));
  }
});

test("formatter differential report cannot forge byte equality by changing the digest", () => {
  const report = referenceReport();
  const pass = report.rows[0].legacy.passes[0];
  const output = Buffer.from(pass.outputBase64, "base64");
  output[0] ^= 1;
  pass.outputBase64 = output.toString("base64");
  pass.outputSha256 = sha256(output);
  assert.throws(
    () => validateFormatterReport(loaded, report, unitBuild),
    /forged reference comparison/,
  );
});

test("formatter differential keeps failed attempt input/output bytes and raw process evidence", () => {
  const fixture = loaded.cases[0];
  const attempt = formatterAttempt(
    1,
    fixture.input,
    fixture.input,
    {
      status: 2,
      signal: null,
      stdout: Buffer.from("partial\r\n"),
      stderr: Buffer.from("failure\n"),
    },
    fixture.expected,
  );
  assert.equal(attempt.inputSha256, sha256(fixture.input));
  assert.equal(attempt.outputSha256, sha256(fixture.input));
  assert.equal(Buffer.from(attempt.stderrBase64, "base64").toString(), "failure\n");
  assert.equal(Buffer.from(attempt.stdoutBase64, "base64").toString(), "partial\r\n");
  const report = referenceReport();
  report.rows[0].legacy = {
    state: "failed",
    verdict: "failed",
    passes: [attempt],
    error: "formatter exited 2",
  };
  report.summary.legacyMatches = 1;
  report.summary.legacyFailures = 1;
  validateFormatterReport(loaded, report, unitBuild);
});

test("formatter differential rejects broken pass input linkage even with updated hashes", () => {
  const report = referenceReport();
  const pass = report.rows[0].legacy.passes[1];
  const bytes = Buffer.from(pass.inputBase64, "base64");
  bytes[0] ^= 1;
  pass.inputBase64 = bytes.toString("base64");
  pass.inputSha256 = sha256(bytes);
  assert.throws(() => validateFormatterReport(loaded, report, unitBuild), /previous output/);
});

test("formatter differential rejects stale source/build and forged executable identity", () => {
  const receipt = referenceReport().buildReceipt;
  validateBuildReceipt(receipt, unitBuild);
  for (const field of ["sourceRevision", "binaryPath", "binarySha256", "cliVersion", "recipe"]) {
    assert.throws(() => validateBuildReceipt({ ...receipt, [field]: "changed" }, unitBuild));
  }
  for (const mutate of [
    (report) => {
      report.sourceRevision = "1".repeat(40);
    },
    (report) => {
      report.binary.sha256 = "1".repeat(64);
    },
    (report) => {
      report.binary.path = "unrelated/vize";
    },
    (report) => {
      report.binary.version = "vize 0.0.0";
    },
    (report) => {
      report.buildReceipt = null;
    },
  ]) {
    const report = referenceReport();
    mutate(report);
    assert.throws(() => validateFormatterReport(loaded, report, unitBuild));
  }
});
