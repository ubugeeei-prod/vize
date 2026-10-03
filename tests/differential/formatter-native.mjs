import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import {
  hash,
  bytes,
  nativeHistoryReceipt,
  validateNativeHistoryBuild,
  nativeHistoryBuildEnvironment,
} from "../../npm/native/scripts/formatter-history-build.mjs";
import { nativePreparationIsActive } from "../../npm/native/scripts/test-preparation.mjs";
import { loadPublicNativeManifest } from "./formatter-native-manifest.mjs";

function totals(rows) {
  return {
    plannedCases: rows.length,
    legacyByteMatches: rows.filter(
      (row) => row.state === "matched-reference" && row.outcome === "success",
    ).length,
    legacyErrorMatches: rows.filter(
      (row) => row.state === "matched-reference" && row.outcome === "error",
    ).length,
    legacyFailures: rows.filter((row) => row.state === "failed").length,
    nativeUnsupported: rows.length,
    nativeHandled: 0,
    nativeEquivalent: 0,
    pairedComparisons: 0,
  };
}
export function validatePublicNativeReport(loaded, report, buildReceiptBytes, observerBytes) {
  assert.equal(report.schema, "vize.public-native-formatter-result");
  assert.equal(report.version, 1);
  assert.equal(report.manifestSha256, loaded.manifestSha256);
  assert.deepEqual(report.buildReceipt, JSON.parse(buildReceiptBytes));
  assert.equal(report.rows.length, loaded.cases.length);
  const seen = new Set();
  for (const row of report.rows) {
    assert(!seen.has(row.id));
    seen.add(row.id);
    const fixture = loaded.cases.find(({ id }) => row.id === id);
    assert(fixture);
    assert.equal(row.outcome, fixture.outcome);
    assert.equal(row.native, "unsupported");
    assert(["matched-reference", "failed"].includes(row.state));
    const planned = fixture.outcome === "success" ? 3 : 1;
    assert(row.passes.length <= planned);
    let source = fixture.input.toString();
    for (const [index, pass] of row.passes.entries()) {
      assert.deepEqual(pass.argv, [report.runtime.nodeExecutable, report.runtime.observer]);
      assert.deepEqual(
        pass.stdin,
        bytes(Buffer.from(JSON.stringify({ source, options: fixture.options }))),
      );
      for (const stream of [pass.stdin, pass.stdout, pass.stderr])
        assert.equal(hash(Buffer.from(stream.base64, "base64")), stream.sha256);
      if (row.state === "matched-reference" || index < row.passes.length - 1) {
        assert.equal(pass.exitStatus, 0);
        assert.equal(pass.signal, null);
        assert.equal(pass.processError, null);
        assert.deepEqual(pass.stderr, bytes(Buffer.alloc(0)));
        const packet = JSON.parse(Buffer.from(pass.stdout.base64, "base64"));
        assert.deepEqual(pass.stdout, bytes(Buffer.from(`${JSON.stringify(packet)}\n`)));
        assert.deepEqual(pass.packet, packet);
        assert.deepEqual(Object.keys(packet), [
          "schema",
          "version",
          "input",
          "inputSha256",
          "preparation",
          "buildReceiptSha256",
          "artifactSha256",
          "observerSha256",
          "runtime",
          "result",
          "error",
        ]);
        assert.equal(packet.schema, "vize.public-native-formatter-observation");
        assert.equal(packet.version, 1);
        assert.deepEqual(packet.input, { source, options: fixture.options });
        assert.equal(packet.inputSha256, pass.stdin.sha256);
        assert.deepEqual(packet.preparation, report.preparation);
        for (const key of ["head", "tree", "workingDiff"])
          assert.equal(packet.preparation[key], report.buildReceipt.source[key]);
        assert.equal(packet.preparation.sha256, report.buildReceipt.frozen.sha256);
        assert.equal(packet.buildReceiptSha256, hash(buildReceiptBytes));
        assert.equal(packet.artifactSha256, report.buildReceipt.frozen.sha256);
        assert.equal(packet.observerSha256, hash(observerBytes));
        assert.deepEqual(packet.runtime, {
          node: report.buildReceipt.toolchain.node,
          nodeOptions: report.runtime.nodeOptions,
          cargoEnvironment: report.runtime.cargoEnvironment,
          errorStackTraceLimit: 0,
        });
        if (fixture.outcome === "error") {
          assert.equal(packet.result, null);
          assert.deepEqual(packet.error, fixture.expectedError);
        } else {
          assert.equal(packet.error, null);
          assert.deepEqual(packet.result, {
            code: fixture.expected.toString(),
            changed: source !== fixture.expected.toString(),
          });
          source = packet.result.code;
          if (index > 0) assert.equal(packet.result.changed, false);
        }
      } else {
        assert.equal(index, row.passes.length - 1, "failed process ends the chain");
        if (pass.packet !== null) {
          assert.deepEqual(pass.packet.input, { source, options: fixture.options });
          assert.deepEqual(pass.stdout, bytes(Buffer.from(`${JSON.stringify(pass.packet)}\n`)));
        }
      }
    }
    if (row.state === "matched-reference") assert.equal(row.passes.length, planned);
    else assert(typeof row.error === "string" && row.error.length);
  }
  assert.deepEqual(report.summary, totals(report.rows));
  return report.summary;
}
export function runPublicNativeFormatter(root) {
  const nativeDir = path.join(root, "npm/native");
  assert(nativePreparationIsActive(nativeDir));
  const buildBytes = fs.readFileSync(nativeHistoryReceipt(nativeDir));
  const receipt = validateNativeHistoryBuild(nativeDir, JSON.parse(buildBytes));
  const preparation = JSON.parse(
    fs.readFileSync(path.join(nativeDir, ".artifacts/native/js-test-preparation.json")),
  );
  assert.equal(preparation.sha256, receipt.frozen.sha256);
  const loaded = loadPublicNativeManifest(root);
  const observer = path.join(nativeDir, "scripts/formatter-history-observe.mjs");
  const runtime = {
    nodeExecutable: process.execPath,
    observer,
    nodeOptions: process.env.NODE_OPTIONS ?? null,
    cargoEnvironment: nativeHistoryBuildEnvironment(),
  };
  const rows = loaded.cases.map((fixture) => {
    const row = {
      id: fixture.id,
      outcome: fixture.outcome,
      state: "matched-reference",
      native: "unsupported",
      passes: [],
    };
    let source = fixture.input.toString();
    try {
      for (let pass = 0; pass < (fixture.outcome === "success" ? 3 : 1); pass++) {
        const input = Buffer.from(JSON.stringify({ source, options: fixture.options }));
        const argv = [process.execPath, observer];
        const result = spawnSync(argv[0], argv.slice(1), {
          cwd: root,
          input,
          timeout: 30_000,
          maxBuffer: 8 * 1024 * 1024,
        });
        const frame = {
          argv,
          stdin: bytes(input),
          stdout: bytes(result.stdout ?? Buffer.alloc(0)),
          stderr: bytes(result.stderr ?? Buffer.alloc(0)),
          exitStatus: result.status,
          signal: result.signal,
          processError: result.error?.message ?? null,
          packet: null,
        };
        row.passes.push(frame);
        assert.equal(frame.exitStatus, 0);
        assert.equal(frame.signal, null);
        assert.equal(frame.processError, null);
        assert.deepEqual(frame.stderr, bytes(Buffer.alloc(0)));
        frame.packet = JSON.parse(result.stdout);
        if (fixture.outcome === "error") {
          assert.equal(frame.packet.result, null);
          assert.deepEqual(frame.packet.error, fixture.expectedError);
        } else {
          assert.equal(frame.packet.error, null);
          assert.deepEqual(frame.packet.result, {
            code: fixture.expected.toString(),
            changed: source !== fixture.expected.toString(),
          });
          source = frame.packet.result.code;
        }
      }
    } catch (error) {
      row.state = "failed";
      row.error = error.message;
    }
    return row;
  });
  const report = {
    schema: "vize.public-native-formatter-result",
    version: 1,
    manifestSha256: loaded.manifestSha256,
    buildReceipt: receipt,
    preparation,
    runtime,
    rows,
    summary: totals(rows),
  };
  // Write custody before fallible admission so a real failed call is replayable.
  fs.writeFileSync(
    path.join(nativeDir, ".artifacts/native/formatter-history/public-native-report.json"),
    `${JSON.stringify(report, null, 2)}\n`,
  );
  validatePublicNativeReport(loaded, report, buildBytes, fs.readFileSync(observer));
  return report;
}
