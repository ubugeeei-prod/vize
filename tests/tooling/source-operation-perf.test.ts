import assert from "node:assert/strict";
import { chmodSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import {
  captureSourceOperationPerf,
  parsePerfStat,
} from "../../tools/benchmarks/scripts/source-operation-perf.ts";

test("PMU unavailable and multiplexing evidence remain distinct from measured zero", () => {
  const rows = parsePerfStat(
    "0,,branch-misses,100,33.3\n<not supported>,,instructions,0,0\n<not counted>,,cycles,0,0\n",
  );
  assert.equal(rows[0].value, 0);
  assert.equal(rows[0].available, true);
  assert.equal(rows[0].runningPercent, "33.3");
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
    assert.equal(receipt.counters.length, 10);
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
    assert.equal(receipt.missingEvents.length, 10);
    assert.match(readFileSync(join(out, "stderr.txt"), "utf8"), /No permission/u);
    assert.equal(readFileSync(join(out, "stdout.txt"), "utf8"), "");
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
