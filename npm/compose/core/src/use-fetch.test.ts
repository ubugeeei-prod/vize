import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope, nextTick, ref } from "vue";

import { renderComposableOnServer } from "./testing/ssr-harness.ts";
import type { TimeoutScheduler } from "./timeout-scheduler.ts";
import { useFetch } from "./use-fetch.ts";
import type { FetchImplementation } from "./use-fetch.ts";

interface Call {
  readonly url: string;
  readonly init: RequestInit;
  readonly resolve: (response: Response) => void;
  readonly reject: (error: unknown) => void;
}

function createFetch(): { readonly fetch: FetchImplementation; readonly calls: Call[] } {
  const calls: Call[] = [];
  const fetch: FetchImplementation = (url, init) =>
    new Promise((resolve, reject) => {
      calls.push({ url, init, resolve, reject });
      init.signal?.addEventListener("abort", () => reject(init.signal?.reason), { once: true });
    });
  return { fetch, calls };
}

function json(body: unknown, status = 200): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: { "content-type": "application/json" },
  });
}

function createScheduler(): TimeoutScheduler & { readonly flush: () => void; pending: number } {
  const timers = new Map<number, () => void>();
  let next = 0;
  const scheduler = {
    pending: 0,
    setTimeout: (callback: () => void) => {
      next += 1;
      timers.set(next, callback);
      scheduler.pending = timers.size;
      return next;
    },
    clearTimeout: (handle: unknown) => {
      if (typeof handle === "number") timers.delete(handle);
      scheduler.pending = timers.size;
    },
    flush: () => {
      const callbacks = [...timers.values()];
      timers.clear();
      scheduler.pending = 0;
      for (const callback of callbacks) callback();
    },
  };
  return scheduler;
}

async function settle(): Promise<void> {
  await new Promise((resolve) => setImmediate(resolve));
}

void test("executes immediately and commits validated JSON", async () => {
  const { fetch, calls } = createFetch();
  const isUser = (value: unknown): value is { readonly name: string } =>
    typeof value === "object" && value !== null && "name" in value;
  const user = useFetch("/user", { fetch, validate: isUser });

  assert.equal(user.status.value, "pending");
  await settle();
  assert.equal(calls.length, 1);
  calls[0]?.resolve(json({ name: "ada" }));
  await settle();
  assert.equal(user.status.value, "success");
  assert.deepEqual(user.data.value, { name: "ada" });
  assert.equal(user.statusCode.value, 200);
  assert.equal(user.pending.value, false);
});

void test("reports invalid bodies, HTTP errors, and parse errors", async () => {
  const { fetch, calls } = createFetch();
  const isNumber = (value: unknown): value is number => typeof value === "number";
  const resource = useFetch("/n", { fetch, validate: isNumber, immediate: false });

  const invalid = resource.execute();
  await settle();
  calls[0]?.resolve(json("nope"));
  assert.deepEqual(await invalid, { status: "error", error: { kind: "invalid", data: "nope" } });

  const http = resource.execute();
  await settle();
  const notFound = json({}, 404);
  calls[1]?.resolve(notFound);
  const httpResult = await http;
  assert.equal(httpResult.status, "error");
  assert.equal(resource.error.value?.kind, "http");
  assert.equal(resource.statusCode.value, 404);

  const parse = resource.execute();
  await settle();
  calls[2]?.resolve(new Response("{broken"));
  await parse;
  assert.equal(resource.error.value?.kind, "parse");
  assert.equal(resource.status.value, "error");
});

void test("selects built-in body readers and custom parsers", async () => {
  const { fetch, calls } = createFetch();
  const text = useFetch("/t", { fetch, responseType: "text" });
  await settle();
  calls[0]?.resolve(new Response("hello"));
  await settle();
  assert.equal(text.data.value, "hello");

  const parsed = useFetch("/p", {
    fetch,
    parse: async (response) => (await response.text()).length,
  });
  await settle();
  calls[1]?.resolve(new Response("four"));
  await settle();
  assert.equal(parsed.data.value, 4);
});

void test("latest execution wins and superseded requests are aborted", async () => {
  const { fetch, calls } = createFetch();
  const resource = useFetch("/x", { fetch, responseType: "text", immediate: false });

  const first = resource.execute();
  await settle();
  const second = resource.execute();
  assert.equal(calls[0]?.init.signal?.aborted, true);
  assert.deepEqual(await first, { status: "superseded" });
  await settle();
  calls[1]?.resolve(new Response("second"));
  const result = await second;
  assert.equal(result.status, "success");
  assert.equal(resource.data.value, "second");
});

void test("manual abort settles as an aborted failure", async () => {
  const { fetch } = createFetch();
  const resource = useFetch("/x", { fetch, immediate: false });
  const execution = resource.execute();

  assert.equal(resource.abort("stop"), true);
  assert.deepEqual(await execution, {
    status: "error",
    error: { kind: "aborted", reason: "stop" },
  });
  assert.equal(resource.status.value, "aborted");
  assert.equal(resource.abort(), false);
});

void test("times out through the injected scheduler", async () => {
  const { fetch } = createFetch();
  const scheduler = createScheduler();
  const resource = useFetch("/slow", { fetch, scheduler, timeoutMs: 50, immediate: false });
  const execution = resource.execute();

  scheduler.flush();
  assert.deepEqual(await execution, { status: "error", error: { kind: "timeout", timeoutMs: 50 } });
  assert.equal(resource.status.value, "error");
});

void test("retries network failures and 5xx responses but not 4xx", async () => {
  const { fetch, calls } = createFetch();
  const scheduler = createScheduler();
  const resource = useFetch("/r", {
    fetch,
    scheduler,
    retry: 2,
    responseType: "text",
    immediate: false,
  });

  const execution = resource.execute();
  await settle();
  calls[0]?.reject(new TypeError("offline"));
  await settle();
  scheduler.flush();
  await settle();
  calls[1]?.resolve(new Response("", { status: 503 }));
  await settle();
  scheduler.flush();
  await settle();
  calls[2]?.resolve(new Response("ok"));
  assert.equal((await execution).status, "success");
  assert.equal(calls.length, 3);

  const clientError = resource.execute();
  await settle();
  calls[3]?.resolve(new Response("", { status: 400 }));
  const result = await clientError;
  assert.equal(result.status, "error");
  assert.equal(calls.length, 4);
});

void test("interceptors rewrite requests, cancel them, and transform data", async () => {
  const { fetch, calls } = createFetch();
  const resource = useFetch("/a", {
    fetch,
    responseType: "text",
    immediate: false,
    beforeRequest: ({ url, init }) =>
      url.endsWith("/cancel")
        ? false
        : { url: `${url}?v=1`, init: { ...init, headers: { authorization: "token" } } },
    afterResponse: ({ data }) => data.toUpperCase(),
  });

  const execution = resource.execute();
  await settle();
  assert.equal(calls[0]?.url, "/a?v=1");
  assert.deepEqual(calls[0]?.init.headers, { authorization: "token" });
  calls[0]?.resolve(new Response("up"));
  await execution;
  assert.equal(resource.data.value, "UP");

  const cancelled = useFetch("/cancel", {
    fetch,
    immediate: false,
    beforeRequest: () => false,
  });
  const result = await cancelled.execute();
  assert.equal(result.status, "error");
  assert.equal(cancelled.status.value, "aborted");
  assert.equal(calls.length, 1);
});

void test("refetches when the reactive URL or init changes and aborts on dispose", async () => {
  const { fetch, calls } = createFetch();
  const id = ref<number | null>(null);
  const method = ref("GET");
  const scope = effectScope();
  const resource = scope.run(() =>
    useFetch(() => (id.value === null ? null : `/item/${String(id.value)}`), {
      fetch,
      init: () => ({ method: method.value }),
    }),
  );
  assert.ok(resource);
  assert.equal(calls.length, 0);
  assert.equal(resource.status.value, "idle");

  id.value = 1;
  await nextTick();
  await settle();
  assert.equal(calls[0]?.url, "/item/1");
  method.value = "HEAD";
  await nextTick();
  await settle();
  assert.equal(calls[1]?.init.method, "HEAD");

  scope.stop();
  assert.equal(calls[1]?.init.signal?.aborted, true);
});

void test("reports a missing fetch implementation without throwing", async () => {
  const resource = useFetch("/x");
  const result = await resource.execute();
  assert.equal(result.status, "error");
  assert.equal(resource.error.value?.kind, "network");
});

void test("rejects invalid timeouts", () => {
  assert.throws(() => useFetch("/x", { timeoutMs: -1 }), /VIZE_COMPOSE_FETCH_INVALID_TIMEOUT/);
});

void test("server rendering performs no automatic request", async () => {
  const state = await renderComposableOnServer(() => {
    const resource = useFetch<{ readonly id: number }>("/api", { initialData: { id: 0 } });
    return { data: resource.data, status: resource.status, pending: resource.pending };
  });

  assert.equal(state, '{"data":{"id":0},"status":"idle","pending":false}');
});
