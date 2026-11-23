import { readToolingProgram } from "./support/program-path.ts";
import assert from "node:assert/strict";
import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { captureColdScalePerf } from "../../tools/benchmarks/scripts/cold-typechecker-scale-profile.ts";
import { sourcePerfEnvironment } from "../../tools/benchmarks/scripts/source-operation-perf.ts";

test("source counters execute the direct checker after preparation and retain unavailable PMU evidence", () => {
  const root = mkdtempSync(join(tmpdir(), "cold-scale-direct-perf-"));
  try {
    const provider = join(root, "provider");
    const native = `@typescript/typescript-${process.platform === "darwin" ? "darwin" : "linux"}-${process.arch}`;
    for (const name of ["vue", "typescript", "typescript-native", native]) {
      const dir = join(provider, "node_modules", name);
      mkdirSync(dir, { recursive: true });
      writeFileSync(join(dir, "package.json"), JSON.stringify({ name, version: "0.0.0" }));
    }
    mkdirSync(join(provider, "node_modules", native, "lib"));
    writeFileSync(join(provider, "node_modules", native, "lib", "tsc"), "backend identity witness");
    writeFileSync(join(provider, "package-lock.json"), '{"name":"test provider"}\n');
    const binary = join(root, "checker.cjs");
    // Stubs exercise capture custody; they provide no product speed/accuracy evidence.
    writeFileSync(
      binary,
      readToolingProgram("commands/cold-typechecker-scale-profile/binary.cjs.in", {
        NODE: process.execPath,
      }),
      { mode: 0o755 },
    );
    const perf = join(root, "perf.cjs");
    writeFileSync(
      perf,
      readToolingProgram("commands/cold-typechecker-scale-profile/record-perf-events.cjs.in", {
        NODE: process.execPath,
        BINARY: JSON.stringify(binary),
      }),
      { mode: 0o755 },
    );
    const calibration = join(root, "calibration.json");
    writeFileSync(
      calibration,
      JSON.stringify({
        schemaVersion: 1,
        environmentFingerprint: sourcePerfEnvironment().fingerprint,
        groups: { core: { qualified: true, reasons: [] }, cache: { qualified: true, reasons: [] } },
      }),
    );
    const options = {
      provider,
      files: 2000,
      tool: "cli" as const,
      mode: "stat" as const,
      perf,
      source: "integration fixture",
      binary,
      calibration,
    };
    const out = join(root, "collected");
    const report = captureColdScalePerf({ ...options, out });
    assert.equal(report.qualified, true);
    const receipt = JSON.parse(readFileSync(join(out, "direct-process", "receipt.json"), "utf8"));
    assert.equal(receipt.command.executable, binary);
    assert.equal(receipt.command.args[0], "check");
    assert.equal(receipt.identity.sfcFiles, 177);
    assert.equal(receipt.identity.cold.sourceConfigIncremental, false);
    assert.equal(receipt.identity.cold.effectiveBackendIncremental, true);
    assert.equal(receipt.identity.cold.cacheQualified, false);
    assert.equal(receipt.identity.backendSha256.length, 64);
    assert.equal(receipt.identity.providerLockSha256.length, 64);
    assert.equal(readFileSync(join(out, "tmp", "checker-ran"), "utf8"), "yes");
    const uncalibrated = captureColdScalePerf({
      ...options,
      calibration: undefined,
      out: join(root, "uncalibrated"),
    });
    assert.equal(uncalibrated.collection, "collected");
    assert.equal(uncalibrated.correctness.passed, true);
    assert.equal(
      uncalibrated.qualified,
      false,
      "raw collection alone does not establish a qualified counter",
    );
    writeFileSync(
      perf,
      readToolingProgram("commands/cold-typechecker-scale-profile/deny-perf-permission.cjs.in", {
        NODE: process.execPath,
      }),
      { mode: 0o755 },
    );
    const denied = join(root, "unavailable");
    const missing = captureColdScalePerf({ ...options, out: denied });
    assert.equal(missing.collection, "unavailable");
    assert.equal(missing.qualified, false);
    assert.equal(missing.correctness.passed, false);
    assert.equal(existsSync(join(denied, "tmp", "checker-ran")), false);
    assert.match(
      readFileSync(join(denied, "direct-process", "stderr.txt"), "utf8"),
      /No permission/u,
    );
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
