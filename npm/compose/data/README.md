# @vizejs/data

Typed SSR data resource foundations for Vize applications.

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
const article = await data.load(articleResource, { id: "intro" });
</script>

<template>
  <article v-if="article.status === 'success'">
    {{ article.data.title }}
  </article>
</template>
```

Server renderers can call `serializeDataSnapshot(data)` and hydrate the client with
`hydrateDataClient(snapshot)`. Hydrated success entries satisfy the first client load without
issuing duplicate network work.
