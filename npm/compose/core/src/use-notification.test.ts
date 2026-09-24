import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope, ref } from "vue";

import { renderComposableOnServer } from "./testing/ssr-harness.ts";
import { useWebNotification } from "./use-notification.ts";
import type { NotificationHost, NotificationOptionsLike } from "./use-notification.ts";

interface FakeRegistry {
  permission: string;
  answer: string;
  requests: number;
  created: FakeNotification[];
  throwOnCreate: boolean;
}

class FakeNotification extends EventTarget {
  readonly title: string;
  readonly options: NotificationOptionsLike<unknown> | undefined;
  closed = false;

  constructor(title: string, options?: NotificationOptionsLike<unknown>) {
    super();
    this.title = title;
    this.options = options;
  }

  close(): void {
    if (this.closed) return;
    this.closed = true;
    this.dispatchEvent(new Event("close"));
  }
}

function createHost(
  permission = "default",
  answer = "granted",
): {
  host: NotificationHost;
  registry: FakeRegistry;
} {
  const registry: FakeRegistry = {
    permission,
    answer,
    requests: 0,
    created: [],
    throwOnCreate: false,
  };
  class Host extends FakeNotification {
    static get permission(): string {
      return registry.permission;
    }

    static requestPermission(): Promise<string> {
      registry.requests += 1;
      registry.permission = registry.answer;
      return Promise.resolve(registry.answer);
    }

    constructor(title: string, options?: NotificationOptionsLike<unknown>) {
      if (registry.throwOnCreate) throw new TypeError("illegal constructor");
      super(title, options);
      registry.created.push(this);
    }
  }
  return { host: Host, registry };
}

void test("requests permission on first show and merges defaults", async () => {
  const { host, registry } = createHost();
  const icon = ref("/a.png");
  const notify = useWebNotification<{ id: number }>({
    host,
    defaults: () => ({ title: "Default", icon: icon.value }),
  });
  assert.equal(notify.permission.value, "default");

  const result = await notify.show(undefined, { body: "hello", data: { id: 1 } });
  assert.equal(result.status, "shown");
  assert.equal(registry.requests, 1);
  assert.equal(notify.permission.value, "granted");
  assert.equal(registry.created[0]?.title, "Default");
  assert.deepEqual(registry.created[0]?.options, {
    icon: "/a.png",
    body: "hello",
    data: { id: 1 },
  });
  assert.deepEqual(notify.data.value, { id: 1 });
});

void test("reports denial without constructing a notification", async () => {
  const { host, registry } = createHost("default", "denied");
  const notify = useWebNotification({ host });
  assert.deepEqual(await notify.show("x"), { status: "denied", error: undefined });
  assert.equal(registry.created.length, 0);

  const passive = useWebNotification({ host: createHost().host, requestPermissionOnShow: false });
  assert.equal((await passive.show("x")).status, "denied");
});

void test("dispatches lifecycle events to registered handlers", async () => {
  const { host, registry } = createHost("granted");
  const notify = useWebNotification({ host });
  const seen: string[] = [];
  const stop = notify.on("click", (event) => seen.push(event.type));
  notify.on("close", (event) => seen.push(event.type));

  await notify.show("a");
  registry.created[0]?.dispatchEvent(new Event("click"));
  stop();
  registry.created[0]?.dispatchEvent(new Event("click"));
  registry.created[0]?.close();
  assert.deepEqual(seen, ["click", "close"]);
  assert.equal(notify.notification.value, undefined);
});

void test("replaces the current notification and closes it with the scope", async () => {
  const { host, registry } = createHost("granted");
  const scope = effectScope();
  const notify = scope.run(() => useWebNotification({ host }));
  assert.ok(notify);

  await notify.show("first");
  await notify.show("second");
  assert.equal(registry.created[0]?.closed, true);
  assert.equal(notify.notification.value, registry.created[1]);

  scope.stop();
  assert.equal(registry.created[1]?.closed, true);
});

void test("reports construction failures and missing support", async () => {
  const { host, registry } = createHost("granted");
  registry.throwOnCreate = true;
  const notify = useWebNotification({ host });
  assert.equal((await notify.show("x")).status, "failed");

  const missing = useWebNotification({ host: null });
  assert.equal(missing.supported.value, false);
  assert.equal(missing.permission.value, "unsupported");
  assert.equal(await missing.requestPermission(), "unsupported");
  assert.equal((await missing.show("x")).status, "unsupported");
});

void test("server rendering never touches the Notifications API", async () => {
  const state = await renderComposableOnServer(() => {
    const notify = useWebNotification();
    return { supported: notify.supported, permission: notify.permission };
  });
  assert.equal(state, '{"supported":false,"permission":"unsupported"}');
});
