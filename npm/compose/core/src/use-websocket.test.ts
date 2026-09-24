import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope, nextTick, ref } from "vue";

import { renderComposableOnServer } from "./testing/ssr-harness.ts";
import type { TimeoutScheduler } from "./timeout-scheduler.ts";
import { useWebSocket } from "./use-websocket.ts";
import type { WebSocketLike, WebSocketSendData } from "./use-websocket.ts";

class FakeSocket extends EventTarget implements WebSocketLike {
  static readonly instances: FakeSocket[] = [];
  readonly url: string;
  readonly protocols: string | string[] | undefined;
  readonly sent: WebSocketSendData[] = [];
  closed: { readonly code: number | undefined; readonly reason: string | undefined } | undefined;

  constructor(url: string, protocols?: string | string[]) {
    super();
    this.url = url;
    this.protocols = protocols;
    FakeSocket.instances.push(this);
  }

  send(data: WebSocketSendData): void {
    this.sent.push(data);
  }

  close(code?: number, reason?: string): void {
    this.closed = { code, reason };
  }

  serverOpen(): void {
    this.dispatchEvent(new Event("open"));
  }

  serverMessage(data: string): void {
    this.dispatchEvent(new MessageEvent("message", { data }));
  }

  serverClose(): void {
    this.dispatchEvent(new Event("close"));
  }
}

function latest(): FakeSocket {
  const socket = FakeSocket.instances.at(-1);
  assert.ok(socket);
  return socket;
}

function createScheduler(): TimeoutScheduler & {
  readonly delays: number[];
  readonly run: () => void;
} {
  const timers = new Map<number, () => void>();
  const delays: number[] = [];
  let next = 0;
  return {
    delays,
    setTimeout: (callback, delayMs) => {
      next += 1;
      timers.set(next, callback);
      delays.push(delayMs);
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

void test("connects with a constructor host, buffers while connecting, and parses messages", () => {
  const received: number[] = [];
  const socket = useWebSocket<{ readonly n: number }, { readonly type: string }>("ws://x", {
    host: FakeSocket,
    protocols: ["v1"],
    parse: (raw) => {
      const value: unknown = JSON.parse(typeof raw === "string" ? raw : "");
      if (typeof value !== "object" || value === null || !("n" in value)) throw new Error("bad");
      return { n: Number(value.n) };
    },
    serialize: (message) => JSON.stringify(message),
    onMessage: (message) => received.push(message.n),
  });
  const instance = latest();

  assert.equal(instance.url, "ws://x");
  assert.deepEqual(instance.protocols, ["v1"]);
  assert.equal(socket.status.value, "connecting");
  assert.equal(socket.send({ type: "hello" }), true);
  assert.deepEqual(instance.sent, []);

  instance.serverOpen();
  assert.equal(socket.status.value, "open");
  assert.deepEqual(instance.sent, ['{"type":"hello"}']);

  instance.serverMessage('{"n":2}');
  assert.deepEqual(socket.data.value, { n: 2 });
  assert.deepEqual(received, [2]);

  instance.serverMessage("oops");
  assert.equal(socket.error.value?.kind, "parse");
  assert.deepEqual(socket.data.value, { n: 2 });
});

void test("rejects messages failing validation", () => {
  const socket = useWebSocket("ws://v", { host: FakeSocket, validate: (raw) => raw !== "bad" });
  const instance = latest();
  instance.serverOpen();
  instance.serverMessage("bad");
  assert.equal(socket.error.value?.kind, "invalid");
  instance.serverMessage("good");
  assert.equal(socket.data.value, "good");
});

void test("send returns false while closed and when buffering is disabled", () => {
  const socket = useWebSocket("ws://y", {
    host: FakeSocket,
    immediate: false,
    bufferWhileConnecting: false,
  });
  assert.equal(socket.status.value, "closed");
  assert.equal(socket.send("x"), false);
  socket.open();
  assert.equal(socket.send("x"), false);
});

void test("reconnects with backoff after unexpected closes until retries run out", () => {
  const scheduler = createScheduler();
  let failed = 0;
  const socket = useWebSocket("ws://r", {
    host: FakeSocket,
    scheduler,
    autoReconnect: { retries: 2, initialDelayMs: 100, onFailed: () => (failed += 1) },
  });
  const first = latest();
  first.serverOpen();
  first.serverClose();
  assert.equal(socket.status.value, "closed");
  assert.equal(socket.reconnectAttempts.value, 1);
  assert.deepEqual(scheduler.delays, [100]);

  scheduler.run();
  const second = latest();
  assert.notEqual(second, first);
  second.serverClose();
  assert.deepEqual(scheduler.delays, [100, 200]);
  scheduler.run();
  latest().serverClose();
  assert.equal(failed, 1);
  assert.equal(socket.error.value?.kind, "reconnect-exhausted");
});

void test("manual close is final and does not reconnect", () => {
  const scheduler = createScheduler();
  const socket = useWebSocket("ws://c", { host: FakeSocket, scheduler, autoReconnect: true });
  const instance = latest();
  instance.serverOpen();
  socket.close(1000, "bye");

  assert.deepEqual(instance.closed, { code: 1000, reason: "bye" });
  assert.equal(socket.status.value, "closed");
  instance.serverClose();
  assert.deepEqual(scheduler.delays, []);
});

void test("heartbeat pings, swallows pongs, and drops silent peers", () => {
  const scheduler = createScheduler();
  const socket = useWebSocket("ws://h", {
    host: FakeSocket,
    scheduler,
    heartbeat: { intervalMs: 1000, pongTimeoutMs: 500 },
  });
  const instance = latest();
  instance.serverOpen();
  assert.deepEqual(scheduler.delays, [1000]);

  scheduler.run();
  assert.deepEqual(instance.sent, ["ping"]);
  instance.serverMessage("ping");
  assert.equal(socket.data.value, undefined, "pong replies are not exposed as data");
  assert.equal(instance.closed, undefined);

  scheduler.run();
  assert.deepEqual(instance.sent, ["ping", "ping"]);
  scheduler.run();
  assert.ok(instance.closed, "a missing pong closes the socket");
});

void test("reconnects when the reactive URL changes and closes on scope dispose", async () => {
  const url = ref<string | null>("ws://a");
  const scope = effectScope();
  const socket = scope.run(() => useWebSocket(url, { host: FakeSocket }));
  assert.ok(socket);
  const first = latest();

  url.value = "ws://b";
  await nextTick();
  const second = latest();
  assert.equal(second.url, "ws://b");
  assert.ok(first.closed);

  scope.stop();
  assert.ok(second.closed);
  assert.equal(socket.status.value, "closed");
});

void test("reports constructor failures", () => {
  class Throwing extends FakeSocket {
    constructor(url: string) {
      super(url);
      throw new SyntaxError("bad url");
    }
  }
  const socket = useWebSocket("nope", { host: Throwing });
  assert.equal(socket.error.value?.kind, "connect");
  assert.equal(socket.status.value, "closed");
});

void test("server rendering never connects", async () => {
  const state = await renderComposableOnServer(() => {
    const socket = useWebSocket("wss://example.test");
    return { status: socket.status, data: socket.data, attempts: socket.reconnectAttempts };
  });
  assert.equal(state, '{"status":"closed","attempts":0}');
});
