import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  loadFormatterApiManifest,
  validateFormatterApiReport,
} from "../differential/formatter-api.mjs";
import { sha256 } from "../differential/manifest.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const manifestPath = path.join(
  root,
  "tests/_fixtures/differential/formatter-history/script-manifest.json",
);

function alteredManifest(t, mutate) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "formatter-api-contract-"));
  t.after(() => fs.rmSync(dir, { recursive: true, force: true }));
  const manifest = JSON.parse(fs.readFileSync(manifestPath, "utf8"));
  mutate(manifest);
  const file = path.join(dir, "manifest.json");
  fs.writeFileSync(file, JSON.stringify(manifest));
  return () => loadFormatterApiManifest(file, root);
}

// Synthetic rows exercise the validator; only the execution test observes APIs.
function syntheticReport(loaded) {
  return {
    schema: "vize.differential.formatter-api-result",
    version: 1,
    manifestSha256: loaded.manifestSha256,
    buildReceipt: {},
    rows: loaded.cases.map((fixture) => ({
      id: fixture.id,
      argv: fixture.argv,
      contract: fixture.contract,
      native: { state: "unsupported", reason: "native formatter adapter unavailable" },
      comparison: { state: "not-compared" },
      legacy: {
        state: "matched-reference",
        passes: Array.from({ length: fixture.passCount }, (_, index) => {
          const input = index ? fixture.expected : fixture.input;
          return {
            pass: index + 1,
            inputBase64: input.toString("base64"),
            inputSha256: sha256(input),
            stdoutBase64: fixture.expected.toString("base64"),
            outputSha256: sha256(fixture.expected),
            stderrBase64:
              fixture.api === "format_sfc"
                ? Buffer.from(`changed=${!input.equals(fixture.expected)}\n`).toString("base64")
                : "",
            exitStatus: 0,
            signal: null,
            processError: null,
            referenceComparison: { state: "equal" },
          };
        }),
      },
    })),
    summary: {
      plannedCases: 6,
      legacyByteMatches: 4,
      legacyInternalObservations: 2,
      legacyFailures: 0,
      nativeUnsupported: 6,
      nativeHandled: 0,
      nativeEquivalent: 0,
      pairedComparisons: 0,
    },
  };
}

void test("API fixture loader rejects drift, duplicate plans and unregistered options/APIs", (t) => {
  for (const mutate of [
    (manifest) => {
      manifest.cases[0].input.sha256 = "0".repeat(64);
    },
    (manifest) => {
      manifest.cases.push(manifest.cases[0]);
    },
    (manifest) => {
      manifest.cases[0].api = "private_helper";
    },
    (manifest) => {
      manifest.cases[0].options.internalOverrides.unknown = true;
    },
    (manifest) => {
      manifest.cases[0].input.path = "/etc/hosts";
    },
    (manifest) => {
      manifest.nativeHandled = 1;
    },
  ])
    assert.throws(alteredManifest(t, mutate));
});

void test("API result validator rejects omitted/extra/duplicate rows and invented native credit", () => {
  const loaded = loadFormatterApiManifest(manifestPath, root);
  for (const mutate of [
    (report) => {
      report.rows.pop();
    },
    (report) => {
      report.rows.push(report.rows[0]);
    },
    (report) => {
      report.rows[1].id = report.rows[0].id;
    },
    (report) => {
      report.summary.nativeHandled = 1;
    },
    (report) => {
      report.rows[0].native.state = "handled";
    },
  ]) {
    const report = syntheticReport(loaded);
    mutate(report);
    assert.throws(() => validateFormatterApiReport(loaded, report, {}));
  }
});

void test("API result validator requires complete bytes, actual pass chains and real process success", () => {
  const loaded = loadFormatterApiManifest(manifestPath, root);
  assert.equal(
    validateFormatterApiReport(loaded, syntheticReport(loaded), {}).legacyByteMatches,
    4,
  );
  for (const mutate of [
    (report) => {
      report.rows[0].legacy.passes.pop();
    },
    (report) => {
      report.rows[0].legacy.passes[1].inputBase64 = Buffer.from("other input").toString("base64");
    },
    (report) => {
      report.rows[0].legacy.passes[0].stdoutBase64 =
        Buffer.from("partial output").toString("base64");
    },
    (report) => {
      report.rows[0].legacy.passes[0].referenceComparison.state = "different";
    },
    (report) => {
      report.rows[0].legacy.passes[0].exitStatus = 1;
    },
    (report) => {
      report.rows[0].legacy.passes[0].processError = "ENOENT";
    },
    (report) => {
      report.rows[2].legacy.passes[0].stderrBase64 = "";
    },
  ]) {
    const report = syntheticReport(loaded);
    mutate(report);
    assert.throws(() => validateFormatterApiReport(loaded, report, {}));
  }
});
