import {
  readToolingProgram,
  toolingProgram,
} from "../../../../tests/tooling/support/program-path.ts";
import assert from "node:assert/strict";
import { spawn, spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import {
  preparationFixture,
  read,
  until,
} from "../../../../tests/tooling/support/oxlint-preparation-fixture.mjs";

void test("real preparation cancellation joins baseline and both complete host process groups before exit", async (t) => {
  const f = preparationFixture(t);
  const url = (module: string) => JSON.stringify(new URL(module, import.meta.url).href);
  const waiting = path.join(f.artifacts, "waiting-child.mjs");
  fs.writeFileSync(
    waiting,
    readToolingProgram(
      new URL(
        "./test-programs/n8n-preparation-cancellation/wait-for-cancellation.js.in",
        import.meta.url,
      ),
      {
        CHILD_PROGRAM: JSON.stringify(toolingProgram("process/ignore-termination.js")),
      },
    ),
  );
  const driver = path.join(f.artifacts, "driver.mjs");
  fs.writeFileSync(
    driver,
    readToolingProgram(
      new URL(
        "./test-programs/n8n-preparation-cancellation/cancel-preparation.js.in",
        import.meta.url,
      ),
      {
        COMMAND_CAPTURE: url("./project-command-capture.mjs"),
        HOST_CONCURRENCY: url("./ci-host-concurrency.mjs"),
        OVERLAP: url("./n8n-preparation-overlap.mjs"),
        PROJECT: JSON.stringify(f),
        WAITING: JSON.stringify(waiting),
      },
    ),
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
