import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import {
  mixedCells,
  joinMixedCaptures,
  runMixedWorkers,
} from "../../npm/oxlint/src/test-support/mixed-directory-8507-workers.mjs";

const root = fileURLToPath(new URL("../..", import.meta.url));
const plan = JSON.parse(
  fs.readFileSync(
    path.join(root, "tests/_fixtures/differential/lint/oxlint-mixed-directory-8507/controls.json"),
  ),
);
const custody = { schema: "partition-control", source: { head: "unit-only" }, calls: "joined" };
function controls() {
  return [0, 1].map((workerIndex) => {
    const cells = mixedCells(plan).filter((_, index) => index % 2 === workerIndex);
    return {
      workerIndex,
      complete: true,
      source: custody.source,
      custody: { ...custody, calls: `worker-${workerIndex}/source-native-calls.jsonl` },
      environment: { VIZE_OXLINT_NATIVE_CUSTODY: `worker-${workerIndex}/source-custody.json` },
      initialEvents: [],
      observations: cells.flatMap(({ fixture, format }) =>
        fixture.surface === "cli"
          ? ["stock", "wrapper"].map((surface) => ({ fixture: fixture.name, format, surface }))
          : [{ fixture: fixture.name, surface: "native" }],
      ),
      passed: {
        cli: cells.filter(({ fixture }) => fixture.surface === "cli").length,
        native: cells.filter(({ fixture }) => fixture.surface === "native").length,
      },
    };
  });
}

await test("fixed partitions retain every original CLI/native cell and canonical observation order", () => {
  const captures = controls();
  assert.deepEqual(
    captures.map((capture) => capture.passed),
    [
      { cli: 23, native: 4 },
      { cli: 22, native: 4 },
    ],
  );
  const joined = joinMixedCaptures(plan, custody, captures);
  assert.deepEqual(joined.passed, { cli: 45, native: 8 });
  assert.equal(joined.observations.length, 98);
  assert.deepEqual(
    joined.observations.map((record) => [record.fixture, record.format ?? null, record.surface]),
    plan.cases.flatMap((fixture) =>
      fixture.surface === "cli"
        ? ["implicit-default", "default", "json", "unix", "stylish"].flatMap((format) =>
            ["stock", "wrapper"].map((surface) => [fixture.name, format, surface]),
          )
        : [[fixture.name, null, "native"]],
    ),
  );
});

await test("duplicate, missing, reordered, incomplete and foreign-authority controls cannot qualify", () => {
  for (const corrupt of [
    (captures) => captures[0].observations.push(captures[0].observations[0]),
    (captures) => captures[1].observations.pop(),
    (captures) => captures[0].observations.reverse(),
    (captures) => {
      captures[1].complete = false;
    },
    (captures) => {
      captures[1].source.head = "foreign";
    },
    (captures) => {
      captures[1].custody.source.head = "foreign";
    },
    (captures) => {
      captures[1].workerIndex = 0;
    },
    (captures) => {
      captures[1].custody.calls = captures[0].custody.calls;
    },
    (captures) => {
      captures[1].environment.VIZE_OXLINT_NATIVE_CUSTODY = "foreign";
    },
  ]) {
    const captures = structuredClone(controls());
    corrupt(captures);
    assert.throws(() => joinMixedCaptures(plan, custody, captures));
  }
});

async function lifecycle(fatal) {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-mixed-worker-control-"));
  const output = path.join(directory, "output");
  fs.mkdirSync(output);
  fs.mkdirSync(path.join(directory, "src"));
  const preload = path.join(directory, "empty-control.cjs");
  fs.writeFileSync(preload, "// Process-only control; no addon or source-acceptance claims.\n");
  fs.writeFileSync(
    path.join(directory, "src/mixed-directory-8507.test.mjs"),
    `
import fs from 'node:fs';
import { spawn } from 'node:child_process';
import { setTimeout } from 'node:timers/promises';
const index = process.env.VIZE_OXLINT_MIXED_WORKER;
${
  fatal
    ? `
if (index === '0') {
  spawn(process.execPath, ['-e', "const fs=require('node:fs'); process.on('SIGTERM',()=>{fs.writeFileSync('descendant-stopped',String(process.pid));process.exit(0)}); fs.writeFileSync('descendant-started',String(process.pid)); setInterval(()=>{},1000)"], { stdio: ['ignore', 'inherit', 'inherit'] });
  while (!fs.existsSync('descendant-started')) await setTimeout(10);
}
`
    : ""
}
fs.writeFileSync('started-' + index, String(process.pid));
for (let attempt = 0; !fs.existsSync('started-' + (1 - Number(index))); attempt++) {
  if (attempt > 200) throw Error('fixed workers were serialized');
  await setTimeout(10);
}
await new Promise((resolve) => process.stdout.write('control-worker-' + index + '\\n', resolve));
await new Promise((resolve) => process.stderr.write('control-stderr-' + index + '\\n', resolve));
fs.writeFileSync('ready-' + index, 'streams flushed');
${fatal ? "if (index === '0') { while (!fs.existsSync('ready-1')) await setTimeout(10); process.exit(7); } await setTimeout(60_000);" : ""}
`,
  );
  try {
    if (fatal)
      await assert.rejects(
        runMixedWorkers({
          packageDir: directory,
          output,
          custody,
          preload,
          environment: process.env,
        }),
      );
    else
      await runMixedWorkers({
        packageDir: directory,
        output,
        custody,
        preload,
        environment: process.env,
      });
    const packet = JSON.parse(fs.readFileSync(path.join(output, "source-process.json")));
    assert.equal(packet.deadlineMs, 180_000);
    assert.equal(packet.maxBufferBytes, 64 * 1024 * 1024);
    assert.equal(
      packet.totalOutputBytes,
      packet.workers.reduce(
        (sum, worker) => sum + worker.stdoutBytes.length + worker.stderrBytes.length,
        0,
      ),
    );
    assert.equal(packet.workers.length, 2);
    assert.notEqual(packet.workers[0].pid, packet.workers[1].pid);
    assert.notEqual(packet.workers[0].calls, packet.workers[1].calls);
    assert.notEqual(packet.workers[0].capture, packet.workers[1].capture);
    for (const [index, worker] of packet.workers.entries()) {
      const retained = JSON.parse(
        fs.readFileSync(path.join(output, `worker-${index}/source-process.json`)),
      );
      assert.deepEqual(retained, worker);
      assert.equal(worker.error, null);
      assert.equal(Buffer.from(worker.stdoutBytes).toString(), `control-worker-${index}\n`);
      assert.equal(Buffer.from(worker.stderrBytes).toString(), `control-stderr-${index}\n`);
    }
    if (fatal) {
      assert.equal(packet.failure, "worker 0 fatal terminal");
      assert.equal(packet.workers[0].status, 7);
      assert.equal(packet.workers[0].signal, null);
      assert.equal(packet.workers[1].status, null);
      assert.equal(packet.workers[1].signal, "SIGTERM");
      assert.equal(
        fs.readFileSync(path.join(directory, "descendant-stopped"), "utf8"),
        fs.readFileSync(path.join(directory, "descendant-started"), "utf8"),
      );
    } else {
      assert.equal(packet.failure, null);
      for (const worker of packet.workers) {
        assert.equal(worker.status, 0);
        assert.equal(worker.signal, null);
      }
    }
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
}

await test("real process-only controls start concurrently and preserve separate whole terminal packets", async () => {
  await lifecycle(false);
});
await test("a real fatal terminal forwards failure and drains the other owned worker", async () => {
  await lifecycle(true);
});
