import assert from "node:assert/strict";
import { test } from "node:test";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  baselineMode,
  baselineModes,
  baselinePartitions,
  admittedPartition,
  validatePartitionRows,
  seedOriginalSources,
} from "./n8n-baseline-partitions.mjs";
import { runBaselineProcesses } from "./n8n-baseline-processes.mjs";

const inventory = {
  files: ["a.vue", "b.vue", "c.vue"].map((file, index) => ({ file, sha256: `source-${index}` })),
};
const rules = Object.fromEntries(
  Array.from({ length: 51 }, (_, index) => [`vize/rule-${index}`, ["error", { literal: index }]]),
);
const rows = (part) =>
  inventory.files.slice(part.start, part.end).flatMap((entry) =>
    baselineModes.map((mode) => {
      const cold = Object.keys(rules).map((rule) => ({
        rule,
        configured: rules[rule],
        options: rules[rule][1],
        diagnostics: [
          { message: "日本語\r\n", location: { start: { offset: 2 }, end: { offset: 7 } } },
        ],
      }));
      return {
        file: entry.file,
        sourceSha256: entry.sha256,
        mode,
        rules,
        cold,
        warm: structuredClone(cold),
        coldCalls: mode.endsWith("PerRule") ? 51 : 1,
        warmCalls: 0,
        elapsedMs: 1,
        cache: { capacity: 128, entries: 1 },
      };
    }),
  );

void test("fixed input ranges cover the original ordered1369 once and admit no arbitrary partition", () => {
  const actual = baselinePartitions(1369);
  assert.deepEqual(actual, [
    { index: 0, start: 0, end: 685, controls: false },
    { index: 1, start: 685, end: 1369, controls: true },
  ]);
  assert.deepEqual(
    actual.flatMap((part) =>
      Array.from({ length: part.end - part.start }, (_, i) => i + part.start),
    ),
    Array.from({ length: 1369 }, (_, i) => i),
  );
  assert.equal(baselineMode({}), "serial");
  assert.equal(baselineMode({ VIZE_N8N_BASELINE_MODE: "partitioned" }), "partitioned");
  assert.throws(() => baselineMode({ VIZE_N8N_BASELINE_MODE: "three" }));
  assert.equal(admittedPartition(inventory, {}), null);
  const expected = baselinePartitions(3)[1];
  assert.deepEqual(
    admittedPartition(inventory, { VIZE_N8N_BASELINE_PARTITION: JSON.stringify(expected) }),
    expected,
  );
  for (const mutation of [
    { ...expected, start: 0 },
    { ...expected, controls: false },
    { ...expected, extra: true },
    { ...expected, index: 2 },
  ])
    assert.throws(() =>
      admittedPartition(inventory, { VIZE_N8N_BASELINE_PARTITION: JSON.stringify(mutation) }),
    );
});

void test("whole51-rule vectors survive validation and missing, duplicate, reordered or substituted rows fail", () => {
  const part = baselinePartitions(3)[0],
    original = rows(part),
    before = structuredClone(original);
  const totals = validatePartitionRows(inventory, part, original, () => rules);
  assert.deepEqual(original, before);
  assert.equal(Object.keys(totals).length, 6);
  const mutations = [
    (x) => x.pop(),
    (x) => x.push(x[0]),
    (x) => {
      x[6] = structuredClone(x[0]);
    },
    (x) => {
      [x[0], x[1]] = [x[1], x[0]];
    },
    (x) => {
      x[0].sourceSha256 = "wrong";
    },
    (x) => {
      x[0].rules["vize/rule-50"][1].literal = -1;
    },
    (x) => {
      x[0].warm[50].diagnostics[0].location.end.offset = 8;
    },
    (x) => {
      x[0].warmCalls = 1;
    },
  ];
  for (const mutate of mutations) {
    const candidate = structuredClone(original);
    mutate(candidate);
    assert.throws(() => validatePartitionRows(inventory, part, candidate, () => rules));
  }
});

void test("parent source backing is complete for duplicates and refuses source or stored-byte corruption", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-n8n-source-store-control-"));
  try {
    const fixture = path.join(root, "fixture"),
      directory = path.join(root, "sources");
    fs.mkdirSync(fixture);
    const bytes = Buffer.from("日本語\r\n<template/>\r\n");
    const sha256 = createHash("sha256").update(bytes).digest("hex");
    fs.writeFileSync(path.join(fixture, "a.vue"), bytes);
    fs.writeFileSync(path.join(fixture, "b.vue"), bytes);
    const original = {
      fixture,
      files: [
        { file: "a.vue", sha256 },
        { file: "b.vue", sha256 },
      ],
    };
    assert.deepEqual(seedOriginalSources(original, directory), { files: 2, uniqueSources: 1 });
    const target = path.join(directory, sha256 + ".vue");
    assert.deepEqual(fs.readFileSync(target), bytes);
    assert.deepEqual(seedOriginalSources(original, directory), { files: 2, uniqueSources: 1 });
    fs.writeFileSync(target, bytes.subarray(0, 1));
    assert.throws(() => seedOriginalSources(original, directory));
    fs.writeFileSync(target, bytes);
    fs.writeFileSync(path.join(fixture, "b.vue"), "changed");
    assert.throws(() => seedOriginalSources(original, directory));
  } finally {
    fs.rmSync(root, { recursive: true });
  }
});

const make = () => fs.mkdtempSync(path.join(os.tmpdir(), "vize-n8n-partition-control-"));
const jobs = (root, source) =>
  [0, 1].map((index) => ({
    index,
    output: path.join(root, `worker-${index}`),
    command: process.execPath,
    args: ["--input-type=commonjs", "-e", source(index, root)],
    cwd: root,
    env: { ...process.env, NODE_OPTIONS: "" },
  }));
const packet = (root) => JSON.parse(fs.readFileSync(path.join(root, "baseline-processes.json")));
const barrier = (index, root) =>
  `const fs=require('node:fs'); const root=${JSON.stringify(root)}; fs.writeFileSync(root+'/${index}.ready',''); const wait=setInterval(()=>{if(!fs.existsSync(root+'/${1 - index}.ready'))return;clearInterval(wait);`;

void test("two actual children overlap, close normally and retain complete separate UTF8 byte streams", async () => {
  const root = make();
  try {
    const result = await runBaselineProcesses(
      jobs(
        root,
        (index, base) =>
          barrier(index, base) +
          `process.stdout.write(${JSON.stringify(`\r\n child${index} 日本語 \r\n\r\n`)});process.stderr.write(${JSON.stringify(` err${index}\n`)});});`,
      ),
      root,
      { deadlineMs: 2_000 },
    );
    assert.equal(result.status, 0);
    assert.equal(result.error, undefined);
    assert.equal(result.signal, null);
    const observed = packet(root);
    assert.equal(observed.workers.length, 2);
    assert.equal(observed.failure, null);
    for (const [index, worker] of observed.workers.entries()) {
      assert.equal(worker.closed, true);
      assert.equal(worker.status, 0);
      assert.equal(worker.signal, null);
      assert.equal(
        fs.readFileSync(path.join(worker.output, "worker.stdout.bin"), "utf8"),
        `\r\n child${index} 日本語 \r\n\r\n`,
      );
      assert.equal(
        fs.readFileSync(path.join(worker.output, "worker.stderr.bin"), "utf8"),
        ` err${index}\n`,
      );
    }
    assert.equal(
      observed.totalOutputBytes,
      observed.workers.reduce((sum, x) => sum + x.stdoutBytes + x.stderrBytes, 0),
    );
  } finally {
    fs.rmSync(root, { recursive: true });
  }
});

void test("one shared byte cap counts both workers and both streams, rather than doubling the old bound", async () => {
  const root = make();
  try {
    const result = await runBaselineProcesses(
      jobs(
        root,
        (index, base) =>
          barrier(index, base) +
          `process.stdout.write('abc');process.stderr.write('def');setInterval(()=>{},1000);});`,
      ),
      root,
      { deadlineMs: 2_000, maxBufferBytes: 10 },
    );
    assert.equal(result.status, 1);
    assert.match(result.error.message, /shared original 32MiB/u);
    const observed = packet(root);
    assert.equal(observed.maxBufferBytes, 10);
    assert.ok(observed.totalOutputBytes > 10);
    assert.ok(observed.workers.every((x) => x.stdoutBytes + x.stderrBytes <= 6));
    assert.ok(observed.workers.every((x) => x.closed));
  } finally {
    fs.rmSync(root, { recursive: true });
  }
});

void test("fatal exit forwards failure and kills/reaps the other owned descendant group before completion", async () => {
  const root = make();
  try {
    const result = await runBaselineProcesses(
      jobs(root, (index, base) =>
        index === 0
          ? `const fs=require('node:fs');const timer=setInterval(()=>{if(!fs.existsSync(${JSON.stringify(path.join(base, "descendant.ready"))}))return;clearInterval(timer);process.stdout.write('strict7');process.exit(7);},5);`
          : `const fs=require('node:fs');const {spawn}=require('node:child_process');spawn(process.execPath,['-e',${JSON.stringify(`require('node:fs').writeFileSync(${JSON.stringify(path.join(base, "descendant.ready"))},String(process.pid));setInterval(()=>{},1000);`)}],{stdio:['ignore',process.stdout,process.stderr]});setInterval(()=>{},1000);`,
      ),
      root,
      { deadlineMs: 2_000 },
    );
    assert.equal(result.status, 1);
    assert.match(result.error.message, /fatal terminal/u);
    const observed = packet(root);
    assert.equal(observed.workers[0].status, 7);
    assert.equal(observed.workers[1].signal, "SIGKILL");
    assert.ok(observed.workers.every((x) => x.closed));
    const pid = Number(fs.readFileSync(path.join(root, "descendant.ready")));
    const state = spawnSync("ps", ["-o", "stat=", "-p", String(pid)], { encoding: "utf8" });
    assert.ok(
      state.status !== 0 || state.stdout.trim().startsWith("Z"),
      "owned descendant must no longer execute",
    );
    assert.equal(
      fs.readFileSync(path.join(observed.workers[0].output, "worker.stdout.bin"), "utf8"),
      "strict7",
    );
  } finally {
    fs.rmSync(root, { recursive: true });
  }
});

void test("a normally closed worker's ignored-stdio descendant is drained before a later sibling failure", async () => {
  const root = make();
  const ready = path.join(root, "ignored-descendant.ready");
  try {
    const result = await runBaselineProcesses(
      jobs(root, (index, base) =>
        index === 0
          ? `const fs=require('node:fs');require('node:child_process').spawn(process.execPath,['-e',${JSON.stringify(`require('node:fs').writeFileSync(${JSON.stringify(ready)},String(process.pid));setInterval(()=>{},1000);`)}],{stdio:'ignore'}).unref();const wait=setInterval(()=>{if(!fs.existsSync(${JSON.stringify(ready)}))return;clearInterval(wait);process.stdout.write('normal0\\r\\n');process.exit(0);},5);`
          : `const fs=require('node:fs');const file=${JSON.stringify(path.join(base, "worker-0/worker-process.json"))};const wait=setInterval(()=>{if(!fs.existsSync(file))return;const record=JSON.parse(fs.readFileSync(file));if(!record.closed)return;clearInterval(wait);process.stdout.write('strict7');process.exit(7);},5);`,
      ),
      root,
      { deadlineMs: 2_000 },
    );
    assert.equal(result.status, 1);
    assert.match(result.error.message, /fatal terminal/u);
    const observed = packet(root);
    assert.equal(observed.workers[0].status, 0);
    assert.equal(observed.workers[0].signal, null);
    assert.equal(observed.workers[1].status, 7);
    assert.ok(observed.workers.every((x) => x.closed));
    assert.equal(
      fs.readFileSync(path.join(observed.workers[0].output, "worker.stdout.bin"), "utf8"),
      "normal0\r\n",
    );
    assert.equal(
      fs.readFileSync(path.join(observed.workers[1].output, "worker.stdout.bin"), "utf8"),
      "strict7",
    );
    const state = spawnSync("ps", ["-o", "stat=", "-p", fs.readFileSync(ready, "utf8")], {
      encoding: "utf8",
    });
    assert.ok(
      state.status !== 0 || state.stdout.trim().startsWith("Z"),
      "closed worker's owned descendant must no longer execute",
    );
  } finally {
    if (fs.existsSync(ready)) {
      try {
        process.kill(Number(fs.readFileSync(ready)), "SIGKILL");
      } catch (error) {
        assert.equal(error.code, "ESRCH");
      }
    }
    fs.rmSync(root, { recursive: true });
  }
});

void test("shared deadline remains a strict failure and all started children close", async () => {
  const root = make();
  try {
    const result = await runBaselineProcesses(
      jobs(root, () => "setInterval(()=>{},1000);"),
      root,
      { deadlineMs: 100 },
    );
    assert.equal(result.status, 1);
    assert.match(result.error.message, /shared original 900s/u);
    assert.equal(packet(root).deadlineMs, 100);
    assert.ok(packet(root).workers.every((x) => x.closed));
  } finally {
    fs.rmSync(root, { recursive: true });
  }
});

void test("spawn failure cannot become aggregate success and the active sibling drains", async () => {
  const root = make();
  try {
    const selected = jobs(root, () => "setInterval(()=>{},1000);");
    selected[0].command = path.join(root, "missing-vp");
    const result = await runBaselineProcesses(selected, root, { deadlineMs: 2_000 });
    assert.equal(result.status, 1);
    assert.ok(packet(root).workers[0].error);
    assert.ok(packet(root).workers.every((x) => x.closed));
  } finally {
    fs.rmSync(root, { recursive: true });
  }
});

void test("expired source preparation refuses input reads before worker admission", () => {
  const expired = () => assert.fail("shared original 900s deadline");
  assert.throws(
    () => seedOriginalSources(inventory, os.tmpdir(), expired),
    /shared original 900s/u,
  );
});
