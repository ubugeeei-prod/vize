# @vizejs/data

Typed SSR data resource foundations for Vize applications.

> This package is released on npm as `@vizejs/data`; its resource and adapter contracts remain
> experimental throughout the Vize alpha line.

```vue
<script setup lang="ts">
import { createDataClient, defineDataResource } from "@vizejs/data";

const articleResource = defineDataResource({
  key: "article.byId",
  source: "./ArticleView.vue",
  async loader({ input }: { input: { id: string } }) {
    return { id: input.id, title: "Hello Vize" };
  },
});

const data = createDataClient();
const article = await data.load(
  articleResource,
  { id: "intro" },
  {
    retries: 2,
    retryDelayMs: ({ attempt }) => attempt * 100,
    deadlineMs: 1500,
  },
);
</script>

<template>
  <article v-if="article.status === 'success'">
    {{ article.data.title }}
  </article>
</template>
```

Server renderers can call `serializeDataSnapshot(data)` and hydrate the client with
`hydrateDataClient(snapshot)`. Hydrated success entries satisfy the first client load without
issuing duplicate network work; `policy: "replace"` opts back into a fresh request.

## Cache, races, and optimistic work

The client supports deduplication, cache-first reads, explicit revalidation, replacement, and
stale-while-revalidate. Each request key has latest-result custody: an older response may resolve to
its caller, but cannot overwrite a newer cache entry. `replace` aborts the previous request, while
`cancel` exposes route- or scope-owned cancellation. Optimistic updates return a guarded rollback;
the rollback refuses to overwrite data that a later request has already committed.

```ts
const rollback = data.optimistic(articleResource, { id: "intro" }, (article) => ({
  ...article,
  title: "Draft title",
}));

const stale = await data.load(
  articleResource,
  { id: "intro" },
  {
    policy: "stale-while-revalidate",
  },
);
```

## Runtime adapters

`createFetchDataResource` adapts typed HTTP endpoints, `createStreamDataResource` reduces an
abort-aware async iterable, and `hydrateDataClientFromStream` restores incremental SSR snapshots.
None of them reads browser globals during module evaluation. `createDataTestAdapter` supplies a
deterministic response queue and records every load context for race, retry, and cancellation tests.
Resources can return an explicit unsupported reason from `supports` before their loader runs.
