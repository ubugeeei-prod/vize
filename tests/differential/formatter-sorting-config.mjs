import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { expectedBuildIdentity, validateBuildReceipt } from "./build-receipt.mjs";
import { loadFormatterApiManifest } from "./formatter-api.mjs";
import { sha256 } from "./manifest.mjs";

export const SORTING_CONFIG_MANIFEST =
  "tests/_fixtures/differential/formatter-history/import-sorting-config-manifest.json";

function asset(root, declared) {
  assert.equal(typeof declared.path, "string");
  assert(!path.isAbsolute(declared.path) && !declared.path.split("/").includes(".."));
  const resolved = fs.realpathSync(path.join(root, declared.path));
  const relative = path.relative(fs.realpathSync(root), resolved);
  assert(relative && !relative.startsWith("..") && !path.isAbsolute(relative));
  const bytes = fs.readFileSync(resolved);
  assert.equal(sha256(bytes), declared.sha256, "sorting config asset changed");
  return bytes;
}

function files(root, declared) {
  assert(declared && typeof declared === "object" && !Array.isArray(declared));
  return Object.fromEntries(
    Object.entries(declared).map(([file, ref]) => {
      assert(
        file &&
          !path.isAbsolute(file) &&
          !file.split("/").some((part) => ["", ".", ".."].includes(part)),
      );
      return [file, asset(root, ref)];
    }),
  );
}

export function loadSortingConfigManifest(root, manifestPath = SORTING_CONFIG_MANIFEST) {
  const raw = fs.readFileSync(path.resolve(root, manifestPath));
  const manifest = JSON.parse(raw.toString());
  assert.equal(manifest.schema, "vize.formatter-sorting-config-history");
  assert.equal(manifest.version, 1);
  assert.equal(manifest.issue, 6882);
  assert.equal(manifest.featureIssue, 7258);
  assert.equal(manifest.nativeHandled, 0);
  asset(root, manifest.apiManifest);
  const api = loadFormatterApiManifest(path.join(root, manifest.apiManifest.path), root);
  const ids = new Set();
  const cases = manifest.cases.map((fixture) => {
    assert.match(fixture.id, /^[a-z0-9/-]+$/);
    assert(!ids.has(fixture.id), "duplicate configuration scenario");
    ids.add(fixture.id);
    const reference = api.cases.find(({ id }) => id === fixture.apiCase);
    assert(reference, "unregistered API reference");
    const owner = asset(root, {
      path: fixture.witness.path,
      sha256: fixture.witness.sourceSha256,
    }).toString();
    assert.match(fixture.witness.function, /^[a-z][a-z0-9_]+$/);
    assert(owner.includes(`fn ${fixture.witness.function}(`), "missing original config law");
    const initialFiles = files(root, fixture.initialFiles);
    assert.equal(typeof fixture.scriptBody, "boolean");
    const source = fixture.scriptBody
      ? Buffer.from(
          reference.input.toString().split("\n").slice(1).join("\n").split("</script>")[0],
        )
      : reference.input;
    assert(initialFiles[fixture.target].equals(source), "unbound authored source input");
    assert(fixture.steps.length > 0);
    const steps = fixture.steps.map((step) => {
      assert(Array.isArray(step.argv) && step.argv.every((arg) => typeof arg === "string"));
      assert.equal(step.argv[0], "fmt");
      for (const arg of step.argv.slice(1)) {
        assert(
          ["--check", "--write", "--no-config", "--config", "*.vue"].includes(arg) ||
            Object.hasOwn(initialFiles, arg),
          "configuration command leaves the declared workspace",
        );
      }
      assert([0, 1, 2].includes(step.exitStatus));
      assert.equal(typeof step.formatted, "boolean");
      const stdout = asset(root, step.stdout);
      const stderr = asset(root, step.stderr);
      const expectedFiles = files(root, step.files);
      assert(Object.hasOwn(expectedFiles, fixture.target));
      const target = expectedFiles[fixture.target];
      const expected = fixture.scriptBody
        ? Buffer.from(
            reference.expected.toString().split("\n").slice(1).join("\n").split("</script>")[0],
          )
        : reference.expected;
      assert(
        target.equals(step.formatted ? expected : initialFiles[fixture.target]),
        "unbound full file reference",
      );
      return { ...step, stdout, stderr, expectedFiles };
    });
    return { ...fixture, initialFiles, steps };
  });
  return { manifest, manifestSha256: sha256(raw), cases };
}

function observation(bytes) {
  return { base64: bytes.toString("base64"), sha256: sha256(bytes) };
}
function snapshot(workspace, directory = workspace) {
  const result = {};
  for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
    const file = path.join(directory, entry.name);
    assert(!entry.isSymbolicLink(), "configuration workspace contains a symlink");
    if (entry.isDirectory()) Object.assign(result, snapshot(workspace, file));
    else {
      assert(entry.isFile(), "configuration workspace contains a non-file");
      result[path.relative(workspace, file).split(path.sep).join("/")] = observation(
        fs.readFileSync(file),
      );
    }
  }
  return result;
}
function expectedSnapshot(files) {
  return Object.fromEntries(
    Object.entries(files).map(([file, bytes]) => [file, observation(bytes)]),
  );
}
function expectedStderr(step, workspace) {
  // Only the declared temporary workspace identity is interpolated. Actual
  // output remains complete raw bytes; no stream is stripped or normalized.
  return Buffer.from(step.stderr.toString().replaceAll("{{workspace}}", workspace));
}
function summary(rows) {
  return {
    plannedCases: rows.length,
    legacyMatches: rows.filter(({ state }) => state === "matched-reference").length,
    legacyFailures: rows.filter(({ state }) => state === "failed").length,
    nativeUnsupported: rows.length,
    nativeHandled: 0,
    nativeEquivalent: 0,
    pairedComparisons: 0,
  };
}

export function validateSortingConfigReport(loaded, report, build) {
  assert.equal(report.schema, "vize.formatter-sorting-config-result");
  assert.equal(report.version, 1);
  assert.equal(report.manifestSha256, loaded.manifestSha256);
  assert.deepEqual(report.buildReceipt, build);
  assert.equal(report.rows.length, loaded.cases.length, "missing configuration scenario");
  const seen = new Set();
  for (const row of report.rows) {
    assert(!seen.has(row.id), "duplicate configuration result");
    seen.add(row.id);
    const fixture = loaded.cases.find(({ id }) => id === row.id);
    assert(fixture, "unplanned configuration result");
    assert(path.isAbsolute(row.workspace));
    if (row.initialFiles === null) {
      assert.equal(row.state, "failed");
      assert.equal(row.passes.length, 0);
    } else assert.deepEqual(row.initialFiles, expectedSnapshot(fixture.initialFiles));
    assert.equal(row.native, "unsupported");
    assert(["matched-reference", "failed"].includes(row.state));
    if (row.cleanupError) {
      assert.equal(row.state, "failed");
      assert.equal(row.cleanupError.kind, "CleanupError");
      assert.equal(typeof row.cleanupError.message, "string");
      assert(row.cleanupError.message.length > 0);
    }
    assert(row.passes.length <= fixture.steps.length);
    let previous = row.initialFiles;
    for (const [index, pass] of row.passes.entries()) {
      const step = fixture.steps[index];
      assert.deepEqual(pass.argv, step.argv);
      assert.deepEqual(pass.inputFiles, previous, "broken actual file-state chain");
      if (pass.outputFiles === null) {
        assert.equal(row.state, "failed", "incomplete file capture cannot match");
        assert.equal(index, row.passes.length - 1, "broken output ends the process chain");
        assert.equal(pass.outputSnapshotError?.kind, "FileSnapshotError");
        assert.equal(typeof pass.outputSnapshotError.message, "string");
        assert(pass.outputSnapshotError.message.length > 0);
      } else assert.equal(pass.outputSnapshotError, null);
      for (const actualFiles of [
        pass.inputFiles,
        ...(pass.outputFiles === null ? [] : [pass.outputFiles]),
      ]) {
        for (const bytes of Object.values(actualFiles)) {
          assert.equal(sha256(Buffer.from(bytes.base64, "base64")), bytes.sha256);
        }
      }
      for (const stream of [pass.stdout, pass.stderr]) {
        assert.equal(sha256(Buffer.from(stream.base64, "base64")), stream.sha256);
      }
      if (row.state === "matched-reference") {
        assert.equal(pass.exitStatus, step.exitStatus);
        assert.equal(pass.signal, null);
        assert.equal(pass.processError, null);
        assert.deepEqual(pass.stdout, observation(step.stdout));
        assert.deepEqual(pass.stderr, observation(expectedStderr(step, row.workspace)));
        assert.deepEqual(pass.outputFiles, expectedSnapshot(step.expectedFiles));
      }
      previous = pass.outputFiles;
    }
    if (row.state === "matched-reference") assert.equal(row.passes.length, fixture.steps.length);
    else assert(typeof row.error === "string" && row.error.length > 0);
  }
  assert.deepEqual(report.summary, summary(report.rows));
  return report.summary;
}

// Store process custody before any file observation that may fail. A broken
// workspace must not erase the complete call that caused the failure.
export function recordSortingConfigProcess(row, step, inputFiles, result, readOutput) {
  const pass = {
    argv: step.argv,
    inputFiles,
    outputFiles: null,
    outputSnapshotError: null,
    stdout: observation(result.stdout ?? Buffer.alloc(0)),
    stderr: observation(result.stderr ?? Buffer.alloc(0)),
    exitStatus: result.status,
    signal: result.signal,
    processError: result.error?.message ?? null,
  };
  row.passes.push(pass);
  try {
    pass.outputFiles = readOutput();
  } catch (error) {
    pass.outputSnapshotError = { kind: "FileSnapshotError", message: error.message };
    throw error;
  }
  return pass;
}

export function runSortingConfigPack({ repoRoot, binaryPath }) {
  const build = expectedBuildIdentity(repoRoot);
  assert.equal(fs.realpathSync(binaryPath), fs.realpathSync(path.join(repoRoot, build.binaryPath)));
  const receipt = JSON.parse(fs.readFileSync(`${binaryPath}.differential-build.json`, "utf8"));
  validateBuildReceipt(receipt, build);
  const loaded = loadSortingConfigManifest(repoRoot);
  const rows = loaded.cases.map((fixture) => {
    const workspace = fs.mkdtempSync(path.join(os.tmpdir(), "formatter-sorting-config-"));
    const row = {
      id: fixture.id,
      workspace,
      initialFiles: null,
      state: "matched-reference",
      native: "unsupported",
      passes: [],
    };
    try {
      for (const [file, bytes] of Object.entries(fixture.initialFiles)) {
        fs.mkdirSync(path.dirname(path.join(workspace, file)), { recursive: true });
        fs.writeFileSync(path.join(workspace, file), bytes);
      }
      row.initialFiles = snapshot(workspace);
      for (const step of fixture.steps) {
        const inputFiles = snapshot(workspace);
        const result = spawnSync(binaryPath, step.argv, {
          cwd: workspace,
          timeout: 30_000,
          maxBuffer: 4 * 1024 * 1024,
        });
        const pass = recordSortingConfigProcess(row, step, inputFiles, result, () =>
          snapshot(workspace),
        );
        assert.equal(result.error, undefined, result.error?.message);
        assert.equal(result.signal, null);
        assert.equal(result.status, step.exitStatus);
        assert.deepEqual(pass.stdout, observation(step.stdout));
        assert.deepEqual(pass.stderr, observation(expectedStderr(step, workspace)));
        assert.deepEqual(pass.outputFiles, expectedSnapshot(step.expectedFiles));
      }
    } catch (error) {
      row.state = "failed";
      row.error = error.message;
    } finally {
      try {
        fs.rmSync(workspace, { recursive: true, force: true });
      } catch (error) {
        row.state = "failed";
        row.error ??= error.message;
        row.cleanupError = { kind: "CleanupError", message: error.message };
      }
    }
    return row;
  });
  const report = {
    schema: "vize.formatter-sorting-config-result",
    version: 1,
    manifestSha256: loaded.manifestSha256,
    buildReceipt: receipt,
    rows,
    summary: summary(rows),
  };
  validateSortingConfigReport(loaded, report, receipt);
  return report;
}
