import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope, nextTick, ref } from "vue";

import { renderComposableOnServer } from "./testing/ssr-harness.ts";
import { useBroadcastChannel } from "./use-broadcast-channel.ts";
import type {
  BroadcastChannelFailure,
  BroadcastChannelHost,
  BroadcastChannelLike,
} from "./use-broadcast-channel.ts";

interface NamedChannel extends BroadcastChannelLike {
  readonly name: string;
}

function createHub(): {
  readonly Host: BroadcastChannelHost;
  readonly open: Set<NamedChannel>;
  readonly deliver: (name: string, data: unknown, from?: NamedChannel) => void;
} {
  const open = new Set<NamedChannel>();
  const deliver = (name: string, data: unknown, from?: NamedChannel): void => {
    for (const channel of open) {
      if (channel !== from && channel.name === name) {
        channel.dispatchEvent(new MessageEvent("message", { data }));
      }
    }
  };
  class FakeChannel extends EventTarget implements NamedChannel {
    readonly name: string;
    constructor(name: string) {
      super();
      this.name = name;
      open.add(this);
    }
    readonly postMessage = (message: unknown): void => {
      if (!open.has(this)) throw new Error("InvalidStateError");
      deliver(this.name, message, this);
    };
    readonly close = (): void => {
      open.delete(this);
    };
  }
  return { Host: { BroadcastChannel: FakeChannel }, open, deliver };
}

type Ping = { readonly type: "ping"; readonly at: number };
const isPing = (data: unknown): data is Ping =>
  typeof data === "object" &&
  data !== null &&
  "type" in data &&
  data.type === "ping" &&
  "at" in data &&
  typeof data.at === "number";

void test("exchanges typed messages between instances", () => {
  const { Host } = createHub();
  const received: Ping[] = [];
  const sender = useBroadcastChannel<Ping>("sync", { host: Host });
  const receiver = useBroadcastChannel<Ping>("sync", {
    host: Host,
    onMessage: (message) => received.push(message),
  });

  assert.equal(sender.supported.value, true);
  assert.equal(sender.closed.value, false);
  assert.equal(sender.post({ type: "ping", at: 1 }), true);
  assert.deepEqual(receiver.data.value, { type: "ping", at: 1 });
  assert.equal(sender.data.value, undefined, "a channel does not receive its own messages");
  assert.deepEqual(received, [{ type: "ping", at: 1 }]);
});

void test("rejects payloads that fail validation", () => {
  const { Host, deliver } = createHub();
  const failures: BroadcastChannelFailure[] = [];
  const channel = useBroadcastChannel("sync", {
    host: Host,
    validate: isPing,
    onError: (failure) => failures.push(failure),
  });

  deliver("sync", { type: "pong" });
  assert.equal(channel.data.value, undefined);
  assert.equal(channel.error.value?.code, "invalid-message");
  deliver("sync", { type: "ping", at: 2 });
  assert.deepEqual(channel.data.value, { type: "ping", at: 2 });
  assert.equal(channel.error.value, undefined);
  assert.equal(failures.length, 1);
});

void test("reports messageerror events", () => {
  const { Host, open } = createHub();
  const channel = useBroadcastChannel("sync", { host: Host });
  for (const instance of open) instance.dispatchEvent(new Event("messageerror"));
  assert.equal(channel.error.value?.code, "message-error");
});

void test("reopens when the reactive name changes", async () => {
  const { Host, open, deliver } = createHub();
  const name = ref("a");
  const channel = useBroadcastChannel<string>(name, { host: Host });

  name.value = "b";
  await nextTick();
  assert.equal(open.size, 1);
  deliver("a", "stale");
  assert.equal(channel.data.value, undefined);
  deliver("b", "fresh");
  assert.equal(channel.data.value, "fresh");
});

void test("closes with the scope and after close()", () => {
  const { Host, open } = createHub();
  const scope = effectScope();
  const scoped = scope.run(() => useBroadcastChannel<string>("x", { host: Host }));
  assert.ok(scoped);
  assert.equal(open.size, 1);
  scope.stop();
  assert.equal(open.size, 0);
  assert.equal(scoped.closed.value, true);
  assert.equal(scoped.post("late"), false);

  const manual = useBroadcastChannel<string>("x", { host: Host });
  manual.close();
  manual.close();
  assert.equal(open.size, 0);
  assert.equal(manual.post("late"), false);
});

void test("reports open and post failures without throwing", () => {
  class Broken extends EventTarget implements BroadcastChannelLike {
    constructor(name: string) {
      super();
      if (name === "bad") throw new Error("denied");
    }
    readonly postMessage = (): void => {
      throw new Error("DataCloneError");
    };
    readonly close = (): void => undefined;
  }
  const bad = useBroadcastChannel("bad", { host: { BroadcastChannel: Broken } });
  assert.equal(bad.error.value?.code, "open-failed");
  assert.equal(bad.closed.value, true);

  const good = useBroadcastChannel<() => void>("good", { host: { BroadcastChannel: Broken } });
  assert.equal(
    good.post(() => undefined),
    false,
  );
  assert.equal(good.error.value?.code, "post-failed");
});

void test("stays closed without a host", () => {
  const channel = useBroadcastChannel("x", { host: null });
  assert.equal(channel.supported.value, false);
  assert.equal(channel.closed.value, true);
  assert.equal(channel.post("x"), false);
});

void test("server rendering opens no channel", async () => {
  const state = await renderComposableOnServer(() => {
    const channel = useBroadcastChannel<string>("x");
    return { data: channel.data, supported: channel.supported, closed: channel.closed };
  });
  assert.equal(state, '{"supported":false,"closed":true}');
});
