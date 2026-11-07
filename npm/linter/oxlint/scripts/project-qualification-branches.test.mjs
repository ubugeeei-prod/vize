import assert from "node:assert/strict";
import { spawn, spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { runHostProcesses, requireSuccessfulHosts } from "./ci-host-concurrency.mjs";
import { runQualificationBranches } from "./project-qualification-branches.mjs";
import { captureProjectCommand } from "./project-command-capture.mjs";
import { runMixedWorkers } from "../src/test-support/mixed-directory-8507-workers.mjs";

const read = (file) => JSON.parse(fs.readFileSync(file));
const pause = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
async function until(predicate) {
  const deadline = performance.now() + 6000;
  while (!predicate()) {
    assert.ok(performance.now() < deadline, "actual process observation deadline");
    await pause(10);
  }
}

function fixture(t, mode) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-project-branches-"));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  fs.mkdirSync(path.join(root, "src"));
  const preload = path.join(root, "preload.cjs");
  fs.writeFileSync(
    preload,
    "// No native substitute: this fixture exercises real process ownership.\n",
  );
  const environment = { ...process.env, FIXTURE_ROOT: root, FIXTURE_MODE: mode };
  fs.writeFileSync(
    path.join(root, "src/mixed-directory-8507.test.mjs"),
    `import fs from "node:fs";
import path from "node:path";
import { spawn } from "node:child_process";
const root = process.env.FIXTURE_ROOT, index = process.env.VIZE_OXLINT_MIXED_WORKER;
const mode = process.env.FIXTURE_MODE;
const child = spawn(process.execPath, ["-e", "process.on('SIGTERM',()=>{});setInterval(()=>{},1000)"], {stdio:"ignore"});
fs.writeSync(1,"whole stdout " + index + "\\n");
fs.writeSync(2,"whole stderr " + index + "\\n");
fs.writeFileSync(path.join(root, index + ".ready.json"), JSON.stringify({pid:process.pid,child:child.pid}));
if (mode === "timeout") process.on("SIGTERM",()=>{});
if (mode === "deadline-prefix" || mode === "deadline-history")
 setTimeout(()=>fs.writeFileSync(path.join(root,index+".after-deadline"),"late"),800);
const timer = setInterval(() => {
  if (!fs.existsSync(path.join(root, "remaining.started"))) return;
  if (mode === "success" || (mode === "closed-leader-red" && index === "0")) { clearInterval(timer); process.exit(0); }
  if (mode === "red" && index === "0") process.exit(7);
  if (mode === "closed-leader-red" && index === "1") {clearInterval(timer);setTimeout(()=>process.exit(7),200);}
  if (mode === "child-cancel" && index === "0") process.kill(process.pid,"SIGTERM");
}, 10);
`,
  );
  const remainingScript = path.join(root, "remaining.mjs");
  fs.writeFileSync(
    remainingScript,
    `import fs from "node:fs";
import path from "node:path";
const root = process.env.FIXTURE_ROOT;
process.stdout.write("complete remaining stdout\\n");
process.stderr.write("complete remaining stderr\\n");
const timer = setInterval(() => {
  if (!["0.ready.json","1.ready.json"].every(name=>fs.existsSync(path.join(root,name)))) return;
  clearInterval(timer);
  fs.writeFileSync(path.join(root,"remaining.started"),"started");
  if (process.env.FIXTURE_MODE === "parent-cancel") setInterval(()=>{},1000);
  else setTimeout(()=>{fs.writeFileSync(path.join(root,"remaining.done"),"complete");},150);
},10);
`,
  );
  const output = path.join(root, "mixed");
  fs.mkdirSync(output);
  const mixedOptions = {
    packageDir: root,
    output,
    custody: { calls: path.join(output, "source-native-calls.jsonl") },
    preload,
    environment,
    ...(mode === "timeout" ? { deadlineMs: 1000 } : {}),
    ...(mode.startsWith("deadline-") ? { deadlineMs: 500 } : {}),
  };
  const remaining = {
    id: "remaining",
    output: path.join(root, "remaining"),
    command: process.execPath,
    args: [remainingScript],
    cwd: root,
    env: environment,
    timeoutMs: 6000,
  };
  const outcomes = path.join(root, "branch-outcomes.json");
  return { root, output, mixedOptions, remaining, outcomes };
}

async function run(f) {
  return runQualificationBranches(
    [
      { id: "mixedDirectory8507", run: () => runMixedWorkers(f.mixedOptions) },
      {
        id: "n8n",
        run: async () => {
          const results = await runHostProcesses([f.remaining], { workers: 1 });
          requireSuccessfulHosts(results);
          return results;
        },
      },
    ],
    f.outcomes,
  );
}

function assertRaw(f) {
  const packet = read(path.join(f.output, "source-process.json"));
  assert.equal(packet.workers.length, 2);
  for (const worker of packet.workers) {
    assert.equal(Buffer.from(worker.stdoutBytes).toString(), `whole stdout ${worker.index}\n`);
    assert.equal(Buffer.from(worker.stderrBytes).toString(), `whole stderr ${worker.index}\n`);
    assert.equal(
      read(path.join(f.output, `worker-${worker.index}/source-process.json`)).pid,
      worker.pid,
    );
  }
  assert.equal(
    fs.readFileSync(path.join(f.remaining.output, "worker.stdout.log"), "utf8"),
    "complete remaining stdout\n",
  );
  assert.equal(
    fs.readFileSync(path.join(f.remaining.output, "worker.stderr.log"), "utf8"),
    "complete remaining stderr\n",
  );
  assert.ok(fs.existsSync(path.join(f.remaining.output, "worker-process.json")));
  return packet;
}

async function assertNoActiveDescendants(f) {
  for (const file of fs.readdirSync(f.root).filter((name) => name.endsWith(".ready.json"))) {
    const { pid, child } = read(path.join(f.root, file));
    for (const processId of [pid, child])
      await until(() => {
        const state = spawnSync("ps", ["-p", String(processId), "-o", "stat="], {
          encoding: "utf8",
        });
        assert.equal(state.error, undefined);
        return state.status === 1 || state.stdout.trim().startsWith("Z");
      });
  }
}

void test("independent real workers overlap and both complete raw outcomes precede acceptance", async (t) => {
  const f = fixture(t, "success");
  const [mixed, remaining] = await run(f);
  assert.equal(mixed.length, 2);
  assert.equal(remaining[0].status, 0);
  assert.ok(fs.existsSync(path.join(f.root, "remaining.done")));
  assert.deepEqual(
    read(f.outcomes).branches.map(({ id, status }) => [id, status]),
    [
      ["mixedDirectory8507", "fulfilled"],
      ["n8n", "fulfilled"],
    ],
  );
  const packet = assertRaw(f);
  assert.equal(packet.deadlineMs, 180_000);
  assert.equal(packet.maxBufferBytes, 64 * 1024 * 1024);
  await assertNoActiveDescendants(f);
});

for (const mode of ["red", "closed-leader-red", "child-cancel", "timeout"])
  void test(`actual ${mode} refuses only after the remaining branch and full raw packets settle`, async (t) => {
    const f = fixture(t, mode);
    await assert.rejects(run(f), AggregateError);
    assert.ok(fs.existsSync(path.join(f.root, "remaining.done")));
    assert.deepEqual(
      read(f.outcomes).branches.map(({ status }) => status),
      ["rejected", "fulfilled"],
    );
    const packet = assertRaw(f);
    assert.ok(packet.failure);
    if (mode === "red") assert.equal(packet.workers[0].status, 7);
    if (mode === "closed-leader-red") {
      assert.equal(packet.workers[0].status, 0);
      assert.equal(packet.workers[1].status, 7);
    }
    if (mode === "child-cancel") assert.equal(packet.workers[0].signal, "SIGTERM");
    if (mode === "timeout") {
      assert.equal(packet.deadlineMs, 1000);
      assert.ok(packet.workers.every(({ signal }) => signal === "SIGKILL"));
    }
    await assertNoActiveDescendants(f);
  });

void test("real parent cancellation joins both groups and preserves evidence before exiting", async (t) => {
  const f = fixture(t, "parent-cancel");
  const driver = path.join(f.root, "driver.mjs");
  const moduleUrl = (name) => JSON.stringify(new URL(name, import.meta.url).href);
  fs.writeFileSync(
    driver,
    `
import fs from "node:fs";
import { runQualificationBranches } from ${moduleUrl("./project-qualification-branches.mjs")};
import { runHostProcesses, requireSuccessfulHosts } from ${moduleUrl("./ci-host-concurrency.mjs")};
import { runMixedWorkers } from ${moduleUrl("../src/test-support/mixed-directory-8507-workers.mjs")};
const f = ${JSON.stringify(f)};
try {
 await runQualificationBranches([
  {id:"mixedDirectory8507",run:()=>runMixedWorkers(f.mixedOptions)},
  {id:"n8n",run:async()=>{const r=await runHostProcesses([f.remaining],{workers:1});requireSuccessfulHosts(r);return r;}},
 ],f.outcomes);
 process.exitCode=0;
} catch {fs.writeFileSync(f.root+"/exit-after-join","joined");process.exitCode=1;}
`,
  );
  const child = spawn(process.execPath, [driver], { stdio: "ignore" });
  t.after(() => {
    if (child.exitCode === null) child.kill("SIGKILL");
  });
  const terminal = new Promise((resolve, reject) => {
    child.once("error", reject);
    child.once("close", (status, signal) => resolve({ status, signal }));
  });
  await until(
    () =>
      fs.existsSync(path.join(f.root, "remaining.started")) &&
      fs.existsSync(path.join(f.root, "1.ready.json")),
  );
  child.kill("SIGTERM");
  assert.deepEqual(await terminal, { status: 1, signal: null });
  const outcomes = read(f.outcomes);
  assert.equal(outcomes.interrupted, "SIGTERM");
  assert.deepEqual(
    outcomes.branches.map(({ status }) => status),
    ["rejected", "rejected"],
  );
  assert.ok(fs.existsSync(path.join(f.root, "exit-after-join")));
  assertRaw(f);
  await assertNoActiveDescendants(f);
});

void test("fixed worker membership and production budget cannot be widened", async () => {
  await assert.rejects(runQualificationBranches([], "unused"));
  await assert.rejects(runQualificationBranches([{ id: "same" }, { id: "same" }], "unused"));
  await assert.rejects(runMixedWorkers({ deadlineMs: 180_001 }));
});

for (const mode of ["deadline-prefix", "deadline-history"])
  void test(`${mode}: original500ms budget survives an adjacent real1500ms command wait`, async (t) => {
    const f = fixture(t, mode);
    let remaining;
    const delayedCommand = async () => {
      remaining = await captureProjectCommand(process.execPath, ["-e", "setTimeout(()=>{},1500)"], {
        timeout: 3000,
        maxBuffer: 1024,
      });
      assert.equal(remaining.error, undefined);
      assert.equal(remaining.status, 0);
    };
    const branches = [
      { id: "mixedDirectory8507", run: () => runMixedWorkers(f.mixedOptions) },
      { id: "n8n", run: delayedCommand },
    ];
    if (mode === "deadline-history") branches.reverse();
    await assert.rejects(runQualificationBranches(branches, f.outcomes), AggregateError);
    const packet = read(path.join(f.output, "source-process.json"));
    assert.equal(packet.deadlineMs, 500);
    assert.ok(packet.walltimeMs < 1000, `deadline must not wait for sibling: ${packet.walltimeMs}`);
    assert.ok(!fs.readdirSync(f.root).some((name) => name.endsWith(".after-deadline")));
    assert.equal(read(f.outcomes).branches.find(({ id }) => id === "n8n").status, "fulfilled");
    await assertNoActiveDescendants(f);
  });
