import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { compareBytes } from "./compare.mjs";
import { sha256 } from "./manifest.mjs";
import { validateObserverReceipt } from "./formatter-api-build.mjs";

const APIS = {
  format_script: "--script",
  format_sfc: "--sfc",
  format_style: "--style",
  format_template: "--template",
};

function immutableArtifact(root, artifact) {
  assert.equal(typeof artifact.path, "string");
  assert(!path.isAbsolute(artifact.path));
  const resolved = fs.realpathSync(path.resolve(root, artifact.path));
  const relative = path.relative(fs.realpathSync(root), resolved);
  assert(relative && !relative.startsWith("..") && !path.isAbsolute(relative));
  assert.match(artifact.sha256, /^[a-f0-9]{64}$/);
  const bytes = fs.readFileSync(resolved);
  assert.equal(sha256(bytes), artifact.sha256, `fixture bytes changed: ${artifact.path}`);
  return bytes;
}

export function loadFormatterApiManifest(manifestPath, repoRoot) {
  const raw = fs.readFileSync(manifestPath);
  const manifest = JSON.parse(raw.toString());
  assert.equal(manifest.schema, "vize.formatter-history-fixtures");
  assert.equal(manifest.version, 1);
  assert.equal(manifest.issue, 6882);
  assert.match(manifest.source.revision, /^[a-f0-9]{40}$/);
  assert.equal(manifest.nativeHandled, 0);
  assert(Array.isArray(manifest.cases) && manifest.cases.length > 0);
  const ids = new Set();
  const cases = manifest.cases.map((fixture) => {
    assert.match(fixture.id, /^[a-z0-9/-]+$/);
    assert(!ids.has(fixture.id), `duplicate case: ${fixture.id}`);
    ids.add(fixture.id);
    assert(Object.hasOwn(APIS, fixture.api), "unregistered public formatter API");
    assert(["default", "skip_script_stabilization"].includes(fixture.profile));
    const internal = fixture.profile === "skip_script_stabilization";
    assert(!internal || ["format_script", "format_sfc"].includes(fixture.api));
    assert.deepEqual(fixture.options, {
      base: "FormatOptions::default()",
      internalOverrides: internal ? { skipScriptStabilization: true } : {},
    });
    assert.equal(fixture.reference, "current-public-api-output");
    assert.equal(fixture.native, "unsupported");
    assert.equal(
      fixture.kind,
      fixture.api === "format_sfc"
        ? "Vue"
        : fixture.api === "format_script"
          ? "TypeScript"
          : fixture.api === "format_style"
            ? "CSS"
            : "VueTemplate",
    );
    return {
      ...fixture,
      input: immutableArtifact(repoRoot, fixture.input),
      expected: immutableArtifact(repoRoot, fixture.expected),
      argv: [APIS[fixture.api], ...(internal ? ["--legacy-single-pass"] : [])],
      passCount: internal ? 1 : 3,
      contract: internal ? "legacy-internal-observation" : "full-output-bytes-and-fixed-point",
    };
  });
  return { manifest, cases, manifestSha256: sha256(raw) };
}

function expectedStderr(fixture, input, output) {
  return fixture.api === "format_sfc"
    ? Buffer.from(`changed=${!input.equals(output)}\n`)
    : Buffer.alloc(0);
}

function summary(rows) {
  return {
    plannedCases: rows.length,
    legacyByteMatches: rows.filter(
      (row) =>
        row.contract !== "legacy-internal-observation" && row.legacy.state === "matched-reference",
    ).length,
    legacyInternalObservations: rows.filter(
      (row) =>
        row.contract === "legacy-internal-observation" && row.legacy.state === "matched-reference",
    ).length,
    legacyFailures: rows.filter((row) => row.legacy.state === "failed").length,
    nativeUnsupported: rows.length,
    nativeHandled: 0,
    nativeEquivalent: 0,
    pairedComparisons: 0,
  };
}

export function validateFormatterApiReport(loaded, report, receipt) {
  assert.equal(report.schema, "vize.differential.formatter-api-result");
  assert.equal(report.version, 1);
  assert.equal(report.manifestSha256, loaded.manifestSha256);
  assert.deepEqual(report.buildReceipt, receipt);
  assert.equal(report.rows.length, loaded.cases.length, "missing or extra result rows");
  const ids = new Set();
  for (const row of report.rows) {
    assert(!ids.has(row.id), `duplicate result: ${row.id}`);
    ids.add(row.id);
    const fixture = loaded.cases.find((item) => item.id === row.id);
    assert(fixture, "unplanned result");
    assert.equal(row.contract, fixture.contract);
    assert.deepEqual(row.argv, fixture.argv);
    assert.equal(row.native.state, "unsupported");
    assert.equal(row.native.reason, "native formatter adapter unavailable");
    assert.equal(row.comparison.state, "not-compared");
    assert(["matched-reference", "failed"].includes(row.legacy.state));
    assert(Array.isArray(row.legacy.passes));
    assert(row.legacy.passes.length <= fixture.passCount);
    let previous = fixture.input;
    for (const [index, pass] of row.legacy.passes.entries()) {
      const input = Buffer.from(pass.inputBase64, "base64");
      const output = Buffer.from(pass.stdoutBase64, "base64");
      const stderr = Buffer.from(pass.stderrBase64, "base64");
      assert.equal(pass.pass, index + 1);
      assert(input.equals(previous));
      assert.equal(pass.inputSha256, sha256(input));
      assert.equal(pass.outputSha256, sha256(output));
      assert.deepEqual(pass.referenceComparison, compareBytes(fixture.expected, output));
      if (row.legacy.state === "matched-reference") {
        assert.equal(pass.exitStatus, 0);
        assert.equal(pass.signal, null);
        assert.equal(pass.processError, null);
        assert.equal(pass.referenceComparison.state, "equal");
        assert(
          stderr.equals(expectedStderr(fixture, input, output)),
          "unexpected API observation stream",
        );
      }
      previous = output;
    }
    if (row.legacy.state === "matched-reference")
      assert.equal(row.legacy.passes.length, fixture.passCount);
    else assert(typeof row.legacy.error === "string" && row.legacy.error.length > 0);
  }
  assert.deepEqual(report.summary, summary(report.rows), "inconsistent actual result accounting");
  return report.summary;
}

export function runFormatterApiPack({ manifestPath, repoRoot, binaryPath, receipt }) {
  assert(path.isAbsolute(binaryPath), "absolute source-built observer is required");
  validateObserverReceipt(receipt, repoRoot, binaryPath);
  const loaded = loadFormatterApiManifest(manifestPath, repoRoot);
  const report = {
    schema: "vize.differential.formatter-api-result",
    version: 1,
    manifestSha256: loaded.manifestSha256,
    buildReceipt: receipt,
    rows: [],
  };
  for (const fixture of loaded.cases) {
    const row = {
      id: fixture.id,
      argv: fixture.argv,
      contract: fixture.contract,
      legacy: { state: "matched-reference", passes: [] },
      native: { state: "unsupported", reason: "native formatter adapter unavailable" },
      comparison: { state: "not-compared" },
    };
    try {
      let input = fixture.input;
      for (let pass = 1; pass <= fixture.passCount; pass += 1) {
        const result = spawnSync(binaryPath, fixture.argv, {
          input,
          timeout: 30_000,
          maxBuffer: 4 * 1024 * 1024,
        });
        const output = result.stdout ?? Buffer.alloc(0);
        const stderr = result.stderr ?? Buffer.alloc(0);
        const comparison = compareBytes(fixture.expected, output);
        row.legacy.passes.push({
          pass,
          inputBase64: input.toString("base64"),
          inputSha256: sha256(input),
          stdoutBase64: output.toString("base64"),
          outputSha256: sha256(output),
          stderrBase64: stderr.toString("base64"),
          exitStatus: result.status,
          signal: result.signal,
          processError: result.error?.message ?? null,
          referenceComparison: comparison,
        });
        assert.equal(result.error, undefined, result.error?.message);
        assert.equal(result.signal, null);
        assert.equal(result.status, 0, stderr.toString());
        assert.equal(comparison.state, "equal", "complete output baseline drift");
        assert(
          stderr.equals(expectedStderr(fixture, input, output)),
          "unexpected API observation stream",
        );
        input = output;
      }
    } catch (error) {
      row.legacy.state = "failed";
      row.legacy.error = error.message;
    }
    report.rows.push(row);
  }
  report.summary = summary(report.rows);
  validateFormatterApiReport(loaded, report, receipt);
  return report;
}
