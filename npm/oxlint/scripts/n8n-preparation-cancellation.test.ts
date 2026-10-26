import assert from "node:assert/strict";
import { spawn, spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import {
  preparationFixture,
  read,
  until,
} from "../../../tests/tooling/support/oxlint-preparation-fixture.mjs";

void test("real preparation cancellation joins baseline and both complete host process groups before exit", async (t) => {
  const f = preparationFixture(t);
  const url = (module: string) => JSON.stringify(new URL(module, import.meta.url).href);
  const waiting = path.join(f.artifacts, "waiting-child.mjs");
  fs.writeFileSync(
    waiting,
    `
import { spawn } from "node:child_process";
import fs from "node:fs";
const [root,id]=process.argv.slice(2);
const child=spawn(process.execPath,["-e","process.on('SIGTERM',()=>{});setInterval(()=>{},1000)"],{stdio:"ignore"});
fs.writeSync(1,"whole stdout "+id+"\\n");fs.writeSync(2,"whole stderr "+id+"\\n");
fs.writeFileSync(root+"/"+id+".ready.json",JSON.stringify({pid:process.pid,child:child.pid}));
setInterval(()=>{},1000);
`,
  );
  const driver = path.join(f.artifacts, "driver.mjs");
  fs.writeFileSync(
    driver,
    `
import assert from "node:assert/strict";
import fs from "node:fs";
import { captureProjectCommand } from ${url("./project-command-capture.mjs")};
import { runHostProcesses, requireSuccessfulHosts } from ${url("./ci-host-concurrency.mjs")};
import { runPreparationBranches } from ${url("./n8n-preparation-overlap.mjs")};
const f=${JSON.stringify(f)}, waiting=${JSON.stringify(waiting)};
try {
 await runPreparationBranches(f,{
  mode:"overlap",
  prepare:async()=>{
   const result=await captureProjectCommand(process.execPath,[waiting,f.artifacts,"baseline"],{encoding:"utf8",timeout:6000,maxBuffer:1024*1024});
   fs.writeFileSync(f.artifacts+"/baseline-process.json",JSON.stringify(result));
   assert.equal(result.status,0);return f.replay;
  },
  runHosts:async()=>{
   const results=await runHostProcesses(["first","second"].map(id=>({id,output:f.artifacts+"/"+id,command:process.execPath,args:[waiting,f.artifacts,id],timeoutMs:6000})),{workers:2});
   fs.writeFileSync(f.artifacts+"/host-results.json",JSON.stringify(results));requireSuccessfulHosts(results);
  },
 });
 process.exitCode=0;
} catch {fs.writeFileSync(f.artifacts+"/exit-after-join","joined");process.exitCode=1;}
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
  const ids = ["baseline", "first", "second"];
  await until(() => ids.every((id) => fs.existsSync(path.join(f.artifacts, id + ".ready.json"))));
  child.kill("SIGTERM");
  assert.deepEqual(await terminal, { status: 1, signal: null });
  const outcomes = read(path.join(f.artifacts, "n8n-preparation-outcomes.json"));
  assert.equal(outcomes.interrupted, "SIGTERM");
  assert.deepEqual(
    outcomes.branches.map(({ status }: { status: string }) => status),
    ["rejected", "rejected"],
  );
  assert.ok(fs.existsSync(path.join(f.artifacts, "exit-after-join")));
  assert.equal(read(path.join(f.artifacts, "n8n-preparation.json")).status, "failed");
  const baseline = read(path.join(f.artifacts, "baseline-process.json"));
  assert.equal(baseline.signal, "SIGTERM");
  assert.equal(baseline.error.code, "EINTR");
  assert.equal(baseline.stdout, "whole stdout baseline\n");
  assert.equal(baseline.stderr, "whole stderr baseline\n");
  for (const worker of read(path.join(f.artifacts, "host-results.json"))) {
    assert.equal(worker.signal, "SIGKILL");
    assert.equal(
      fs.readFileSync(path.join(f.artifacts, worker.id, "worker.stdout.log"), "utf8"),
      `whole stdout ${worker.id}\n`,
    );
    assert.equal(
      fs.readFileSync(path.join(f.artifacts, worker.id, "worker.stderr.log"), "utf8"),
      `whole stderr ${worker.id}\n`,
    );
  }
  for (const id of ids) {
    const { pid, child: descendant } = read(path.join(f.artifacts, id + ".ready.json"));
    for (const processId of [pid, descendant])
      await until(() => {
        const state = spawnSync("ps", ["-p", String(processId), "-o", "stat="], {
          encoding: "utf8",
        });
        assert.equal(state.error, undefined);
        return state.status === 1 || state.stdout.trim().startsWith("Z");
      });
  }
});
