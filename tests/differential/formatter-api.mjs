import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import {
  currentFormatterReference,
  currentFormatterReferenceSummary,
  formatterReferenceComparisons,
  validateFormatterReferencePass,
} from "./formatter-current-reference.ts";
import { normalizeFormatterHistoryManifest } from "./formatter-api-provenance.ts";
import {
  loadFormatterCaptureReceipt,
  validateFormatterCapturedCase,
} from "./formatter-api-capture.ts";
import { sha256 } from "./manifest.mjs";
import { resolvePreservedFormatterSource } from "./formatter-history-source-artifact.ts";
import { validateObserverReceipt } from "./formatter-api-build.mjs";
import {
  assertFormatterError,
  configuredFormatterOptions,
  formatterApiKind,
} from "./formatter-api-contract.ts";

const APIS = {
  format_script: "--script",
  format_script_with_sort_imports: "--sorted-script",
  "GlyphFormatter::format": "--sorted-sfc",
  format_sfc: "--sfc",
  format_style: "--style",
  format_template: "--template",
  format_json: "--json",
  format_jsonc: "--jsonc",
};

function immutableArtifact(root, artifact) {
  assert.equal(typeof artifact.path, "string");
  assert(!path.isAbsolute(artifact.path));
  const resolved = fs.realpathSync(path.resolve(root, artifact.path));
  const relative = path.relative(fs.realpathSync(root), resolved);
  assert(relative && !relative.startsWith("..") && !path.isAbsolute(relative));
  assert.match(artifact.sha256, /^[a-f0-9]{64}$/);
  const preserved = resolvePreservedFormatterSource(root, artifact);
  if (preserved) return preserved;
  const bytes = fs.readFileSync(resolved);
  assert.equal(sha256(bytes), artifact.sha256, `fixture bytes changed: ${artifact.path}`);
  return bytes;
}

export function loadFormatterApiManifest(manifestPath, repoRoot) {
  const raw = fs.readFileSync(manifestPath);
  const manifest = normalizeFormatterHistoryManifest(JSON.parse(raw.toString()));
  assert.equal(manifest.schema, "vize.formatter-history-fixtures");
  assert([1, 2].includes(manifest.version));
  assert.equal(manifest.issue, 6882);
  assert.match(manifest.source.revision, /^[a-f0-9]{40}$/);
  assert.equal(manifest.nativeHandled, 0);
  assert(Array.isArray(manifest.cases) && manifest.cases.length > 0);
  const capture = loadFormatterCaptureReceipt(repoRoot, manifest);
  const ids = new Set();
  const cases = manifest.cases.map((fixture) => {
    assert.match(fixture.id, /^[a-z0-9/-]+$/);
    assert(!ids.has(fixture.id), `duplicate case: ${fixture.id}`);
    ids.add(fixture.id);
    assert(Object.hasOwn(APIS, fixture.api), "unregistered public formatter API");
    assert(["default", "skip_script_stabilization"].includes(fixture.profile));
    const internal = fixture.profile === "skip_script_stabilization";
    assert(!internal || ["format_script", "format_sfc"].includes(fixture.api));
    const options = configuredFormatterOptions(fixture);
    if (manifest.featureIssue === 7258) assert(fixture.importSorting, "missing feature options");
    if (fixture.importSorting) {
      assert.equal(manifest.featureIssue, 7258);
      assert.deepEqual(
        JSON.parse(immutableArtifact(repoRoot, fixture.sourceOptionsProbe).toString()),
        options.effective,
        "complete recorded import sorting probe changed",
      );
      const owner = immutableArtifact(repoRoot, {
        path: fixture.witness.path,
        sha256: fixture.witness.sourceSha256,
      }).toString();
      assert.match(fixture.witness.function, /^[a-z][a-z0-9_]+$/);
      assert(owner.includes(`fn ${fixture.witness.function}(`), "missing original option law");
    }
    const error = fixture.outcome === "error";
    assert(fixture.outcome === undefined || ["success", "error"].includes(fixture.outcome));
    if (error) assertFormatterError(fixture.api, internal, fixture.typedError);
    assert.equal(fixture.reference, "current-public-api-output");
    assert.equal(fixture.native, "unsupported");
    assert.equal(fixture.kind, formatterApiKind(fixture.api, fixture.kind));
    if (fixture.sourceExpected) immutableArtifact(repoRoot, fixture.sourceExpected);
    if (fixture.options.userOverrides) {
      if (fixture.witness.commit) {
        assert.match(fixture.witness.commit, /^[a-f0-9]{40}$/);
        assert.equal(fixture.witness.sourceArtifact.sha256, fixture.witness.sourceSha256);
        immutableArtifact(repoRoot, fixture.witness.sourceArtifact);
      } else {
        immutableArtifact(repoRoot, {
          path: fixture.witness.path,
          sha256: fixture.witness.sourceSha256,
        });
      }
    }
    const input = immutableArtifact(repoRoot, fixture.input);
    const expected = immutableArtifact(repoRoot, fixture.expected);
    if (fixture.transport) {
      const {
        prefix,
        suffix,
        bodyInputByteRange: [start, end],
        bodySha256,
      } = fixture.transport;
      assert.equal(fixture.transport.wholeInputIsOriginal, false);
      assert.equal(start, Buffer.byteLength(prefix));
      assert.equal(end + Buffer.byteLength(suffix), input.length);
      assert(input.subarray(0, start).equals(Buffer.from(prefix)));
      assert(input.subarray(end).equals(Buffer.from(suffix)));
      assert.equal(sha256(input.subarray(start, end)), bodySha256);
    }
    validateFormatterCapturedCase(
      repoRoot,
      capture,
      fixture,
      options,
      APIS[fixture.api],
      input,
      expected,
    );
    return {
      ...fixture,
      input,
      expected,
      ...currentFormatterReference(repoRoot, fixture, input, expected),
      argv: [APIS[fixture.api], ...options.flags, ...(error ? ["--expect-error"] : [])],
      optionsArgv: [
        fixture.importSorting ? "--sort-options-json" : "--options-json",
        ...options.flags,
      ],
      effectiveOptions: options.effective,
      passCount: internal || error ? 1 : 3,
      contract: error
        ? "typed-error-bytes"
        : internal
          ? "legacy-internal-observation"
          : "full-output-bytes-and-fixed-point",
    };
  });
  return { manifest, cases, manifestSha256: sha256(raw) };
}

function expectedStderr(fixture, input, output) {
  return fixture.outcome === "error"
    ? Buffer.from(`error=${fixture.typedError}\n`)
    : ["format_sfc", "GlyphFormatter::format"].includes(fixture.api)
      ? Buffer.from(`changed=${!input.equals(output)}\n`)
      : Buffer.alloc(0);
}

function summary(rows) {
  return {
    plannedCases: rows.length,
    legacyByteMatches: rows.filter(
      (row) =>
        row.contract === "full-output-bytes-and-fixed-point" &&
        !row.currentReference &&
        row.legacy.state === "matched-reference",
    ).length,
    legacyInternalObservations: rows.filter(
      (row) =>
        row.contract === "legacy-internal-observation" && row.legacy.state === "matched-reference",
    ).length,
    legacyErrorMatches: rows.filter(
      (row) => row.contract === "typed-error-bytes" && row.legacy.state === "matched-reference",
    ).length,
    legacyFailures: rows.filter((row) => row.legacy.state === "failed").length,
    ...currentFormatterReferenceSummary(rows),
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
    assert.deepEqual(row.options.argv, fixture.optionsArgv);
    const optionBytes = Buffer.from(row.options.stdoutBase64, "base64");
    assert.equal(sha256(optionBytes), row.options.sha256);
    if (row.legacy.state === "matched-reference") {
      assert.equal(row.options.exitStatus, 0);
      assert.equal(row.options.signal, null);
      assert.equal(row.options.processError, null);
      assert.equal(row.options.stderrBase64, "");
      assert.deepEqual(
        JSON.parse(optionBytes.toString()),
        fixture.effectiveOptions,
        "actual effective formatter options changed",
      );
    }
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
      validateFormatterReferencePass(fixture, row, pass, output);
      if (row.legacy.state === "matched-reference") {
        assert.equal(pass.exitStatus, fixture.outcome === "error" ? 1 : 0);
        assert.equal(pass.signal, null);
        assert.equal(pass.processError, null);
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
    const probe = spawnSync(binaryPath, fixture.optionsArgv, { timeout: 30_000 });
    const optionBytes = probe.stdout ?? Buffer.alloc(0);
    const row = {
      id: fixture.id,
      argv: fixture.argv,
      contract: fixture.contract,
      ...(fixture.currentReference ? { currentReference: fixture.currentReference } : {}),
      options: {
        argv: fixture.optionsArgv,
        exitStatus: probe.status,
        signal: probe.signal,
        processError: probe.error?.message ?? null,
        stdoutBase64: optionBytes.toString("base64"),
        sha256: sha256(optionBytes),
        stderrBase64: (probe.stderr ?? Buffer.alloc(0)).toString("base64"),
      },
      legacy: { state: "matched-reference", passes: [] },
      native: { state: "unsupported", reason: "native formatter adapter unavailable" },
      comparison: { state: "not-compared" },
    };
    try {
      assert.equal(probe.error, undefined, probe.error?.message);
      assert.equal(probe.signal, null);
      assert.equal(probe.status, 0);
      assert.equal((probe.stderr ?? Buffer.alloc(0)).length, 0);
      assert.deepEqual(
        JSON.parse(optionBytes.toString()),
        fixture.effectiveOptions,
        "actual effective formatter options changed",
      );
      let input = fixture.input;
      for (let pass = 1; pass <= fixture.passCount; pass += 1) {
        const result = spawnSync(binaryPath, fixture.argv, {
          input,
          timeout: 30_000,
          maxBuffer: 4 * 1024 * 1024,
        });
        const output = result.stdout ?? Buffer.alloc(0);
        const stderr = result.stderr ?? Buffer.alloc(0);
        const comparisons = formatterReferenceComparisons(fixture, output);
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
          ...comparisons,
        });
        assert.equal(result.error, undefined, result.error?.message);
        assert.equal(result.signal, null);
        assert.equal(result.status, fixture.outcome === "error" ? 1 : 0, stderr.toString());
        assert.equal(
          (comparisons.currentReferenceComparison ?? comparisons.referenceComparison).state,
          "equal",
          "complete output baseline drift",
        );
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
