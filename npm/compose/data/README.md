# @vizejs/data

Typed SSR data resource foundations for Vize applications.

> This experimental package is currently workspace-only and has not been published to npm. The
> example below requires the Vize repository workspace.

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
