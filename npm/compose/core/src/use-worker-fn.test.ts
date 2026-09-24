import assert from "node:assert/strict";
import { test } from "node:test";
import { runInNewContext } from "node:vm";
import { effectScope } from "vue";

import { renderComposableOnServer } from "./testing/ssr-harness.ts";
import type { TimeoutScheduler } from "./timeout-scheduler.ts";
import { createWorkerFnScript, useWorkerFn } from "./use-worker-fn.ts";
import type { WorkerFnHost, WorkerLike } from "./use-worker-fn.ts";

class FakeWorker extends EventTarget implements WorkerLike {
  static readonly instances: FakeWorker[] = [];
  readonly url: string;
  readonly posted: unknown[] = [];
  terminated = false;

  constructor(url: string) {
    super();
    this.url = url;
    FakeWorker.instances.push(this);
  }

  postMessage(message: unknown): void {
    this.posted.push(message);
  }

  terminate(): void {
    this.terminated = true;
  }

  reply(kind: "success" | "error", value: unknown): void {
    this.dispatchEvent(new MessageEvent("message", { data: [kind, value] }));
  }
}

function latest(): FakeWorker {
  const worker = FakeWorker.instances.at(-1);
  assert.ok(worker);
  return worker;
}

function createHost(): WorkerFnHost & {
  readonly scripts: Map<string, Blob>;
  readonly revoked: string[];
} {
  const scripts = new Map<string, Blob>();
  const revoked: string[] = [];
  return {
    scripts,
    revoked,
    Worker: FakeWorker,
    createObjectURL: (script) => {
      const url = `blob:${String(scripts.size + 1)}`;
      scripts.set(url, script);
      return url;
    },
    revokeObjectURL: (url) => revoked.push(url),
  };
}

function createScheduler(): TimeoutScheduler & { readonly run: () => void } {
  const timers = new Map<number, () => void>();
  let next = 0;
  return {
    setTimeout: (callback) => {
      next += 1;
      timers.set(next, callback);
      return next;
    },
    clearTimeout: (handle) => {
      if (typeof handle === "number") timers.delete(handle);
    },
    run: () => {
      const callbacks = [...timers.values()];
      timers.clear();
      for (const callback of callbacks) callback();
    },
  };
}

function double(value: number): number {
  return value * 2;
}

void test("the generated script runs the function and posts its awaited result", async () => {
  const script = createWorkerFnScript(async (value: number) => double(value) + 1, {
    localDependencies: [double],
  });
  const posted: unknown[] = [];
  const self: { onmessage?: (event: { readonly data: unknown }) => Promise<void> } = {};
  runInNewContext(script, {
    self: Object.assign(self, { postMessage: (message: unknown) => posted.push(message) }),
  });
  assert.ok(self.onmessage);
  await self.onmessage({ data: [20] });
  assert.equal(JSON.stringify(posted), `[["success",41]]`);

  const failing = createWorkerFnScript(() => {
    throw new Error("boom");
  });
  const failingSelf: { onmessage?: (event: { readonly data: unknown }) => Promise<void> } = {};
  const failures: unknown[] = [];
  runInNewContext(failing, {
    self: Object.assign(failingSelf, { postMessage: (message: unknown) => failures.push(message) }),
  });
  await failingSelf.onmessage?.({ data: [] });
  assert.equal(JSON.stringify(failures), `[["error","boom"]]`);
});

void test("the generated script imports dependencies and rejects anonymous helpers", () => {
  const script = createWorkerFnScript(() => 1, { dependencies: ["https://cdn.test/a.js"] });
  assert.match(script, /^importScripts\("https:\/\/cdn\.test\/a\.js"\);/);
  assert.throws(
    () =>
      createWorkerFnScript(() => 1, {
        localDependencies: [
          (
            () => () =>
              1
          )(),
        ],
      }),
    /VIZE_COMPOSE_WORKER_FN_ANONYMOUS_DEPENDENCY/,
  );
});

void test("runs in a Blob-URL worker and cleans up after success", async () => {
  const host = createHost();
  const sum = useWorkerFn((left: number, right: number) => left + right, { host });
  assert.equal(sum.supported.value, true);

  const result = sum.run(1, 2);
  const worker = latest();
  assert.equal(sum.status.value, "running");
  assert.deepEqual(worker.posted, [[1, 2]]);
  const source = await host.scripts.get(worker.url)?.text();
  assert.match(source ?? "", /left \+ right/);

  worker.reply("success", 3);
  assert.equal(await result, 3);
  assert.equal(sum.status.value, "success");
  assert.equal(worker.terminated, true);
  assert.deepEqual(host.revoked, [worker.url]);
});

void test("rejects with tagged errors for failures, busy runs, and timeouts", async () => {
  const host = createHost();
  const scheduler = createScheduler();
  const task = useWorkerFn((value: string) => value, { host, scheduler, timeoutMs: 100 });

  const failing = task.run("x");
  await assert.rejects(task.run("y"), /VIZE_COMPOSE_WORKER_FN_BUSY/);
  latest().reply("error", "boom");
  await assert.rejects(failing, /VIZE_COMPOSE_WORKER_FN_FAILED\] boom/);
  assert.equal(task.status.value, "error");

  const slow = task.run("z");
  const worker = latest();
  scheduler.run();
  await assert.rejects(slow, /VIZE_COMPOSE_WORKER_FN_TIMEOUT/);
  assert.equal(task.status.value, "timeout");
  assert.equal(worker.terminated, true);

  const crashed = task.run("w");
  latest().dispatchEvent(new Event("error"));
  await assert.rejects(crashed, /VIZE_COMPOSE_WORKER_FN_FAILED/);
});

void test("terminate and scope disposal reject the pending run", async () => {
  const host = createHost();
  const scope = effectScope();
  const task = scope.run(() => useWorkerFn(() => 1, { host }));
  assert.ok(task);

  const first = task.run();
  task.terminate();
  await assert.rejects(first, /VIZE_COMPOSE_WORKER_FN_TERMINATED/);
  assert.equal(task.status.value, "idle");

  const second = task.run();
  const worker = latest();
  scope.stop();
  await assert.rejects(second, /VIZE_COMPOSE_WORKER_FN_TERMINATED/);
  assert.equal(worker.terminated, true);
});

void test("rejects without worker support and validates the timeout", async () => {
  const task = useWorkerFn(() => 1, { host: null });
  assert.equal(task.supported.value, false);
  await assert.rejects(task.run(), /VIZE_COMPOSE_WORKER_FN_UNSUPPORTED/);
  assert.throws(() => useWorkerFn(() => 1, { timeoutMs: Number.NaN }), RangeError);
});

void test("server rendering spawns no worker", async () => {
  const state = await renderComposableOnServer(() => {
    const task = useWorkerFn((value: number) => value * 2);
    return { status: task.status, supported: task.supported };
  });
  assert.equal(state, '{"status":"idle","supported":false}');
});
