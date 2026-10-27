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
      `#!${process.execPath}
const fs=require('node:fs'),crypto=require('node:crypto'),a=process.argv.slice(2);
if(a[0]==='-Vv'){console.log('fixture compiler; protocol control, not genuine Rust or hardware proof');process.exit(0);}
if(JSON.stringify(a.slice(0,3))!==JSON.stringify(['--edition=2024','-O','-g']))process.exit(2);
const source=a[3],binary=a[a.indexOf('-o')+1];
fs.writeFileSync(binary,'test-only carrier from source '+crypto.createHash('sha256').update(fs.readFileSync(source)).digest('hex'));
fs.writeFileSync(binary+'.compiler-argv.json',JSON.stringify(a));\n`,
      { mode: 0o755 },
    );
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
const setup=mode.startsWith('cache-setup'),large=mode.endsWith('large');
const refs=(large?5000:50)+(!setup&&!process.env.PMU_FIXTURE_CACHE_SETUP_ONLY?(large?10000:100):0);
const misses=(large?100:1)+(!setup&&!process.env.PMU_FIXTURE_CACHE_SETUP_ONLY?(large?1000:2):0);
const count=e=>e==='instructions'?values[0]:e==='cycles'?(process.env.PMU_FIXTURE_ZERO_CYCLES?'0':values[1]):e==='branches'?values[2]:e==='branch-misses'?values[3]:e==='cache-references'?refs:e==='cache-misses'?misses:123;
fs.writeFileSync(a[a.indexOf('--output')+1],events.map(e=>count(e)+',,'+e+',100,100.00').join('\\n'));
console.log(JSON.stringify({schema:'vize-pmu-calibration',mode,iterations,bytes,checksum:'1234567890abcdef',dependentLoads:mode.startsWith('cache-')?(setup?0:iterations):null,completed:!process.env.PMU_FIXTURE_BAD_WORKLOAD}));\n`,
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
