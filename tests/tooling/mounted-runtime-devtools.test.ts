import assert from "node:assert/strict";
import test from "node:test";
import {
  mountedRuntimeDevtools,
  withMountedRuntimeDevtools,
} from "./support/mounted-runtime-devtools.ts";

const key = "__VUE_DEVTOOLS_GLOBAL_HOOK__";

test("successful completed traces close once before releasing the observer", async () => {
  const target = {};
  const order: string[] = [];
  const result = await withMountedRuntimeDevtools(
    false,
    async () => {
      assert(Object.hasOwn(target, key));
      order.push("close");
    },
    async () => {
      order.push("trace");
      return 42;
    },
    { target, enabled: true },
  );
  assert.equal(result, 42);
  assert.deepEqual(order, ["trace", "close"]);
  assert.equal(Object.hasOwn(target, key), false);
});

test("completed trace failures close once and preserve even an undefined rejection", async () => {
  const target = {};
  let closed = 0;
  let rejected = false;
  await withMountedRuntimeDevtools(
    false,
    async () => {
      closed++;
      throw new Error("cleanup must not replace the trace failure");
    },
    async () => {
      await Promise.resolve();
      await Promise.reject(undefined);
    },
    { target, enabled: true },
  ).catch((error) => {
    rejected = true;
    assert.equal(error, undefined);
  });
  assert.equal(rejected, true);
  assert.equal(closed, 1);
  assert.equal(Object.hasOwn(target, key), false);
});

test("cleanup failures after a successful trace still fail and restore the hook", async () => {
  const target = {};
  const failure = new Error("DOM close failed");
  let closed = 0;
  await assert.rejects(
    withMountedRuntimeDevtools(
      false,
      async () => {
        closed++;
        throw failure;
      },
      async () => "completed",
      { target, enabled: true },
    ),
    (error) => error === failure,
  );
  assert.equal(closed, 1);
  assert.equal(Object.hasOwn(target, key), false);
});

test("repeated traces reattach each fresh observer to an already-created renderer", () => {
  const target = {};
  let attached: ReturnType<typeof mountedRuntimeDevtools>["hook"] | undefined;
  const runtime = {
    setDevtoolsHook(hook: NonNullable<typeof attached>, receiver: object) {
      assert.equal(receiver, target);
      attached = hook;
      hook.enabled = true;
    },
  };
  let previous: typeof attached;
  for (let trace = 0; trace < 2; trace++) {
    const observer = mountedRuntimeDevtools(false, { target, enabled: true });
    observer.attach(runtime);
    assert.equal(attached, observer.hook);
    assert.notEqual(attached, previous);
    const app = {};
    // An existing renderer emits through its module's current devtools pointer.
    attached!.emit("app:init", app, "3.6.0-rc.9", {});
    attached!.emit("app:unmount", app);
    observer.complete(app, [{ tree: [] }], []);
    observer.dispose();
    assert.equal(Object.hasOwn(target, key), false);
    previous = attached;
  }
  const refused = mountedRuntimeDevtools(false, { target, enabled: true });
  assert.throws(() => refused.attach({}), /attachment API/);
  refused.dispose();
});

test("failed async setup restores its hook even when DOM cleanup fails", async () => {
  const target = {};
  Object.defineProperty(target, key, { value: undefined, writable: false, configurable: true });
  const original = Object.getOwnPropertyDescriptor(target, key);
  const failure = new Error("runtime import or render setup failed");
  let closed = 0;
  await assert.rejects(
    withMountedRuntimeDevtools(
      false,
      async () => {
        closed++;
        throw new Error("DOM close failed");
      },
      async (observer) => {
        assert.equal(Reflect.get(target, key), observer.hook);
        await Promise.reject(failure);
      },
      { target, enabled: true },
    ),
    (error) => error === failure,
  );
  assert.equal(closed, 1);
  assert.deepEqual(Object.getOwnPropertyDescriptor(target, key), original);
});

test("the fresh observer retains hook events and releases the exact unmounted app", () => {
  const target = {};
  const observer = mountedRuntimeDevtools(false, { target, enabled: true });
  const hook = Reflect.get(target, key);
  const app = {};
  const payloads: unknown[][] = [];
  const listener = (...payload: unknown[]) => payloads.push(payload);
  hook.on("component:added", listener);
  hook.once("component:added", listener);
  hook.enabled = true; // Vue's setDevtoolsHook performs this attachment.
  hook.emit("app:init", app, "3.6.0-rc.9", { Text: Symbol("Text") });
  assert.equal(hook.appRecords[0].app, app);
  hook.emit("component:added", app, 1, undefined, { uid: 1 });
  hook.off("component:added", listener);
  hook.emit("component:added", app, 2);
  assert.equal(payloads.length, 2, "on/once/off preserve the supported listener contract");
  assert.equal(payloads[0][0], app);
  hook.emit("app:unmount", app);
  observer.complete(app, [{ tree: [] }], []);
  assert.equal(hook.appRecords.length, 0);
  observer.dispose();
  assert.equal(Object.hasOwn(target, key), false);
  const next = mountedRuntimeDevtools(false, { target, enabled: true });
  assert.notEqual(next.hook, hook, "a new trace cannot reuse module observer state");
  assert.equal(next.hook.appRecords.length, 0);
  next.dispose();
});

test("missing, duplicate and wrong-app lifecycle events fail complete trace evidence", () => {
  const observer = mountedRuntimeDevtools(false, { target: {}, enabled: true });
  const app = {};
  const hook = observer.hook;
  assert.throws(() => observer.complete(app, [], []), /Vue must attach/);
  hook.enabled = true;
  assert.throws(() => observer.complete(app, [], []), /initialization once/);
  hook.emit("app:init", app, "3.6.0-rc.9", {});
  hook.emit("app:unmount", {});
  assert.throws(() => observer.complete(app, [], []), /unmount once/);
  hook.emit("app:unmount", app);
  assert.throws(() => observer.complete(app, [], []), /exactly one app/);
  hook.emit("app:init", app, "3.6.0-rc.9", {});
  assert.throws(() => observer.complete(app, [], []), /initialization once/);
  observer.dispose();
  const valid = mountedRuntimeDevtools(false, { target: {}, enabled: true });
  valid.hook.enabled = true;
  valid.hook.emit("app:init", app, "3.6.0-rc.9", {});
  valid.hook.emit("app:unmount", app);
  assert.throws(() => valid.complete(app, [], ["runtime warning"]), /diagnostics/);
  valid.dispose();
});

test("production, explicit baseline and external hooks keep their existing environment", () => {
  for (const options of [
    { production: true, enabled: true },
    { production: false, enabled: false },
  ]) {
    const target = {};
    const observer = mountedRuntimeDevtools(options.production, {
      target,
      enabled: options.enabled,
    });
    assert.equal(observer.active, false);
    observer.complete({}, [], []);
    observer.dispose();
    assert.equal(Object.hasOwn(target, key), false);
  }
  const external = { emit() {} };
  const target = { [key]: external };
  const original = Object.getOwnPropertyDescriptor(target, key);
  const observer = mountedRuntimeDevtools(false, { target, enabled: true });
  assert.equal(observer.active, false);
  observer.dispose();
  assert.deepEqual(Object.getOwnPropertyDescriptor(target, key), original);
  const frozen = Object.freeze({ [key]: undefined });
  const refused = mountedRuntimeDevtools(false, { target: frozen, enabled: true });
  assert.equal(refused.active, false);
  refused.dispose();
});

test("cleanup restores an undefined descriptor and preserves a replacement owner", () => {
  const target = {};
  Object.defineProperty(target, key, { value: undefined, writable: false, configurable: true });
  const original = Object.getOwnPropertyDescriptor(target, key);
  const observer = mountedRuntimeDevtools(false, { target, enabled: true });
  observer.dispose();
  assert.deepEqual(Object.getOwnPropertyDescriptor(target, key), original);
  const replacement = {};
  const next = mountedRuntimeDevtools(false, { target, enabled: true });
  Object.defineProperty(target, key, { value: replacement, configurable: true });
  next.dispose();
  assert.equal(Reflect.get(target, key), replacement);
});
