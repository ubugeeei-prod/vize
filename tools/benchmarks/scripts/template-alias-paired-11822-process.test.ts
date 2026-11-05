import assert from "node:assert/strict";
import { EventEmitter } from "node:events";
import type { ChildProcess, spawn } from "node:child_process";
import { test } from "node:test";
import { measuredProcess } from "./template-alias-paired-11822-process.ts";

function fixture() {
  const emitter = new EventEmitter();
  let unreferenced = false;
  const child = Object.assign(emitter, {
    pid: 123,
    unref: () => {
      unreferenced = true;
    },
    kill: () => true,
  }) as unknown as ChildProcess;
  const groups: number[] = [];
  const options = {
    command: "finite-fake-command",
    args: [],
    cwd: "/",
    environment: {},
    stdout: 1,
    stderr: 2,
    timePath: "/unused",
    timeoutMilliseconds: 5,
    closeWaitMilliseconds: 5,
    launch: (() => child) as typeof spawn,
    readStart: () => ({ pid: 123, group: 123, startTicks: "10" }),
    signalGroup: (group: number) => {
      groups.push(group);
    },
  };
  return { emitter, options, groups, unreferenced: () => unreferenced };
}

void test("missing close ends finitely with unknown status and retains timeout identity", async () => {
  const state = fixture();
  const result = await measuredProcess(state.options);
  assert.equal(result.closeObserved, false);
  assert.equal(result.status, null);
  assert.equal(result.signal, null);
  assert.equal(result.timedOut, true);
  assert.ok(state.groups.length > 0 && state.groups.every((group) => group === -123));
  assert.equal(state.unreferenced(), true);
});

void test("PID reuse or vanished group leaders cannot receive a group signal", async () => {
  for (const replacement of ["11", "gone"]) {
    const state = fixture();
    let reads = 0;
    state.options.readStart = () => {
      if (++reads > 1 && replacement === "gone")
        throw Object.assign(new Error("vanished"), { code: "ESRCH" });
      return { pid: 123, group: 123, startTicks: reads > 1 ? replacement : "10" };
    };
    const result = await measuredProcess(state.options);
    assert.deepEqual(state.groups, []);
    assert.equal(result.closeObserved, false);
    assert.equal(result.error, undefined);
  }
});

void test("an actual close packet alone supplies status and clears deadline signals", async () => {
  const state = fixture();
  queueMicrotask(() => state.emitter.emit("close", 1, null));
  const result = await measuredProcess(state.options);
  assert.equal(result.status, 1);
  assert.equal(result.closeObserved, true);
  assert.equal(result.timedOut, false);
  assert.deepEqual(state.groups, []);
});
