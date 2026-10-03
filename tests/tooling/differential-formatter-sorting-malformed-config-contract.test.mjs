import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  loadSortingConfigManifest,
  validateSortingConfigReport,
} from "../differential/formatter-sorting-config.mjs";
import { retainedFormatterFunction } from "../differential/formatter-history-source-artifact.ts";
import { sha256 } from "../differential/manifest.mjs";
import { planToolingTests } from "../../tools/support/compat/github/plan-tooling-tests.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const manifestPath =
  "tests/_fixtures/differential/formatter-history/import-sorting-malformed-config-manifest.json";
const observation = (bytes) => ({ base64: bytes.toString("base64"), sha256: sha256(bytes) });
const snapshot = (files) =>
  Object.fromEntries(Object.entries(files).map(([key, bytes]) => [key, observation(bytes)]));

// Synthetic control frames test admission only; they are never runtime evidence.
function control(loaded) {
  const workspace = path.join(os.tmpdir(), "malformed-config-contract");
  const rows = loaded.cases.map((fixture) => {
    const initialFiles = snapshot(fixture.initialFiles);
    let previous = initialFiles;
    const passes = fixture.steps.map((step) => {
      const pass = {
        argv: step.argv,
        inputFiles: previous,
        outputFiles: snapshot(step.expectedFiles),
        outputSnapshotError: null,
        stdout: observation(step.stdout),
        stderr: observation(
          Buffer.from(step.stderr.toString().replaceAll("{{workspace}}", workspace)),
        ),
        exitStatus: step.exitStatus,
        signal: null,
        processError: null,
      };
      previous = pass.outputFiles;
      return pass;
    });
    return {
      id: fixture.id,
      workspace,
      initialFiles,
      state: "matched-reference",
      native: "unsupported",
      passes,
    };
  });
  return {
    schema: "vize.formatter-sorting-config-result",
    version: 1,
    manifestSha256: loaded.manifestSha256,
    buildReceipt: {},
    rows,
    summary: {
      plannedCases: 8,
      legacyMatches: 8,
      legacyFailures: 0,
      nativeUnsupported: 8,
      nativeHandled: 0,
      nativeEquivalent: 0,
      pairedComparisons: 0,
    },
  };
}

void test("malformed history retains all original named-law settings and complete protected/bypass plans", () => {
  const loaded = loadSortingConfigManifest(root, manifestPath);
  assert.deepEqual(
    loaded.cases.map(({ id }) => id.split("/")[1]),
    [
      "groups-wrong-type",
      "unknown-newline-key",
      "unknown-custom-key",
      "leading-boundary",
      "trailing-boundary",
      "consecutive-boundaries",
      "invalid-json",
      "contradictory-partition",
    ],
  );
  assert.equal(
    loaded.cases.reduce((sum, { steps }) => sum + steps.length, 0),
    16,
  );
  for (const fixture of loaded.cases) {
    assert.deepEqual(
      fixture.steps.map(({ argv, exitStatus }) => ({ argv, exitStatus })),
      [
        { argv: ["fmt", "--write", "UserCard.vue"], exitStatus: 2 },
        { argv: ["fmt", "--no-config", "--check", "UserCard.vue"], exitStatus: 0 },
      ],
    );
    for (const step of fixture.steps) {
      assert.deepEqual(step.expectedFiles, fixture.initialFiles);
      assert.equal(step.stdout.length, 0);
      assert.equal(step.formatted, false);
    }
  }
  assert.deepEqual(
    validateSortingConfigReport(loaded, control(loaded), {}),
    control(loaded).summary,
  );
});

void test("malformed admission rejects lost complete errors, bypasses and file protection", () => {
  const loaded = loadSortingConfigManifest(root, manifestPath);
  for (const mutate of [
    (report) => report.rows.pop(),
    (report) => report.rows[0].passes.shift(),
    (report) => (report.rows[0].passes[0].exitStatus = 0),
    (report) => (report.rows[0].passes[1].argv = ["fmt", "--check", "UserCard.vue"]),
    (report) => (report.rows[0].passes[0].stderr = observation(Buffer.alloc(0))),
    (report) => (report.rows[0].passes[0].stderr.sha256 = "0".repeat(64)),
    (report) =>
      (report.rows[0].passes[0].outputFiles["UserCard.vue"] = observation(Buffer.from("changed"))),
    (report) =>
      (report.rows[0].passes[0].outputFiles["vize.config.json"] = observation(Buffer.from("{}"))),
    (report) => (report.rows[0].passes[1].inputFiles = {}),
    (report) => (report.rows[0].passes[1].signal = "SIGTERM"),
    (report) => (report.rows[0].passes[1].processError = "spawn failed"),
    (report) => (report.summary.nativeHandled = 1),
  ]) {
    const report = structuredClone(control(loaded));
    mutate(report);
    assert.throws(() => validateSortingConfigReport(loaded, report, {}));
  }
});

void test("configuration source authority binds exact original literal bytes inside the named function", (t) => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "malformed-history-control-"));
  t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
  for (const mutate of [
    (manifest) => (manifest.providerConfigManifest.sha256 = "0".repeat(64)),
    (manifest) => {
      delete manifest.profile;
    },
    (manifest) => {
      delete manifest.cases[0].historicalConfig;
    },
    (manifest) => {
      delete manifest.cases[0].witness.functionSha256;
    },
    (manifest) => {
      const first = manifest.cases[0],
        sibling = manifest.cases[1];
      delete first.historicalConfig;
      first.initialFiles["vize.config.json"] = sibling.initialFiles["vize.config.json"];
      first.steps.forEach((step, index) => {
        step.files["vize.config.json"] = sibling.steps[index].files["vize.config.json"];
        step.stderr = sibling.steps[index].stderr;
      });
    },
    (manifest) => {
      const first = manifest.cases[0],
        sibling = manifest.cases[1];
      first.historicalConfig = sibling.historicalConfig;
      first.initialFiles["vize.config.json"] = sibling.initialFiles["vize.config.json"];
      first.steps.forEach((step, index) => {
        step.files["vize.config.json"] = sibling.steps[index].files["vize.config.json"];
        step.stderr = sibling.steps[index].stderr;
      });
    },
    (manifest) => (manifest.cases[0].witness.functionSha256 = "0".repeat(64)),
    (manifest) =>
      (manifest.cases[0].historicalConfig.literal = manifest.cases[1].historicalConfig.literal),
    (manifest) => (manifest.cases[0].historicalConfig.file = "UserCard.vue"),
    (manifest) => (manifest.cases[0].historicalConfig.literal = '"source"'),
    (manifest) => {
      const fixture = manifest.cases[0];
      fixture.witness.function = "contradictory_sort_settings_cannot_write_authored_sources";
      fixture.witness.functionSha256 = sha256(
        retainedFormatterFunction(
          fs.readFileSync(path.join(root, fixture.witness.path)),
          fixture.witness.function,
        ),
      );
    },
  ]) {
    const manifest = JSON.parse(fs.readFileSync(path.join(root, manifestPath)));
    mutate(manifest);
    const file = path.join(directory, "manifest.json");
    fs.writeFileSync(file, JSON.stringify(manifest));
    assert.throws(() => loadSortingConfigManifest(root, file));
  }
  const execution = "tests/tooling/differential-formatter-api-execution.test.mjs";
  assert(!planToolingTests([manifestPath], { cwd: root }).tests.includes(execution));
  assert(planToolingTests([manifestPath], { cwd: root, tier: "merge" }).tests.includes(execution));
});
