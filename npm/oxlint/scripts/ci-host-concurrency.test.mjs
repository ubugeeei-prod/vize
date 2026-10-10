import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";
import { createHash } from "node:crypto";
import { verifyProjectHostCalls } from "./ci-host-worker.mjs";
import {
  aggregateNativeEvidence,
  parseHostOptions,
  requireSuccessfulHosts,
  runHostProcesses,
} from "./ci-host-concurrency.mjs";

function subprocesses(t, failures = []) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-ci-host-concurrency-"));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const script = path.join(root, "child.mjs");
  fs.writeFileSync(
    script,
    `
    import fs from "node:fs";
    const [directory, id, failure] = process.argv.slice(2);
    fs.writeFileSync(directory + "/start.json", JSON.stringify({ time: Date.now(), pid: process.pid }));
    console.log("started " + id);
    console.error("evidence " + id);
    await new Promise((resolve) => setTimeout(resolve, 200));
    fs.writeFileSync(directory + "/end.json", JSON.stringify({ time: Date.now() }));
    process.exitCode = Number(failure);
  `,
  );
  return ["first", "second", "third"].map((id) => {
    const output = path.join(root, id);
    return {
      id,
      output,
      command: process.execPath,
      args: [script, output, id, failures.includes(id) ? "1" : "0"],
    };
  });
}

function maximumOverlap(jobs) {
  const edges = jobs
    .flatMap(({ output }) => [
      { time: JSON.parse(fs.readFileSync(path.join(output, "start.json"))).time, change: 1 },
      { time: JSON.parse(fs.readFileSync(path.join(output, "end.json"))).time, change: -1 },
    ])
    .sort((a, b) => a.time - b.time || a.change - b.change);
  let active = 0;
  let maximum = 0;
  for (const { change } of edges) maximum = Math.max(maximum, (active += change));
  return maximum;
}

void test("real host subprocesses overlap, with at most two live workers", async (t) => {
  const jobs = subprocesses(t);
  const results = await runHostProcesses(jobs, { workers: 2 });
  requireSuccessfulHosts(results);
  assert.deepEqual(
    results.map(({ id }) => id),
    jobs.map(({ id }) => id),
  );
  assert.equal(maximumOverlap(jobs), 2);
  for (const { output, id } of jobs) {
    assert.equal(
      fs.readFileSync(path.join(output, "worker.stdout.log"), "utf8"),
      `started ${id}\n`,
    );
    assert.equal(
      fs.readFileSync(path.join(output, "worker.stderr.log"), "utf8"),
      `evidence ${id}\n`,
    );
  }
});

void test("serial benchmarking executes the same three subprocesses without overlap", async (t) => {
  const jobs = subprocesses(t);
  const results = await runHostProcesses(jobs, { workers: 1 });
  requireSuccessfulHosts(results);
  assert.equal(maximumOverlap(jobs), 1);
});

void test("both failed hosts and the queued host retain complete output before aggregate failure", async (t) => {
  const jobs = subprocesses(t, ["first", "second"]);
  const results = await runHostProcesses(jobs, { workers: 2 });
  assert.throws(
    () => requireSuccessfulHosts(results),
    (error) => {
      assert.ok(error instanceof AggregateError);
      assert.equal(error.errors.length, 2);
      return true;
    },
  );
  assert.deepEqual(
    results.map(({ status }) => status),
    [1, 1, 0],
  );
  for (const { output } of jobs) {
    assert.ok(fs.existsSync(path.join(output, "end.json")));
    assert.ok(fs.existsSync(path.join(output, "worker-process.json")));
    assert.ok(fs.readFileSync(path.join(output, "worker.stderr.log"), "utf8").length > 0);
  }
});

void test("timed out workers are killed and successful peers still finish", async (t) => {
  const jobs = subprocesses(t).slice(0, 2);
  jobs[0].args = ["-e", "setInterval(() => {}, 1000)"];
  jobs[0].timeoutMs = 40;
  const results = await runHostProcesses(jobs, { workers: 2 });
  assert.match(results[0].error, /timed out/u);
  assert.equal(results[0].signal, "SIGKILL");
  assert.equal(results[1].status, 0);
  assert.throws(() => process.kill(results[0].pid, 0), /ESRCH/u);
  assert.throws(() => requireSuccessfulHosts(results), AggregateError);
});

void test("worker timeout also kills its real subprocess descendants", async (t) => {
  const [job] = subprocesses(t);
  const late = path.join(path.dirname(job.output), "orphan-evidence.json");
  const descendant = `setTimeout(() => require('node:fs').writeFileSync(${JSON.stringify(late)}, 'orphan'), 350)`;
  job.args = [
    "-e",
    `require('node:child_process').spawn(process.execPath, ['-e', ${JSON.stringify(descendant)}]); setInterval(() => {}, 1000)`,
  ];
  job.timeoutMs = 150;
  const results = await runHostProcesses([job], { workers: 2 });
  assert.equal(results[0].signal, "SIGKILL");
  await new Promise((resolve) => setTimeout(resolve, 400));
  assert.equal(fs.existsSync(late), false, "a descendant survived its host worker timeout");
});

void test("spawn failures keep process evidence and do not stop the other host", async (t) => {
  const jobs = subprocesses(t).slice(0, 2);
  jobs[0].command = path.join(jobs[0].output, "missing-executable");
  const results = await runHostProcesses(jobs, { workers: 2 });
  assert.match(results[0].error, /ENOENT/u);
  assert.equal(results[1].status, 0);
  assert.ok(fs.existsSync(path.join(jobs[0].output, "worker-process.json")));
});

void test("strict options select reproducible modes and reject misspellings and duplicate flags", () => {
  assert.deepEqual(parseHostOptions([], {}), { mode: "concurrent", workers: 2 });
  assert.deepEqual(
    parseHostOptions(["--host-mode", "serial", "--phase-walltime", "/tmp/time.json"], {}),
    {
      mode: "serial",
      workers: 1,
      phaseWalltime: "/tmp/time.json",
    },
  );
  assert.deepEqual(parseHostOptions([], { VIZE_OXLINT_HOST_MODE: "serial" }), {
    mode: "serial",
    workers: 1,
  });
  assert.deepEqual(parseHostOptions(["--host-mode=serial"], {}), { mode: "serial", workers: 1 });
  assert.throws(() => parseHostOptions([], { VIZE_OXLINT_HOST_MODE: "parallel" }));
  assert.throws(() => parseHostOptions(["--host-mode", "serial", "--host-mode", "concurrent"], {}));
  assert.throws(() => parseHostOptions(["--host-mode"], {}));
  assert.throws(() => parseHostOptions(["--host-mode=serial=concurrent"], {}));
  assert.throws(() => parseHostOptions(["--workers", "3"], {}));
});

function segment(custody, phases, files = ["A.vue"]) {
  return {
    custody,
    phases,
    observed: {
      inventory: { files },
      counts: new Map([[phases[0], new Map([["A.vue", 1]])]]),
      calls: new Map([[phases[0], new Map([["A.vue", [{ kind: "call" }]]])]]),
      events: 2,
      processes: 1,
    },
  };
}

void test("isolated proof aggregation retains every call and rejects identity, corpus and phase collisions", () => {
  const custody = {
    source: { head: "same" },
    toolchain: { rust: "same" },
    binary: { path: "/same.node", sha256: "same" },
  };
  const baseline = segment(custody, ["frozenPerRule"]);
  const host = segment({ ...custody, calls: "/host.jsonl" }, ["host:1.78.0:baseline"]);
  const result = aggregateNativeEvidence(custody, baseline.observed.inventory, [baseline, host]);
  assert.deepEqual([...result.counts.keys()], ["frozenPerRule", "host:1.78.0:baseline"]);
  assert.equal(result.calls.get("host:1.78.0:baseline").get("A.vue").length, 1);
  assert.equal(result.events, 4);
  assert.equal(result.processes, 2);
  for (const changed of [
    { source: { head: "other" } },
    { binary: { path: "/other.node", sha256: "same" } },
    { binary: { path: "/same.node", sha256: "other" } },
    { toolchain: { rust: "other" } },
  ])
    assert.throws(() =>
      aggregateNativeEvidence(custody, baseline.observed.inventory, [
        baseline,
        { ...host, custody: { ...custody, ...changed } },
      ]),
    );
  assert.throws(() =>
    aggregateNativeEvidence(custody, baseline.observed.inventory, [
      baseline,
      segment(custody, ["host:1.78.0:baseline"], ["B.vue"]),
    ]),
  );
  assert.throws(
    () =>
      aggregateNativeEvidence(custody, baseline.observed.inventory, [
        baseline,
        segment(custody, ["frozenPerRule"]),
      ]),
    /duplicate native phase/u,
  );
});

void test("transport proofs still require all five guards, eighteen batch calls and authenticated exact bytes", () => {
  const source = { head: "source" };
  const custody = { binary: { path: "/source.node" } };
  const receipt = { source, frozen: { sha256: "binary-digest" } };
  const common = { source, binary: custody.binary.path, binarySha256: receipt.frozen.sha256 };
  const input = "<template>original</template>";
  const events = [
    ...Array.from({ length: 5 }, (_, index) => ({
      ...common,
      kind: "load",
      pid: index + 1,
      control: `Guard-${index}.vue`,
    })),
    { ...common, kind: "load", pid: 10 },
    ...Array.from({ length: 18 }, () => ({
      ...common,
      kind: "call",
      pid: 10,
      outcome: "return",
      hasRuleHint: true,
      args: [
        input,
        { filename: "/Original.vue", enabledRules: Array.from({ length: 51 }, () => "rule") },
      ],
      originalSha256: createHash("sha256").update(input).digest("hex"),
    })),
  ];
  assert.deepEqual(verifyProjectHostCalls(events, custody, receipt), {
    calls: 18,
    batchCalls: 18,
    processes: 6,
  });
  assert.throws(
    () => verifyProjectHostCalls(events.slice(1), custody, receipt),
    /all five guard hosts/u,
  );
  assert.throws(
    () => verifyProjectHostCalls(events.slice(0, -1), custody, receipt),
    /nine originals/u,
  );
  for (const change of [
    { source: { head: "different" } },
    { binarySha256: "different" },
    { originalSha256: "different" },
    { pid: 99 },
    { control: "Guard-0.vue" },
    { args: [input, { filename: "/Guard-0.vue" }] },
  ]) {
    const changed = [...events.slice(0, -1), { ...events.at(-1), ...change }];
    assert.throws(() => verifyProjectHostCalls(changed, custody, receipt));
  }
});

void test("failed source prerequisites emit sanitized failure timings before baseline or hosts run", (t) => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-ci-host-prerequisites-"));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const destination = path.join(root, "time.json");
  const script = fileURLToPath(new URL("./check-project-transport.mjs", import.meta.url));
  const result = spawnSync(
    process.execPath,
    [script, "--host-mode", "serial", "--phase-walltime", destination],
    {
      encoding: "utf8",
      env: { ...process.env, GITHUB_ACTIONS: "false", CI_TEST_PRIVATE_TOKEN: "must-never-appear" },
    },
  );
  assert.equal(result.status, 1);
  const bytes = fs.readFileSync(destination, "utf8");
  const timing = JSON.parse(bytes);
  assert.equal(timing.status, "failed");
  assert.equal(timing.mode, "serial");
  assert.equal(timing.workers, 1);
  assert.deepEqual(timing.hosts, []);
  assert.equal(timing.phaseWalltimeMs.baselinePreparation, undefined);
  assert.ok(timing.phaseWalltimeMs.total >= 0);
  assert.ok(!bytes.includes("must-never-appear"));
  assert.ok(!bytes.includes("--host-mode"));
});
