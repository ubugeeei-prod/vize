<script setup lang="ts">
import { defineDataResource } from "../src/index.ts";

export const articleResource = defineDataResource({
  key: "article.byId",
  source: "./ArticleView.vue",
  async loader({ input }: { input: { id: string } }) {
    return {
      id: input.id,
      title: `Article ${input.id}`,
    };
  },
});
</script>

<template>
  <article>{{ articleResource.key }}</article>
</template>
