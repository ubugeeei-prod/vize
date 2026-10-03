import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  loadSortingConfigManifest,
  recordSortingConfigProcess,
  SORTING_CONFIG_MANIFEST,
  validateSortingConfigReport,
} from "../differential/formatter-sorting-config.mjs";
import { sha256 } from "../differential/manifest.mjs";
import { planToolingTests } from "../../tools/support/compat/github/plan-tooling-tests.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const observe = (bytes) => ({ base64: bytes.toString("base64"), sha256: sha256(bytes) });
const snapshot = (files) =>
  Object.fromEntries(Object.entries(files).map(([file, bytes]) => [file, observe(bytes)]));

// These synthetic rows exercise admission only. Real CLI observations come
// from the source-built mandatory T1 execution, with a real build receipt.
function synthetic(loaded) {
  const rows = loaded.cases.map((fixture) => {
    const workspace = path.join(os.tmpdir(), "formatter-config-contract");
    const initialFiles = snapshot(fixture.initialFiles);
    let previous = initialFiles;
    const passes = fixture.steps.map((step) => {
      const outputFiles = snapshot(step.expectedFiles);
      const pass = {
        argv: step.argv,
        inputFiles: previous,
        outputFiles,
        outputSnapshotError: null,
        stdout: observe(step.stdout),
        stderr: observe(Buffer.from(step.stderr.toString().replaceAll("{{workspace}}", workspace))),
        exitStatus: step.exitStatus,
        signal: null,
        processError: null,
      };
      previous = outputFiles;
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
      plannedCases: 6,
      legacyMatches: 6,
      legacyFailures: 0,
      nativeUnsupported: 6,
      nativeHandled: 0,
      nativeEquivalent: 0,
      pairedComparisons: 0,
    },
  };
}

void test("configuration history binds complete author input, process outputs and all file effects", () => {
  const loaded = loadSortingConfigManifest(root);
  assert.deepEqual(
    loaded.cases.map(({ id }) => id),
    [
      "configured-json",
      "disabled-json",
      "no-config",
      "invalid-boundary",
      "standalone-typescript",
      "effectful-mjs",
    ].map((id) => "import-sorting-config/" + id),
  );
  assert.deepEqual(
    loaded.cases.map(({ steps }) => steps.length),
    [4, 2, 2, 1, 2, 1],
  );
  assert.deepEqual(
    validateSortingConfigReport(loaded, synthetic(loaded), {}),
    synthetic(loaded).summary,
  );
  const effect = loaded.cases[5];
  assert.equal(effect.initialFiles["eval-count.txt"].toString(), "0");
  assert.equal(effect.steps[0].expectedFiles["eval-count.txt"].toString(), "1");
  assert(
    effect.steps[0].expectedFiles["nested/Ignored.vue"].equals(
      effect.initialFiles["nested/Ignored.vue"],
    ),
  );
  assert.deepEqual(JSON.parse(effect.steps[0].expectedFiles["returned-config.json"].toString()), {
    formatter: { sortImports: {} },
    entries: [{ basePath: "nested", files: ["*.vue"], ignores: ["Ignored.vue"] }],
  });
});

void test("configuration result admission rejects lost raw bytes, skipped steps and forged effects", () => {
  const loaded = loadSortingConfigManifest(root);
  for (const mutate of [
    (report) => {
      report.rows.pop();
    },
    (report) => {
      report.rows[1].id = report.rows[0].id;
    },
    (report) => {
      report.rows[0].passes.pop();
    },
    (report) => {
      report.rows[0].passes[0].argv.push("--no-config");
    },
    (report) => {
      report.rows[0].passes[1].inputFiles = {};
    },
    (report) => {
      report.rows[0].passes[0].exitStatus = 0;
    },
    (report) => {
      report.rows[0].passes[0].signal = "SIGTERM";
    },
    (report) => {
      report.rows[0].passes[0].stdout = observe(Buffer.from("partial"));
    },
    (report) => {
      report.rows[0].passes[0].stderr = observe(Buffer.alloc(0));
    },
    (report) => {
      report.rows[0].passes[2].outputFiles["UserCard.vue"] = observe(Buffer.from("partial"));
    },
    (report) => {
      report.rows[5].passes[0].outputFiles["eval-count.txt"] = observe(Buffer.from("2"));
    },
    (report) => {
      delete report.rows[5].passes[0].outputFiles["returned-config.json"];
    },
    (report) => {
      report.rows[5].passes[0].outputFiles["nested/Ignored.vue"] = observe(Buffer.from("changed"));
    },
    (report) => {
      report.summary.nativeHandled = 1;
    },
  ]) {
    const report = structuredClone(synthetic(loaded));
    mutate(report);
    assert.throws(() => validateSortingConfigReport(loaded, report, {}));
  }
});

void test("configuration manifests reject source drift and unbound complete file references", (t) => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "formatter-config-manifest-"));
  t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
  for (const mutate of [
    (manifest) => {
      manifest.apiManifest.sha256 = "0".repeat(64);
    },
    (manifest) => {
      manifest.cases[0].witness.sourceSha256 = "0".repeat(64);
    },
    (manifest) => {
      manifest.cases[0].witness.function = "invented_config_law";
    },
    (manifest) => {
      manifest.cases[0].steps[0].files["UserCard.vue"] =
        manifest.cases[0].steps[2].files["UserCard.vue"];
    },
    (manifest) => {
      manifest.cases[0].initialFiles["UserCard.vue"] =
        manifest.cases[0].steps[2].files["UserCard.vue"];
    },
    (manifest) => {
      manifest.cases[0].steps[0].stdout.sha256 = "0".repeat(64);
    },
    (manifest) => {
      manifest.cases.push(manifest.cases[0]);
    },
    (manifest) => {
      manifest.nativeHandled = 1;
    },
  ]) {
    const manifest = JSON.parse(fs.readFileSync(path.join(root, SORTING_CONFIG_MANIFEST), "utf8"));
    mutate(manifest);
    const file = path.join(directory, "manifest.json");
    fs.writeFileSync(file, JSON.stringify(manifest));
    assert.throws(() => loadSortingConfigManifest(root, file));
  }
  const execution = "tests/tooling/differential-formatter-api-execution.test.mjs";
  assert(!planToolingTests([SORTING_CONFIG_MANIFEST], { cwd: root }).tests.includes(execution));
  assert(
    planToolingTests([SORTING_CONFIG_MANIFEST], { cwd: root, tier: "merge" }).tests.includes(
      execution,
    ),
  );
});

void test("a failed output snapshot retains the complete already-executed process frame", () => {
  const loaded = loadSortingConfigManifest(root);
  const report = synthetic(loaded);
  const row = report.rows[0];
  row.passes = [];
  row.state = "failed";
  row.error = "unreadable generated output";
  const result = {
    stdout: Buffer.from("complete stdout\0tail"),
    stderr: Buffer.from("complete stderr\n"),
    status: 2,
    signal: null,
  };
  assert.throws(() =>
    recordSortingConfigProcess(row, loaded.cases[0].steps[0], row.initialFiles, result, () => {
      throw new Error("unreadable generated output");
    }),
  );
  assert.equal(row.passes.length, 1);
  assert.deepEqual(row.passes[0], {
    argv: loaded.cases[0].steps[0].argv,
    inputFiles: row.initialFiles,
    outputFiles: null,
    outputSnapshotError: { kind: "FileSnapshotError", message: "unreadable generated output" },
    stdout: observe(result.stdout),
    stderr: observe(result.stderr),
    exitStatus: 2,
    signal: null,
    processError: null,
  });
  report.summary.legacyMatches = 5;
  report.summary.legacyFailures = 1;
  assert.deepEqual(validateSortingConfigReport(loaded, report, {}), report.summary);
  for (const mutate of [
    (changed) => {
      changed.rows[0].state = "matched-reference";
    },
    (changed) => {
      delete changed.rows[0].passes[0].outputSnapshotError;
    },
    (changed) => {
      changed.rows[0].passes.push(changed.rows[0].passes[0]);
    },
    (changed) => {
      changed.rows[0].passes[0].stdout.sha256 = "0".repeat(64);
    },
  ]) {
    const changed = structuredClone(report);
    mutate(changed);
    assert.throws(() => validateSortingConfigReport(loaded, changed, {}));
  }
});
