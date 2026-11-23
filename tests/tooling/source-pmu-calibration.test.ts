import { readToolingProgram } from "./support/program-path.ts";
import assert from "node:assert/strict";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { calibrateSourcePmu } from "../../tools/benchmarks/scripts/source-pmu-calibration.ts";

void test("calibration admits distinct workload signals and rejects zero-cycle or incomplete controls", () => {
  const root = mkdtempSync(join(tmpdir(), "source-pmu-calibration-"));
  const oldZero = process.env.PMU_FIXTURE_ZERO_CYCLES;
  const oldBad = process.env.PMU_FIXTURE_BAD_WORKLOAD;
  const oldSetup = process.env.PMU_FIXTURE_CACHE_SETUP_ONLY;
  try {
    const perf = join(root, "fixture-perf.cjs"),
      compiler = join(root, "fixture-rustc.cjs");
    writeFileSync(
      compiler,
      readToolingProgram("commands/source-pmu-calibration/compiler.cjs.in", {
        NODE: process.execPath,
      }),
      { mode: 0o755 },
    );
    // This external protocol control tests evidence admission, not the PMU or Rust workload.
    writeFileSync(
      perf,
      readToolingProgram("commands/source-pmu-calibration/perf.cjs.in", { NODE: process.execPath }),
      { mode: 0o755 },
    );
    const options = (name: string) => ({
      binary: join(root, "carrier-" + name),
      perf,
      rounds: 1,
      iterations: 1000,
      rustcExecutable: compiler,
      out: join(root, name),
    });
    const accepted = calibrateSourcePmu(options("accepted"));
    assert.equal(accepted.qualified, true);
    assert.equal(accepted.groups.core.qualified, true);
    assert.equal(accepted.groups.cache.qualified, true);
    assert.equal(accepted.controls.length, 7);
    assert.equal(accepted.binarySha256, accepted.build.binarySha256);
    assert.equal(accepted.sourceSha256, accepted.build.sourceSha256);
    assert.equal(accepted.build.qualified, true);
    assert.deepEqual(
      JSON.parse(readFileSync(accepted.build.binary + ".compiler-argv.json", "utf8")),
      accepted.build.compiler.args,
    );
    for (const control of accepted.controls) {
      const raw = JSON.parse(readFileSync(join(root, "accepted", control.receipt), "utf8"));
      assert.equal(raw.binarySha256, accepted.binarySha256);
      assert.equal(raw.binarySha256After, accepted.binarySha256);
      assert.equal(raw.identity.binarySha256, accepted.binarySha256);
      assert.equal(raw.identity.sourceSha256, accepted.sourceSha256);
    }
    assert.throws(
      () => calibrateSourcePmu({ ...options("existing-binary"), binary: accepted.build.binary }),
      /fresh compiler output/,
    );
    assert.equal(accepted.sourceSha256.length, 64);
    assert.deepEqual(accepted.signals, {
      baselineInstructions: 100,
      busyInstructions: 100000000,
      baselineBranches: 10,
      busyBranches: 10000000,
      smallCacheReferences: 100,
      largeCacheReferences: 10000,
      smallCacheMisses: 2,
      largeCacheMisses: 1000,
    });
    process.env.PMU_FIXTURE_CACHE_SETUP_ONLY = "1";
    const setupOnly = calibrateSourcePmu(options("setup-only"));
    assert.equal(setupOnly.groups.core.qualified, true);
    assert.equal(setupOnly.groups.cache.qualified, false);
    assert.equal(setupOnly.signals.largeCacheReferences, 0);
    assert.equal(setupOnly.signals.smallCacheReferences, 0);
    assert.equal(setupOnly.signals.largeCacheMisses, 0);
    delete process.env.PMU_FIXTURE_CACHE_SETUP_ONLY;
    process.env.PMU_FIXTURE_ZERO_CYCLES = "1";
    const invalid = calibrateSourcePmu(options("zero-cycles"));
    assert.equal(invalid.qualified, false);
    assert.equal(invalid.groups.core.qualified, false);
    assert.equal(invalid.groups.cache.qualified, true);
    assert.deepEqual(invalid.groups.core.reasons, ["missing or implausible hardware events"]);
    const raw = JSON.parse(
      readFileSync(join(root, "zero-cycles", "instructions-1", "receipt.json"), "utf8"),
    );
    assert.equal(raw.counters.find((row: { event: string }) => row.event === "cycles").value, 0);
    delete process.env.PMU_FIXTURE_ZERO_CYCLES;
    process.env.PMU_FIXTURE_BAD_WORKLOAD = "1";
    const incomplete = calibrateSourcePmu(options("incomplete"));
    assert.equal(incomplete.qualified, false);
    assert.equal(
      incomplete.controls.every((row) => row.correct === false),
      true,
    );
    assert.deepEqual(incomplete.groups.core.reasons, [
      "a calibration command did not complete its exact workload contract",
    ]);
  } finally {
    if (oldZero === undefined) delete process.env.PMU_FIXTURE_ZERO_CYCLES;
    else process.env.PMU_FIXTURE_ZERO_CYCLES = oldZero;
    if (oldSetup === undefined) delete process.env.PMU_FIXTURE_CACHE_SETUP_ONLY;
    else process.env.PMU_FIXTURE_CACHE_SETUP_ONLY = oldSetup;
    if (oldBad === undefined) delete process.env.PMU_FIXTURE_BAD_WORKLOAD;
    else process.env.PMU_FIXTURE_BAD_WORKLOAD = oldBad;
    rmSync(root, { recursive: true, force: true });
  }
});
