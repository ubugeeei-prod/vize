import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { captureProjectCommand } from "./project-command-capture.mjs";
import { requireSuccessfulHosts, runHostProcesses } from "./ci-host-concurrency.mjs";
import {
  parsePreparationOptions,
  publishReplayReady,
  replayReadyPath,
  runPreparationBranches,
  waitForReplayReady,
} from "./n8n-preparation-overlap.mjs";
import {
  hash,
  preparationFixture,
  read,
  until,
} from "../../../tests/tooling/support/oxlint-preparation-fixture.mjs";

function campaign(t, mode, failure, timeoutMs = 6000) {
  const f = preparationFixture(t);
  const module = new URL("./n8n-preparation-overlap.mjs", import.meta.url).href;
  const worker = path.join(f.artifacts, "controlled-host.mjs");
  const config = path.join(f.artifacts, "context.json");
  fs.writeFileSync(config, JSON.stringify(f));
  fs.writeFileSync(
    worker,
    `
import fs from "node:fs";
import path from "node:path";
import { waitForReplayReady } from ${JSON.stringify(module)};
const [file,id,failure]=process.argv.slice(2), f=JSON.parse(fs.readFileSync(file));
for (const phase of ["install","transport","htmlTransport"])
 fs.writeFileSync(path.join(f.artifacts,id+"."+phase),phase+"\\r\\n");
console.log("whole host stdout " + id);
console.error("whole host stderr " + id);
if (failure==="host" && id==="first") process.exit(7);
try {
 await waitForReplayReady(f);
 if (!fs.existsSync(path.join(f.artifacts,"baseline.done"))) throw Error("early replay");
 fs.writeFileSync(path.join(f.artifacts,id+".replay"),"complete original replay");
} catch {process.exitCode=1;}
fs.writeFileSync(path.join(f.artifacts,id+".done"),"terminal evidence");
`,
  );
  const jobs = ["first", "second"].map((id) => ({
    id,
    output: path.join(f.artifacts, id),
    command: process.execPath,
    args: [worker, config, id, failure ?? "none"],
    timeoutMs,
  }));
  const baseline = `
const fs=require("node:fs"),root=${JSON.stringify(f.artifacts)};
fs.writeFileSync(root+"/baseline.started","started");
console.log("whole baseline stdout");console.error("whole baseline stderr");
const timer=setInterval(()=>{
 if(!fs.existsSync(root+"/baseline.release"))return;
 clearInterval(timer);fs.writeFileSync(root+"/baseline.done","joined vectors and reference");
 process.exit(${failure === "baseline" ? 7 : 0});
},10);
`;
  const pending = runPreparationBranches(f, {
    mode,
    prepare: async () => {
      const result = await captureProjectCommand(process.execPath, ["-e", baseline], {
        timeout: 6000,
        maxBuffer: 1024 * 1024,
        encoding: "utf8",
      });
      fs.writeFileSync(path.join(f.artifacts, "baseline-process.json"), JSON.stringify(result));
      assert.equal(result.status, 0);
      return f.replay;
    },
    runHosts: async () => {
      const results = await runHostProcesses(jobs, { workers: 2 });
      fs.writeFileSync(path.join(f.artifacts, "host-results.json"), JSON.stringify(results));
      requireSuccessfulHosts(results);
      return results;
    },
  });
  // Observe rejection immediately while the causal release controls run.
  const outcome = pending.then(
    (value) => ({ value }),
    (error) => ({ error }),
  );
  return { ...f, jobs, outcome };
}

for (const mode of ["overlap", "sequential"])
  void test(`${mode} retains both real host preparations and gates all replays on joined baseline`, async (t) => {
    const f = campaign(t, mode);
    await until(() => fs.existsSync(path.join(f.artifacts, "baseline.started")));
    if (mode === "overlap") {
      await until(() =>
        f.jobs.every(({ id }) => fs.existsSync(path.join(f.artifacts, id + ".htmlTransport"))),
      );
      assert.ok(!fs.existsSync(replayReadyPath(f.artifacts)));
      assert.ok(f.jobs.every(({ id }) => !fs.existsSync(path.join(f.artifacts, id + ".replay"))));
    } else
      assert.ok(f.jobs.every(({ id }) => !fs.existsSync(path.join(f.artifacts, id + ".install"))));
    fs.writeFileSync(path.join(f.artifacts, "baseline.release"), "release original baseline");
    const result = await f.outcome;
    assert.equal(result.error, undefined);
    for (const { id, output } of f.jobs) {
      assert.ok(fs.existsSync(path.join(f.artifacts, id + ".replay")));
      assert.equal(
        fs.readFileSync(path.join(output, "worker.stdout.log"), "utf8"),
        `whole host stdout ${id}\n`,
      );
      assert.equal(
        fs.readFileSync(path.join(output, "worker.stderr.log"), "utf8"),
        `whole host stderr ${id}\n`,
      );
      assert.equal(read(path.join(output, "worker-process.json")).status, 0);
    }
    assert.deepEqual(
      read(path.join(f.artifacts, "n8n-preparation-outcomes.json")).branches.map(
        ({ status }) => status,
      ),
      ["fulfilled", "fulfilled"],
    );
  });

for (const failure of ["baseline", "host"])
  void test(`actual ${failure} failure joins every worker and retains raw outcomes before refusal`, async (t) => {
    const f = campaign(t, "overlap", failure);
    await until(() =>
      f.jobs.every(({ id }) => fs.existsSync(path.join(f.artifacts, id + ".htmlTransport"))),
    );
    fs.writeFileSync(path.join(f.artifacts, "baseline.release"), "release original baseline");
    const result = await f.outcome;
    assert.ok(result.error instanceof AggregateError);
    const results = read(path.join(f.artifacts, "host-results.json"));
    assert.equal(results.length, 2);
    const outcomes = read(path.join(f.artifacts, "n8n-preparation-outcomes.json"));
    assert.deepEqual(
      outcomes.branches.map(({ status }) => status),
      failure === "baseline" ? ["rejected", "rejected"] : ["fulfilled", "rejected"],
    );
    assert.equal(
      read(path.join(f.artifacts, "baseline-process.json")).stdout,
      "whole baseline stdout\n",
    );
    for (const { id, output } of f.jobs) {
      assert.ok(fs.existsSync(path.join(output, "worker-process.json")));
      assert.equal(
        fs.readFileSync(path.join(output, "worker.stdout.log"), "utf8"),
        `whole host stdout ${id}\n`,
      );
      if (failure === "baseline") {
        const result = results.find((row) => row.id === id);
        assert.equal(result.status, 1);
        assert.equal(result.signal, null);
        assert.equal(
          result.error,
          undefined,
          "failed preparation must release waiters before their deadline",
        );
        assert.ok(fs.existsSync(path.join(f.artifacts, id + ".done")));
        assert.ok(!fs.existsSync(path.join(f.artifacts, id + ".replay")));
      }
    }
    if (failure === "host") assert.ok(fs.existsSync(path.join(f.artifacts, "second.replay")));
  });

void test("the original host deadline still kills workers waiting for readiness while baseline is joined", async (t) => {
  const f = campaign(t, "overlap", undefined, 500);
  await until(() =>
    f.jobs.every(({ output }) => fs.existsSync(path.join(output, "worker-process.json"))),
  );
  assert.ok(!fs.existsSync(replayReadyPath(f.artifacts)));
  assert.ok(!fs.existsSync(path.join(f.artifacts, "baseline.done")));
  fs.writeFileSync(
    path.join(f.artifacts, "baseline.release"),
    "release after original host deadline",
  );
  assert.ok((await f.outcome).error instanceof AggregateError);
  assert.equal(read(path.join(f.artifacts, "baseline-process.json")).status, 0);
  for (const result of read(path.join(f.artifacts, "host-results.json"))) {
    assert.equal(result.signal, "SIGKILL");
    assert.match(result.error, /timed out after 500 ms/u);
    assert.ok(!fs.existsSync(path.join(f.artifacts, result.id + ".replay")));
  }
});

const packetChanges = [
  (packet) => {
    packet.identity.source.head = "c".repeat(40);
  },
  (packet) => {
    packet.identity.source.tree = "c".repeat(40);
  },
  (packet) => {
    packet.identity.source.workingDiff = "changed";
  },
  (packet) => {
    packet.identity.toolchain.rustc = "stale compiler";
  },
  (packet) => {
    packet.identity.binary.sha256 = "stale binary";
  },
  (packet) => {
    packet.replay.reference.source.tree = "c".repeat(40);
  },
  (packet) => {
    packet.replay.reference.plugin += ".other";
  },
  (packet) => {
    packet.replay.reference.bundles.pop();
  },
  (packet) => {
    packet.replay.reference.physicalSource[0].blob = "stale source member";
  },
  (packet) => {
    packet.replay.reference.wrapper += ".other";
  },
  (packet) => {
    packet.replay.custody.sources += ".other";
  },
  (packet) => {
    packet.evidence.pop();
  },
];
void test("all stale identity and incomplete reference packets refuse before replay", async (t) => {
  for (const change of packetChanges) {
    const f = preparationFixture(t);
    publishReplayReady(f, f.replay);
    const file = replayReadyPath(f.artifacts),
      packet = read(file);
    change(packet);
    // Reference and custody assertions still refuse a complete but stale
    // descriptor, independently of the accidental-change checksum.
    packet.descriptorSha256 = hash(JSON.stringify(packet.replay));
    fs.writeFileSync(file, JSON.stringify(packet));
    await assert.rejects(waitForReplayReady(f));
  }
});

void test("changing any whole replay descriptor member refuses even when unused by a host stage", async (t) => {
  const f = preparationFixture(t);
  publishReplayReady(f, f.replay);
  const packet = read(replayReadyPath(f.artifacts));
  packet.replay.inventory.controlled = "changed original descriptor";
  fs.writeFileSync(replayReadyPath(f.artifacts), JSON.stringify(packet));
  await assert.rejects(waitForReplayReady(f), /replay descriptor changed/u);
});

void test("changed joined vectors, ledger, runtime/copied bundles and physical reference sources refuse", async (t) => {
  for (const select of [
    (f) => path.join(f.replay.output, "bridge/bridge-vectors.jsonl"),
    (f) => f.replay.custody.calls,
    (f) => path.join(f.replay.output, "reference-source.json"),
    (f) => f.replay.reference.plugin,
    (f) => path.join(f.replay.output, "reference-bundles/dist/index.mjs"),
    (f) => path.join(f.replay.reference.build.cwd, "source.mjs"),
    (f) => f.binary,
  ]) {
    const f = preparationFixture(t);
    publishReplayReady(f, f.replay);
    fs.appendFileSync(select(f), "changed bytes\n");
    await assert.rejects(waitForReplayReady(f));
  }
});

void test("a complete authenticated packet grants only the unchanged descriptor and cannot be republished", async (t) => {
  const f = preparationFixture(t);
  publishReplayReady(f, f.replay);
  assert.deepEqual(await waitForReplayReady(f), f.replay);
  assert.throws(() => publishReplayReady(f, f.replay), /already published/u);
});

void test("separate preparation selection preserves every original host option and refuses malformed modes", () => {
  assert.deepEqual(
    parsePreparationOptions(
      [
        "--host-mode",
        "serial",
        "--preparation-mode",
        "sequential",
        "--phase-walltime",
        "/tmp/time",
      ],
      {},
    ),
    {
      mode: "sequential",
      args: ["--host-mode", "serial", "--phase-walltime", "/tmp/time"],
    },
  );
  assert.equal(parsePreparationOptions([], {}).mode, "overlap");
  assert.equal(
    parsePreparationOptions([], { VIZE_OXLINT_PREPARATION_MODE: "sequential" }).mode,
    "sequential",
  );
  for (const args of [
    ["--preparation-mode"],
    ["--preparation-mode="],
    ["--preparation-mode", "wide"],
    ["--preparation-mode=overlap", "--preparation-mode=sequential"],
  ])
    assert.throws(() => parsePreparationOptions(args, {}));
});
