import assert from "node:assert/strict";
import { test } from "node:test";

import {
  createDataClient,
  createDataManifest,
  defineDataResource,
  hydrateDataClient,
  serializeDataSnapshot,
  streamDataSnapshot,
} from "./index.ts";

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((done) => {
    resolve = done;
  });
  return { promise, resolve };
}

void test("hydrates SSR data without duplicate client work", async () => {
  let calls = 0;
  const article = defineDataResource({
    key: "article.byId",
    source: "../fixtures/ArticleView.vue",
    async loader({ input }: { input: { id: string } }) {
      calls += 1;
      return { id: input.id, title: `Article ${input.id}` };
    },
  });

  const server = createDataClient({ now: () => 100 });
  assert.deepEqual(await server.load(article, { id: "intro" }), {
    status: "success",
    key: "article.byId",
    input: { id: "intro" },
    requestKey: 'article.byId:{"id":"intro"}',
    data: { id: "intro", title: "Article intro" },
    updatedAt: 100,
  });

  const client = hydrateDataClient(serializeDataSnapshot(server));
  const hydrated = await client.load(article, { id: "intro" });
  assert.equal(hydrated.status, "success");
  assert.equal(calls, 1);
});

void test("replace policy bypasses hydrated success entries", async () => {
  let calls = 0;
  const article = defineDataResource({
    key: "article.replace",
    source: "../fixtures/ArticleView.vue",
    async loader({ input }: { input: { id: string } }) {
      calls += 1;
      return { id: input.id, title: `Article ${calls}` };
    },
  });

  const server = createDataClient({ now: () => 100 });
  await server.load(article, { id: "intro" });

  const client = hydrateDataClient(serializeDataSnapshot(server), { now: () => 200 });
  assert.equal((await client.load(article, { id: "intro" })).status, "success");

  const replaced = await client.load(article, { id: "intro" }, { policy: "replace" });
  assert.equal(replaced.status, "success");
  if (replaced.status === "success")
    assert.deepEqual(replaced.data, { id: "intro", title: "Article 2" });
  assert.equal(calls, 2);
});

void test("deduplicates in-flight requests by resource key and input", async () => {
  const gate = deferred<{ name: string }>();
  let calls = 0;
  const user = defineDataResource({
    key: "user.profile",
    source: "../fixtures/UserCard.vue",
    loader() {
      calls += 1;
      return gate.promise;
    },
  });

  const client = createDataClient();
  const first = client.load(user, { id: "ada" });
  const second = client.load(user, { id: "ada" });

  assert.strictEqual(first, second);
  assert.equal(client.peek(user, { id: "ada" })?.status, "pending");
  gate.resolve({ name: "Ada" });

  const state = await second;
  assert.equal(state.status, "success");
  assert.equal(calls, 1);
});

void test("retries failed loads with deterministic backoff", async () => {
  const delays: number[] = [];
  const reasons: string[] = [];
  let calls = 0;
  const flaky = defineDataResource({
    key: "flaky.example",
    source: "../fixtures/UserCard.vue",
    loader({ reason }) {
      calls += 1;
      reasons.push(reason);
      if (calls === 1) throw new Error("temporary");
      return { ok: true };
    },
    serializeError(error) {
      return { message: error instanceof Error ? error.message : String(error) };
    },
  });
  const client = createDataClient({
    now: () => 350,
    sleep(milliseconds, signal) {
      if (signal.aborted) throw signal.reason;
      delays.push(milliseconds);
      return Promise.resolve();
    },
  });

  const state = await client.load(
    flaky,
    {},
    {
      retries: 1,
      retryDelayMs: ({ attempt }) => attempt * 25,
    },
  );

  assert.equal(state.status, "success");
  assert.deepEqual(delays, [25]);
  assert.deepEqual(reasons, ["initial", "retry"]);
});

void test("invalidates cached success entries into stale states", async () => {
  const resource = defineDataResource({
    key: "settings.current",
    source: "../fixtures/UserCard.vue",
    loader: () => ({ theme: "dark" as const }),
  });
  const client = createDataClient({ now: () => 250 });

  await client.load(resource, {});
  assert.equal(client.invalidate(resource, {}, "mutation"), 1);

  const state = client.peek(resource, {});
  assert.equal(state?.status, "stale");
  if (state?.status === "stale") assert.deepEqual(state.data, { theme: "dark" });
});

void test("deadline cancellation serializes a cancelled state", async () => {
  const resource = defineDataResource({
    key: "deadline.example",
    source: "../fixtures/UserCard.vue",
    loader({ signal }) {
      if (signal.aborted) throw signal.reason;
      return "late";
    },
  });
  const client = createDataClient({ now: () => 425 });

  assert.deepEqual(await client.load(resource, {}, { deadlineMs: 0 }), {
    status: "cancelled",
    key: "deadline.example",
    input: {},
    requestKey: "deadline.example:{}",
    reason: "deadline",
    cancelledAt: 425,
  });
});

void test("serializes errors, cancellations, and snapshot streams", async () => {
  const cancelled = defineDataResource({
    key: "cancelled.example",
    source: "../fixtures/UserCard.vue",
    async loader({ signal }) {
      signal.throwIfAborted();
      return "never";
    },
  });
  const failed = defineDataResource({
    key: "failed.example",
    source: "../fixtures/UserCard.vue",
    loader() {
      throw new Error("boom");
    },
    serializeError(error) {
      return { message: error instanceof Error ? error.message : String(error) };
    },
  });

  const controller = new AbortController();
  controller.abort("route-left");
  const client = createDataClient({ now: () => 300 });

  assert.equal(
    (await client.load(cancelled, {}, { signal: controller.signal })).status,
    "cancelled",
  );
  assert.equal((await client.load(failed, {})).status, "error");

  const streamed = [];
  for await (const state of streamDataSnapshot(client.snapshot())) streamed.push(state.status);
  assert.deepEqual([...streamed].sort(), ["cancelled", "error"]);
});

void test("emits generator metadata from source-owned Vue fixtures", () => {
  const resources = [
    defineDataResource({
      key: "article.byId",
      source: "../fixtures/ArticleView.vue",
      meta: { owner: "content" },
      loader: () => ({ title: "Intro" }),
    }),
    defineDataResource({
      key: "user.profile",
      source: "../fixtures/UserCard.vue",
      loader: () => ({ name: "Ada" }),
    }),
  ] as const;

  assert.deepEqual(createDataManifest(resources), {
    schemaVersion: 1,
    resources: [
      {
        key: "article.byId",
        source: "../fixtures/ArticleView.vue",
        meta: { owner: "content" },
      },
      {
        key: "user.profile",
        source: "../fixtures/UserCard.vue",
        meta: {},
      },
    ],
  });
});

void test("invalid resource definitions fail closed", () => {
  assert.throws(
    () =>
      defineDataResource({
        key: "broken",
        // @ts-expect-error Runtime diagnostics still protect untyped callers.
        source: "./broken.ts",
        loader: () => null,
      }),
    /VIZE_DATA_SOURCE_NOT_VUE/,
  );
});
