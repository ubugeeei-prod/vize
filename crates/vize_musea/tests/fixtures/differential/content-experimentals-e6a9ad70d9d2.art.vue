<script setup lang="ts">
type Entry =
  | { kind: "article"; data: { title: string; published: boolean } }
  | { kind: "draft"; reason: string }
  | { kind: "archived"; id: string };

const entry = ref<Entry>({ kind: "draft", reason: "editing" });
</script>

<template v-match="entry">
  <article v-when="{ kind: 'article', data: const article } if (article.published)">
    {{ article.title }}
  </article>
  <p v-when="{ kind: 'draft' } | { kind: 'archived' }">Hidden</p>
  <p v-when="_">No published article</p>
</template>
