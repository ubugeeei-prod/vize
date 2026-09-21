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
