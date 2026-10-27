import assert from "node:assert/strict";
import { chmodSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import {
  captureSourceOperationPerf,
  parsePerfEventStats,
  parsePerfStat,
  sourcePerfEnvironment,
} from "../../tools/benchmarks/scripts/source-operation-perf.ts";

test("PMU unavailable and multiplexing evidence remain distinct from measured zero", () => {
  const rows = parsePerfStat(
    "0,,branch-misses,100,33.3\n<not supported>,,instructions,0,0\n<not counted>,,cycles,0,0\n",
  );
  assert.equal(rows[0].value, 0);
  assert.equal(rows[0].available, true);
  assert.equal(rows[0].runningPercent, "33.3");
  assert.equal(rows[0].interpretation, "scaled estimate from multiplexed event");
  assert.equal(rows[1].value, null);
  assert.equal(rows[1].available, false);
  assert.equal(rows[2].value, null);
});

test("direct command stdout and nonzero child status survive successful collection", () => {
  const root = mkdtempSync(join(tmpdir(), "source-perf-"));
  try {
    const fake = join(root, "perf-fixture");
    writeFileSync(
      fake,
      `#!${process.execPath}
const {writeFileSync}=require('node:fs');
const {spawnSync}=require('node:child_process');
const a=process.argv.slice(2), sep=a.indexOf('--');
if(a[0]==='--version'){console.log('fixture-perf 1');process.exit(0);}
const events=a[a.indexOf('-e')+1].split(',');
writeFileSync(a[a.indexOf('--output')+1],events.map(e=>'7,,'+e+',100,50.00').join('\\n'));
const c=spawnSync(a[sep+1],a.slice(sep+2),{stdio:'inherit',env:process.env});
process.exit(c.status);
`,
    );
    chmodSync(fake, 0o755);
    const out = join(root, "evidence");
    const receipt = captureSourceOperationPerf({
      mode: "stat",
      perfExecutable: fake,
      outputDir: out,
      command: {
        executable: process.execPath,
        args: ["-e", "console.log(process.env.PERF_TEST_UNIT);process.exit(2)"],
        cwd: root,
        env: { PERF_TEST_UNIT: "exact-output" },
      },
      identity: { source: "fake-integration-control" },
    });
    assert.equal(receipt.collection, "collected");
    assert.equal(receipt.perf.status, 2);
    assert.equal(receipt.commandExit.status, 2);
    assert.equal(receipt.counters.length, 8);
    assert.equal(receipt.qualified, false, "uncalibrated, multiplexed counts are not qualified");
    assert.equal(receipt.binarySha256.length, 64);
    assert.equal(readFileSync(join(out, "stdout.txt"), "utf8"), "exact-output\n");
    assert.throws(
      () =>
        captureSourceOperationPerf({
          mode: "stat",
          perfExecutable: fake,
          outputDir: out,
          command: receipt.command,
          identity: {},
        }),
      /Refusing to overwrite/u,
    );
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test("permission failure retains raw stderr and never fabricates command acceptance", () => {
  const root = mkdtempSync(join(tmpdir(), "source-perf-denied-"));
  try {
    const fake = join(root, "perf-denied");
    writeFileSync(
      fake,
      `#!${process.execPath}\nconsole.error('No permission to enable instructions event.');process.exit(255);\n`,
    );
    chmodSync(fake, 0o755);
    const out = join(root, "evidence");
    const receipt = captureSourceOperationPerf({
      mode: "stat",
      perfExecutable: fake,
      outputDir: out,
      command: {
        executable: process.execPath,
        args: ["-e", "throw Error('must not run')"],
        cwd: root,
        env: {},
      },
      identity: {},
    });
    assert.equal(receipt.collection, "unavailable");
    assert.equal(receipt.commandExit.status, null);
    assert.equal(receipt.perf.status, 255);
    assert.equal(receipt.missingEvents.length, 8);
    assert.match(readFileSync(join(out, "stderr.txt"), "utf8"), /No permission/u);
    assert.equal(readFileSync(join(out, "stdout.txt"), "utf8"), "");
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test("positive instructions with zero cycles retain raw data and fail hardware sanity", () => {
  const root = mkdtempSync(join(tmpdir(), "source-perf-zero-cycles-"));
  try {
    const fake = join(root, "perf-fixture");
    writeFileSync(
      fake,
      `#!${process.execPath}
const fs=require('node:fs'),a=process.argv.slice(2);
if(a[0]==='--version'){console.log('fixture-perf');process.exit(0);}
fs.writeFileSync(a[a.indexOf('--output')+1],a[a.indexOf('-e')+1].split(',').map(e=>(e==='instructions'?'312731498115':'0')+',,'+e+',100,100.00').join('\\n'));
console.log('executed output');\n`,
      { mode: 0o755 },
    );
    const receipt = captureSourceOperationPerf({
      mode: "stat",
      perfExecutable: fake,
      outputDir: join(root, "evidence"),
      command: { executable: process.execPath, args: [], cwd: root, env: {} },
      identity: {},
    });
    assert.equal(receipt.collection, "unavailable");
    assert.equal(receipt.qualified, false);
    assert.equal(receipt.counters[0].value, 312731498115, "untrusted raw count is retained");
    assert.deepEqual(receipt.counterSanity.reasons, [
      "positive instructions with zero cycles",
      "large instruction count with zero branches",
    ]);
    assert.equal(readFileSync(join(root, "evidence", "stdout.txt"), "utf8"), "executed output\n");
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test("same-machine full-scheduled calibration qualifies core counters and rejects identity drift", () => {
  const root = mkdtempSync(join(tmpdir(), "source-perf-calibrated-"));
  try {
    const fake = join(root, "perf-fixture"),
      calibrationPath = join(root, "calibration.json");
    writeFileSync(
      fake,
      `#!${process.execPath}
const fs=require('node:fs'),a=process.argv.slice(2);
if(a[0]==='--version'){console.log('fixture-perf 1');process.exit(0);}
fs.writeFileSync(a[a.indexOf('--output')+1],a[a.indexOf('-e')+1].split(',').map(e=>'7,,'+e+',100,100.00').join('\\n'));\n`,
      { mode: 0o755 },
    );
    const metadata = {
      schemaVersion: 1,
      environmentFingerprint: sourcePerfEnvironment().fingerprint,
      groups: { core: { qualified: true } },
    };
    writeFileSync(calibrationPath, JSON.stringify(metadata));
    const request = {
      mode: "stat" as const,
      perfExecutable: fake,
      command: { executable: process.execPath, args: [], cwd: root, env: {} },
      identity: { fixture: true },
      calibrationPath,
    };
    const accepted = captureSourceOperationPerf({ ...request, outputDir: join(root, "accepted") });
    assert.equal(accepted.qualified, true);
    assert.equal(accepted.binarySha256, accepted.binarySha256After);
    assert.equal(accepted.perf.version.stdout, "fixture-perf 1\n");
    writeFileSync(
      calibrationPath,
      JSON.stringify({ ...metadata, environmentFingerprint: "different-machine" }),
    );
    const changed = captureSourceOperationPerf({ ...request, outputDir: join(root, "changed") });
    assert.equal(changed.collection, "collected");
    assert.equal(changed.qualified, false);
    assert.deepEqual(changed.calibration.reasons, [
      "calibration CPU/kernel/affinity identity mismatch",
    ]);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test("kernel-symbol permission warnings preserve samples while source errors and report timeouts fail qualification", () => {
  const root = mkdtempSync(join(tmpdir(), "source-perf-record-"));
  try {
    const fake = join(root, "perf-fixture");
    writeFileSync(
      fake,
      `#!${process.execPath}
const fs=require('node:fs'),a=process.argv.slice(2);
if(a[0]==='--version'){console.log('fixture-perf');process.exit(0);}
if(process.env.PERF_CONFIG!=='/dev/null')throw Error('ambient perf config must not be inherited');
if(a[0]==='report'&&a.includes('--stats')){console.log(process.env.NO_CENSUS?'Aggregated stats: TOTAL events: 1':'Aggregated stats:\\n SAMPLE events: 12 (100.0%)');process.exit(0);}
if(a[0]==='report'){
 if(process.env.SLOW_REPORT){setTimeout(()=>{},10000);}
 else {console.log('100.0% fixture_function fixture.rs:'+(process.env.ZERO_LINE?'0':'42')); if(process.env.BAD_REPORT)console.error('addr2line fixture: could not read first record');}
}else{
 if(!a.includes('--no-buildid')||!a.includes('--no-buildid-cache'))throw Error('record post-pass must be disabled');
 fs.writeFileSync(a[a.indexOf('--output')+1],'retained trace');
 console.error('WARNING: Kernel address maps are restricted. Check /proc/kallsyms permission or run as root.');
 console.error('workload note: optional feature not supported');
 console.error('[ perf record: Captured and wrote 1.0 MB trace ]');
}\n`,
      { mode: 0o755 },
    );
    const request = {
      mode: "record" as const,
      perfExecutable: fake,
      command: { executable: process.execPath, args: [], cwd: root, env: {} },
      identity: { fixture: true },
    };
    const good = captureSourceOperationPerf({ ...request, outputDir: join(root, "good") });
    assert.equal(good.collection, "collected");
    assert.equal(good.sampleCount, 12);
    assert.equal(good.recordSampleCount, null, "no-buildid record does not print a count");
    assert.equal(good.eventCensus?.status, 0);
    assert.equal(good.qualified, true);
    assert.equal(good.report?.sourceRows, 1);
    const noCensus = captureSourceOperationPerf({
      ...request,
      outputDir: join(root, "no-census"),
      command: { ...request.command, env: { NO_CENSUS: "1" } },
    });
    assert.equal(noCensus.collection, "collected");
    assert.equal(noCensus.sampleCount, null);
    assert.equal(noCensus.qualified, false, "unobserved counts are not fabricated zero or success");

    const bad = captureSourceOperationPerf({
      ...request,
      outputDir: join(root, "bad"),
      command: { ...request.command, env: { BAD_REPORT: "1" } },
    });
    assert.equal(bad.collection, "collected");
    assert.equal(bad.report?.status, 0);
    assert.equal(bad.qualified, false, "exit zero cannot hide a broken addr2line protocol");
    const zeroLine = captureSourceOperationPerf({
      ...request,
      outputDir: join(root, "zero-line"),
      command: { ...request.command, env: { ZERO_LINE: "1" } },
    });
    assert.equal(zeroLine.report?.sourceRows, 0);
    assert.equal(zeroLine.qualified, false, "line zero is not a resolved source location");
    const slow = captureSourceOperationPerf({
      ...request,
      outputDir: join(root, "slow"),
      reportTimeoutMs: 50,
      command: { ...request.command, env: { SLOW_REPORT: "1" } },
    });
    assert.equal(slow.collection, "collected");
    assert.equal(slow.qualified, false);
    assert.equal(readFileSync(join(root, "slow", "perf.data"), "utf8"), "retained trace");
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test("a missing perf executable or invalid perf invocation is an infrastructure failure", () => {
  const root = mkdtempSync(join(tmpdir(), "source-perf-command-failure-"));
  try {
    const command = { executable: process.execPath, args: [], cwd: root, env: {} };
    const absent = captureSourceOperationPerf({
      mode: "stat",
      perfExecutable: join(root, "absent"),
      outputDir: join(root, "absent-output"),
      command,
      identity: {},
    });
    assert.equal(absent.collection, "failed");
    assert.equal(absent.qualified, false);
    const bad = join(root, "invalid-perf.cjs");
    writeFileSync(
      bad,
      `#!${process.execPath}\nif(process.argv.includes('--version')){console.log('fixture-perf');}else{console.error('usage: perf stat invalid option');process.exit(129);}\n`,
      { mode: 0o755 },
    );
    const invalid = captureSourceOperationPerf({
      mode: "stat",
      perfExecutable: bad,
      outputDir: join(root, "invalid-output"),
      command,
      identity: {},
    });
    assert.equal(invalid.collection, "failed");
    assert.equal(invalid.qualified, false);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test("count-only event statistics retain exact counts without abbreviated report units", () => {
  assert.equal(parsePerfEventStats("Aggregated stats:\n SAMPLE events: 2205 (90.5%)\n"), 2205);
  assert.equal(parsePerfEventStats("PERF_RECORD_SAMPLE: 12\n"), 12);
  assert.equal(parsePerfEventStats("SAMPLE events: 2K\n"), null);
  assert.equal(parsePerfEventStats("TOTAL events: 12\n"), null);
});
