import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope } from "vue";

import { renderComposableOnServer } from "./testing/ssr-harness.ts";
import { useServiceWorker } from "./use-service-worker.ts";
import type {
  ServiceWorkerContainerLike,
  ServiceWorkerLike,
  ServiceWorkerRegisterOptions,
  ServiceWorkerRegistrationLike,
  ServiceWorkerStateName,
} from "./use-service-worker.ts";

class FakeWorker extends EventTarget implements ServiceWorkerLike {
  readonly messages: unknown[] = [];
  state: ServiceWorkerStateName;

  constructor(state: ServiceWorkerStateName) {
    super();
    this.state = state;
  }

  postMessage(message: unknown): void {
    this.messages.push(message);
  }

  setState(state: ServiceWorkerStateName): void {
    this.state = state;
    this.dispatchEvent(new Event("statechange"));
  }
}

class FakeRegistration extends EventTarget implements ServiceWorkerRegistrationLike {
  installing: FakeWorker | null = null;
  waiting: FakeWorker | null = null;
  active: FakeWorker | null = null;
  readonly scope = "https://example.test/";
  updates = 0;
  unregistered = false;

  update(): Promise<void> {
    this.updates += 1;
    return Promise.resolve();
  }

  unregister(): Promise<boolean> {
    this.unregistered = true;
    return Promise.resolve(true);
  }

  /** Simulate a new script being found and installed. */
  findUpdate(): FakeWorker {
    const worker = new FakeWorker("installing");
    this.installing = worker;
    this.dispatchEvent(new Event("updatefound"));
    return worker;
  }

  /** Move the installing worker to waiting. */
  finishInstall(): void {
    const worker = this.installing;
    this.installing = null;
    this.waiting = worker;
    worker?.setState("installed");
  }
}

class FakeContainer extends EventTarget implements ServiceWorkerContainerLike {
  controller: FakeWorker | null = null;
  readonly registration = new FakeRegistration();
  readonly calls: { url: string | URL; options: ServiceWorkerRegisterOptions | undefined }[] = [];
  failure: unknown = undefined;

  register(
    url: string | URL,
    options?: ServiceWorkerRegisterOptions,
  ): Promise<ServiceWorkerRegistrationLike> {
    this.calls.push({ url, options });
    return this.failure === undefined
      ? Promise.resolve(this.registration)
      : Promise.reject(this.failure);
  }

  takeControl(worker: FakeWorker): void {
    this.controller = worker;
    this.dispatchEvent(new Event("controllerchange"));
  }
}

async function flushPromises(): Promise<void> {
  for (let index = 0; index < 4; index += 1) await Promise.resolve();
}

void test("registers with the provided options and mirrors worker states", async () => {
  const container = new FakeContainer();
  container.registration.active = new FakeWorker("activated");
  const sw = useServiceWorker("/sw.js", {
    container,
    scope: "/app/",
    type: "module",
    updateViaCache: "none",
  });
  assert.equal(sw.supported.value, true);
  await flushPromises();
  assert.deepEqual(container.calls, [
    { url: "/sw.js", options: { scope: "/app/", type: "module", updateViaCache: "none" } },
  ]);
  assert.equal(sw.registration.value, container.registration);
  assert.deepEqual(sw.state.value, { installing: null, waiting: null, active: "activated" });
});

void test("tracks the update flow and posts the skip-waiting message", async () => {
  const container = new FakeContainer();
  const current = new FakeWorker("activated");
  container.registration.active = current;
  container.controller = current;
  const changes: (ServiceWorkerLike | null)[] = [];
  const sw = useServiceWorker("/sw.js", {
    container,
    onControllerChange: (controller) => changes.push(controller),
  });
  await flushPromises();
  assert.equal(sw.updateAvailable.value, false);
  assert.equal(sw.skipWaiting(), false);

  const next = container.registration.findUpdate();
  assert.equal(sw.state.value.installing, "installing");
  container.registration.finishInstall();
  assert.deepEqual(sw.state.value, { installing: null, waiting: "installed", active: "activated" });
  assert.equal(sw.updateAvailable.value, true);

  assert.equal(sw.skipWaiting(), true);
  assert.deepEqual(next.messages, [{ type: "SKIP_WAITING" }]);
  container.registration.waiting = null;
  container.registration.active = next;
  next.setState("activated");
  container.takeControl(next);
  assert.deepEqual(changes, [next]);
  assert.equal(sw.updateAvailable.value, false);
});

void test("a first install without a controller is not an update", async () => {
  const container = new FakeContainer();
  const sw = useServiceWorker("/sw.js", { container, skipWaitingMessage: "skip" });
  await flushPromises();
  container.registration.findUpdate();
  container.registration.finishInstall();
  assert.equal(sw.updateAvailable.value, false);
  assert.equal(sw.skipWaiting(), true);
  assert.deepEqual(container.registration.waiting?.messages, ["skip"]);
});

void test("update and unregister delegate to the registration", async () => {
  const container = new FakeContainer();
  const sw = useServiceWorker(() => new URL("https://example.test/sw.js"), {
    container,
    immediate: false,
  });
  assert.equal(await sw.update(), false);
  assert.equal(await sw.unregister(), false);
  assert.equal(await sw.register(), container.registration);
  assert.ok(container.calls[0]?.url instanceof URL);
  assert.equal(await sw.update(), true);
  assert.equal(container.registration.updates, 1);
  assert.equal(await sw.unregister(), true);
  assert.equal(sw.registration.value, null);
  assert.deepEqual(sw.state.value, { installing: null, waiting: null, active: null });
});

void test("registration failures land in error", async () => {
  const container = new FakeContainer();
  const failure = new Error("SecurityError");
  container.failure = failure;
  const sw = useServiceWorker("/sw.js", { container });
  await flushPromises();
  assert.equal(sw.error.value, failure);
  assert.equal(sw.registration.value, null);
});

void test("removes listeners when the scope stops", async () => {
  const container = new FakeContainer();
  container.controller = new FakeWorker("activated");
  let changes = 0;
  const scope = effectScope();
  const sw = scope.run(() =>
    useServiceWorker("/sw.js", { container, onControllerChange: () => (changes += 1) }),
  );
  assert.ok(sw);
  await flushPromises();
  scope.stop();
  container.registration.findUpdate();
  container.takeControl(new FakeWorker("activated"));
  assert.equal(changes, 0);
  assert.deepEqual(sw.state.value, { installing: null, waiting: null, active: null });
});

void test("reports unsupported without a container", async () => {
  const sw = useServiceWorker("/sw.js", { container: null });
  assert.equal(sw.supported.value, false);
  assert.equal(await sw.register(), null);
});

void test("server rendering registers nothing", async () => {
  const state = await renderComposableOnServer(() => {
    const sw = useServiceWorker("/sw.js");
    return {
      supported: sw.supported,
      state: sw.state,
      updateAvailable: sw.updateAvailable,
      registration: sw.registration,
    };
  });
  assert.equal(
    state,
    '{"supported":false,"state":{"installing":null,"waiting":null,"active":null},"updateAvailable":false,"registration":null}',
  );
});
