import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope, nextTick, ref } from "vue";

import { renderComposableOnServer } from "./testing/ssr-harness.ts";
import type { TimeoutScheduler } from "./timeout-scheduler.ts";
import { useEventSource } from "./use-event-source.ts";
import type { EventSourceLike } from "./use-event-source.ts";

class FakeEventSource extends EventTarget implements EventSourceLike {
  static readonly instances: FakeEventSource[] = [];
  readonly url: string;
  readonly withCredentials: boolean;
  readyState = 0;
  listening = new Map<string, number>();

  constructor(url: string, init?: { readonly withCredentials?: boolean }) {
    super();
    this.url = url;
    this.withCredentials = init?.withCredentials ?? false;
    FakeEventSource.instances.push(this);
  }

  override addEventListener(type: string, listener: EventListener): void {
    this.listening.set(type, (this.listening.get(type) ?? 0) + 1);
    super.addEventListener(type, listener);
  }

  override removeEventListener(type: string, listener: EventListener): void {
    this.listening.set(type, (this.listening.get(type) ?? 0) - 1);
    super.removeEventListener(type, listener);
  }

  close(): void {
    this.readyState = 2;
  }

  serverOpen(): void {
    this.readyState = 1;
    this.dispatchEvent(new Event("open"));
  }

  serverEvent(type: string, data: string, lastEventId = ""): void {
    this.dispatchEvent(new MessageEvent(type, { data, lastEventId }));
  }

  serverError(fatal: boolean): void {
    this.readyState = fatal ? 2 : 0;
    this.dispatchEvent(new Event("error"));
  }
}

function latest(): FakeEventSource {
  const source = FakeEventSource.instances.at(-1);
  assert.ok(source);
  return source;
}

function createScheduler(): TimeoutScheduler & { readonly run: () => void; count: number } {
  const timers = new Map<number, () => void>();
  let next = 0;
  const scheduler = {
    count: 0,
    setTimeout: (callback: () => void) => {
      next += 1;
      scheduler.count += 1;
      timers.set(next, callback);
      return next;
    },
    clearTimeout: (handle: unknown) => {
      if (typeof handle === "number") timers.delete(handle);
    },
    run: () => {
      const callbacks = [...timers.values()];
      timers.clear();
      for (const callback of callbacks) callback();
    },
  };
  return scheduler;
}

void test("receives default message events as text", () => {
  const stream = useEventSource("/sse", { host: FakeEventSource, withCredentials: true });
  const source = latest();

  assert.equal(source.url, "/sse");
  assert.equal(source.withCredentials, true);
  assert.equal(stream.status.value, "connecting");
  source.serverOpen();
  assert.equal(stream.status.value, "open");

  source.serverEvent("message", "hello", "7");
  assert.equal(stream.data.value, "hello");
  assert.equal(stream.event.value, "message");
  assert.equal(stream.lastEventId.value, "7");
  assert.deepEqual(stream.message.value, { event: "message", data: "hello", lastEventId: "7" });
});

void test("decodes named events with typed parsers and handlers", () => {
  const prices: number[] = [];
  const stream = useEventSource<{ price: number; notice: string }>("/feed", {
    host: FakeEventSource,
    events: ["price", "notice"],
    parse: { price: (raw) => Number(raw) },
  });
  const off = stream.on("price", (price) => prices.push(price));
  const source = latest();

  source.serverEvent("price", "42");
  source.serverEvent("notice", "closing soon");
  assert.deepEqual(prices, [42]);
  assert.equal(stream.data.value, "closing soon");
  assert.equal(stream.event.value, "notice");

  off();
  source.serverEvent("price", "43");
  assert.deepEqual(prices, [42]);
  assert.equal(stream.data.value, 43);
});

void test("reports parser failures without replacing data", () => {
  const stream = useEventSource<{ message: { readonly ok: boolean } }>("/json", {
    host: FakeEventSource,
    parse: {
      message: (raw) => {
        const value: unknown = JSON.parse(raw);
        return { ok: typeof value === "object" && value !== null };
      },
    },
  });
  const source = latest();
  source.serverEvent("message", "{}");
  source.serverEvent("message", "{broken");

  assert.deepEqual(stream.data.value, { ok: true });
  assert.equal(stream.error.value?.kind, "parse");
});

void test("leaves transient errors to native reconnection", () => {
  const scheduler = createScheduler();
  const stream = useEventSource("/t", { host: FakeEventSource, scheduler, autoReconnect: true });
  const source = latest();
  source.serverOpen();
  source.serverError(false);

  assert.equal(stream.status.value, "connecting");
  assert.equal(stream.error.value?.kind, "error");
  assert.equal(scheduler.count, 0);
  assert.equal(latest(), source);
});

void test("reopens streams the browser closed, until retries run out", () => {
  const scheduler = createScheduler();
  let failed = false;
  const stream = useEventSource("/r", {
    host: FakeEventSource,
    scheduler,
    autoReconnect: { retries: 1, onFailed: () => (failed = true) },
  });
  const first = latest();
  first.serverError(true);
  assert.equal(stream.status.value, "closed");
  scheduler.run();
  const second = latest();
  assert.notEqual(second, first);
  assert.equal(first.listening.get("message"), 0, "old listeners are removed");

  second.serverError(true);
  assert.equal(failed, true);
  assert.equal(stream.error.value?.kind, "reconnect-exhausted");
});

void test("reconnects on URL change, respects immediate, and closes with the scope", async () => {
  const url = ref("/a");
  const scope = effectScope();
  const stream = scope.run(() => useEventSource(url, { host: FakeEventSource, immediate: false }));
  assert.ok(stream);
  const before = FakeEventSource.instances.length;
  assert.equal(stream.status.value, "closed");

  stream.open();
  const first = latest();
  url.value = "/b";
  await nextTick();
  const second = latest();
  assert.equal(FakeEventSource.instances.length, before + 2);
  assert.equal(second.url, "/b");
  assert.equal(first.readyState, 2);

  scope.stop();
  assert.equal(second.readyState, 2);
  assert.equal(stream.status.value, "closed");
});

void test("reports constructor failures", () => {
  class Throwing extends FakeEventSource {
    constructor(url: string) {
      super(url);
      throw new SyntaxError("bad url");
    }
  }
  const stream = useEventSource("::", { host: Throwing });
  assert.equal(stream.error.value?.kind, "connect");
});

void test("server rendering never connects", async () => {
  const state = await renderComposableOnServer(() => {
    const stream = useEventSource("/sse");
    return { status: stream.status, event: stream.event };
  });
  assert.equal(state, '{"status":"closed"}');
});
