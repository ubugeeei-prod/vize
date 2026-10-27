import assert from "node:assert/strict";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { calibrateSourcePmu } from "../../tools/benchmarks/scripts/source-pmu-calibration.ts";

test("calibration admits distinct workload signals and rejects zero-cycle or incomplete controls", () => {
  const root = mkdtempSync(join(tmpdir(), "source-pmu-calibration-"));
  const oldZero = process.env.PMU_FIXTURE_ZERO_CYCLES;
  const oldBad = process.env.PMU_FIXTURE_BAD_WORKLOAD;
  try {
    const binary = join(root, "fixture-carrier"),
      perf = join(root, "fixture-perf.cjs");
    writeFileSync(binary, "test-only carrier identity; no real hardware claim");
    // This external protocol control tests evidence admission, not the PMU or Rust workload.
    writeFileSync(
      perf,
      `#!${process.execPath}
const fs=require('node:fs'),a=process.argv.slice(2);
if(a[0]==='--version'){console.log('fixture-perf 1');process.exit(0);}
const sep=a.indexOf('--'),mode=a[sep+2],iterations=Number(a[sep+3]),bytes=Number(a[sep+4]);
const signals={baseline:[100,100,10,1],instructions:[100000000,400000000,2000000,200],branches:[50000000,100000000,10000000,300000]};
const values=signals[mode]??[7,7,7,7];
const events=a[a.indexOf('-e')+1].split(',');
const count=e=>e==='instructions'?values[0]:e==='cycles'?(process.env.PMU_FIXTURE_ZERO_CYCLES?'0':values[1]):e==='branches'?values[2]:e==='branch-misses'?values[3]:e==='cache-references'?(mode==='cache-large'?10000:100):e==='cache-misses'?(mode==='cache-large'?1000:2):123;
fs.writeFileSync(a[a.indexOf('--output')+1],events.map(e=>count(e)+',,'+e+',100,100.00').join('\\n'));
console.log(JSON.stringify({schema:'vize-pmu-calibration',mode,iterations,bytes,checksum:'1234567890abcdef',completed:!process.env.PMU_FIXTURE_BAD_WORKLOAD}));\n`,
      { mode: 0o755 },
    );
    const options = { binary, perf, rounds: 1, iterations: 1000 };
    const accepted = calibrateSourcePmu({ ...options, out: join(root, "accepted") });
    assert.equal(accepted.qualified, true);
    assert.equal(accepted.groups.core.qualified, true);
    assert.equal(accepted.groups.cache.qualified, true);
    assert.equal(accepted.controls.length, 5);
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
    process.env.PMU_FIXTURE_ZERO_CYCLES = "1";
    const invalid = calibrateSourcePmu({ ...options, out: join(root, "zero-cycles") });
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
    const incomplete = calibrateSourcePmu({ ...options, out: join(root, "incomplete") });
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
    if (oldBad === undefined) delete process.env.PMU_FIXTURE_BAD_WORKLOAD;
    else process.env.PMU_FIXTURE_BAD_WORKLOAD = oldBad;
    rmSync(root, { recursive: true, force: true });
  }
});
