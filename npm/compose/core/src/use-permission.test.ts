import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope, nextTick, ref } from "vue";

import { renderComposableOnServer } from "./testing/ssr-harness.ts";
import { usePermission } from "./use-permission.ts";
import type {
  PermissionDescriptorLike,
  PermissionsHost,
  PermissionStatusLike,
} from "./use-permission.ts";

class FakeStatus extends EventTarget implements PermissionStatusLike {
  state: string;
  listeners = 0;

  constructor(state: string) {
    super();
    this.state = state;
  }

  override addEventListener(type: string, listener: EventListener): void {
    this.listeners += 1;
    super.addEventListener(type, listener);
  }

  override removeEventListener(type: string, listener: EventListener): void {
    this.listeners -= 1;
    super.removeEventListener(type, listener);
  }

  set(state: string): void {
    this.state = state;
    this.dispatchEvent(new Event("change"));
  }
}

class FakePermissions implements PermissionsHost {
  readonly statuses = new Map<string, FakeStatus>();
  readonly descriptors: PermissionDescriptorLike[] = [];

  query(descriptor: PermissionDescriptorLike): Promise<PermissionStatusLike> {
    this.descriptors.push(descriptor);
    const status = this.statuses.get(descriptor.name);
    return status ? Promise.resolve(status) : Promise.reject(new TypeError("unknown name"));
  }
}

async function settle(): Promise<void> {
  for (let index = 0; index < 3; index += 1) await Promise.resolve();
}

void test("queries the permission and follows change events until disposal", async () => {
  const host = new FakePermissions();
  const camera = new FakeStatus("prompt");
  host.statuses.set("camera", camera);
  const scope = effectScope();
  const permission = scope.run(() => usePermission("camera", { host }));
  assert.ok(permission);

  assert.equal(permission.state.value, "unknown");
  await settle();
  assert.equal(permission.state.value, "prompt");
  assert.equal(permission.supported.value, true);

  camera.set("granted");
  assert.equal(permission.state.value, "granted");

  scope.stop();
  assert.equal(camera.listeners, 0);
  camera.set("denied");
  assert.equal(permission.state.value, "granted");
});

void test("resubscribes when the reactive name changes", async () => {
  const host = new FakePermissions();
  const camera = new FakeStatus("granted");
  const microphone = new FakeStatus("denied");
  host.statuses.set("camera", camera);
  host.statuses.set("microphone", microphone);
  const name = ref<"camera" | "microphone">("camera");
  const permission = usePermission(name, { host });

  await settle();
  assert.equal(permission.state.value, "granted");
  name.value = "microphone";
  await nextTick();
  await settle();
  assert.equal(permission.state.value, "denied");
  assert.equal(camera.listeners, 0);
  assert.equal(microphone.listeners, 1);
});

void test("passes full descriptors through unchanged", async () => {
  const host = new FakePermissions();
  host.statuses.set("push", new FakeStatus("prompt"));
  const permission = usePermission({ name: "push", userVisibleOnly: true }, { host });
  await settle();

  assert.deepEqual(host.descriptors, [{ name: "push", userVisibleOnly: true }]);
  assert.equal(permission.state.value, "prompt");
});

void test("reports unsupported names and missing hosts", async () => {
  const host = new FakePermissions();
  const unknown = usePermission("teleport", { host });
  await settle();
  assert.equal(unknown.state.value, "unsupported");

  const missing = usePermission("camera", { host: () => null });
  assert.equal(await missing.query(), "unknown", "no window: the answer stays unknown");
  assert.equal(missing.supported.value, false);
});

void test("normalizes future browser states to unknown", async () => {
  const host = new FakePermissions();
  host.statuses.set("camera", new FakeStatus("quantum"));
  const permission = usePermission("camera", { host });
  await settle();
  assert.equal(permission.state.value, "unknown");
});

void test("ignores a stale query that settles after a newer one", async () => {
  const resolvers: ((status: PermissionStatusLike) => void)[] = [];
  const host: PermissionsHost = {
    query: () => new Promise((resolve) => resolvers.push(resolve)),
  };
  const permission = usePermission("camera", { host });
  const newer = permission.query();
  resolvers[1]?.(new FakeStatus("denied"));
  await newer;
  resolvers[0]?.(new FakeStatus("granted"));
  await settle();
  assert.equal(permission.state.value, "denied");
});

void test("server rendering keeps the initial state without querying", async () => {
  const state = await renderComposableOnServer(() => {
    const permission = usePermission("geolocation");
    return { state: permission.state, supported: permission.supported };
  });
  assert.equal(state, '{"state":"unknown","supported":false}');
});
