import assert from "node:assert/strict";
import { test } from "node:test";

import {
  createDataClient,
  createDataTestAdapter,
  createFetchDataResource,
  createStreamDataResource,
  defineDataResource,
  hydrateDataClientFromStream,
  streamDataSnapshot,
} from "./index.ts";

void test("fetch adapter preserves endpoint input, cancellation and typed JSON", async () => {
  const requests: string[] = [];
  const resource = createFetchDataResource<"article.fetch", { id: string }, { title: string }>({
    key: "article.fetch",
    source: "../fixtures/ArticleView.vue",
    endpoint: ({ id }) => `https://example.test/articles/${id}`,
    fetch: async (input, init) => {
      const target =
        typeof input === "string" ? input : input instanceof URL ? input.href : input.url;
      requests.push(`${target}:${String(init?.signal?.aborted)}`);
      return new Response(JSON.stringify({ title: "Adapters" }), {
        headers: { "content-type": "application/json" },
      });
    },
  });
  const state = await createDataClient().load(resource, { id: "intro" });
  assert.equal(state.status, "success");
  assert.deepEqual(state.status === "success" ? state.data : undefined, { title: "Adapters" });
  assert.deepEqual(requests, ["https://example.test/articles/intro:false"]);
});

void test("stream adapter reduces chunks and incremental hydration restores them", async () => {
  const resource = createStreamDataResource({
    key: "events.stream",
    source: "../fixtures/ArticleView.vue",
    initial: () => [] as number[],
    async *open() {
      yield 1;
      yield 2;
      yield 3;
    },
    reduce: (items, item: number) => [...items, item],
  });
  const server = createDataClient();
  const state = await server.load(resource, {});
  assert.deepEqual(state.status === "success" ? state.data : undefined, [1, 2, 3]);

  const hydrated = await hydrateDataClientFromStream(streamDataSnapshot(server.snapshot()));
  assert.equal((await hydrated.load(resource, {})).status, "success");
});

void test("queue-backed test adapter records contexts and deterministic failures", async () => {
  const adapter = createDataTestAdapter<{ id: string }, { id: string }>();
  adapter.enqueue(({ input }) => ({ id: input.id }));
  adapter.enqueue(new Error("planned"));
  const resource = defineDataResource({
    key: "test.adapter",
    source: "../fixtures/UserCard.vue",
    loader: adapter.loader,
    serializeError: (error) => String(error),
  });
  const client = createDataClient();
  assert.equal((await client.load(resource, { id: "one" })).status, "success");
  assert.equal((await client.load(resource, { id: "two" }, { policy: "replace" })).status, "error");
  assert.deepEqual(
    adapter.calls.map(({ input }) => input.id),
    ["one", "two"],
  );
});
