import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
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
import { observerSourceIdentity, OBSERVER_SOURCE } from "../differential/formatter-api-build.mjs";
import { planToolingTests } from "../../tools/support/compat/github/plan-tooling-tests.mjs";

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
      options: {
        argv: fixture.optionsArgv,
        exitStatus: 0,
        signal: null,
        processError: null,
        stderrBase64: "",
        stdoutBase64: Buffer.from(JSON.stringify(fixture.effectiveOptions)).toString("base64"),
        sha256: sha256(Buffer.from(JSON.stringify(fixture.effectiveOptions))),
      },
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
              fixture.outcome === "error"
                ? Buffer.from(`error=${fixture.typedError}\n`).toString("base64")
                : fixture.api === "format_sfc"
                ? Buffer.from(`changed=${!input.equals(fixture.expected)}\n`).toString("base64")
                : "",
            exitStatus: fixture.outcome === "error" ? 1 : 0,
            signal: null,
            processError: null,
            referenceComparison: { state: "equal" },
          };
        }),
      },
    })),
    summary: {
      plannedCases: loaded.cases.length,
      legacyByteMatches: loaded.cases.filter((item) => item.contract === "full-output-bytes-and-fixed-point").length,
      legacyInternalObservations: loaded.cases.filter((item) => item.contract === "legacy-internal-observation").length,
      legacyErrorMatches: loaded.cases.filter((item) => item.contract === "typed-error-bytes").length,
      legacyFailures: 0,
      nativeUnsupported: loaded.cases.length,
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
      manifest.cases[0].options.userOverrides = { skipScriptStabilization: true };
    },
    (manifest) => {
      manifest.cases[0].options.userOverrides = { printWidth: -1 };
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
    (report) => {
      const row = report.rows[0];
      const bytes = Buffer.from(JSON.stringify({ printWidth: 7 }));
      row.options.stdoutBase64 = bytes.toString("base64");
      row.options.sha256 = sha256(bytes);
    },
  ]) {
    const report = syntheticReport(loaded);
    mutate(report);
    assert.throws(() => validateFormatterApiReport(loaded, report, {}));
  }
});

void test("source identity rejects untracked production Rust while keeping isolated test fixtures separate", (t) => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "formatter-api-source-"));
  t.after(() => fs.rmSync(dir, { recursive: true, force: true }));
  for (const [file, content] of [
    ["Cargo.lock", "fixture lock\n"],
    ["crates/vize_glyph/src/lib.rs", "// synthetic source identity fixture\n"],
    [OBSERVER_SOURCE, "// synthetic observer identity fixture\n"],
  ]) {
    fs.mkdirSync(path.dirname(path.join(dir, file)), { recursive: true });
    fs.writeFileSync(path.join(dir, file), content);
  }
  for (const argv of [
    ["init", "--quiet"],
    ["add", "."],
    [
      "-c",
      "user.name=Fixture",
      "-c",
      "user.email=fixture@example.invalid",
      "commit",
      "--quiet",
      "-m",
      "fixture",
    ],
  ]) {
    const result = spawnSync("git", argv, { cwd: dir, encoding: "utf8" });
    assert.equal(result.status, 0, result.stderr);
  }
  const identity = observerSourceIdentity(dir);
  const fixture = path.join(dir, "crates/vize_glyph/tests/fixture.rs");
  fs.mkdirSync(path.dirname(fixture), { recursive: true });
  fs.writeFileSync(fixture, "// isolated test fixture\n");
  assert.deepEqual(observerSourceIdentity(dir), identity);
  const untracked = path.join(dir, "crates/vize_glyph/src/未登録.rs");
  fs.writeFileSync(untracked, "// not a tracked source\n");
  assert.throws(() => observerSourceIdentity(dir), /untracked Rust product sources/);
  fs.rmSync(untracked);
  assert.deepEqual(observerSourceIdentity(dir), identity);
});

void test("real Cargo/API execution remains in T1 while contract and unknown input checks stay in T0", () => {
  const execution = "tests/tooling/differential-formatter-api-execution.test.mjs";
  const contract = "tests/tooling/differential-formatter-api.test.mjs";
  for (const paths of [[execution], ["unclassified/new-input.txt"], ["pnpm-lock.yaml"]]) {
    const pr = planToolingTests(paths, { cwd: root });
    assert(!pr.tests.includes(execution));
    assert(pr.tests.includes(contract));
    const merge = planToolingTests(paths, { tier: "merge", cwd: root });
    assert(merge.tests.includes(execution) && merge.tests.includes(contract));
  }
});
