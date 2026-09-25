<!-- Feed that pages in more posts from a sentinel or a load-more button, with a live status. -->
<script setup lang="ts">
import { computed, ref } from "vue";

import {
  InfiniteScrollItem,
  InfiniteScrollLoadMore,
  InfiniteScrollRoot,
  InfiniteScrollSentinel,
  InfiniteScrollStatus,
} from "../infinite-scroll.ts";

const total = 12;
const pageSize = 4;
const posts = ref(Array.from({ length: pageSize }, (_, index) => `Release note #${index + 1}`));
const hasMore = computed(() => posts.value.length < total);

async function loadPage(): Promise<void> {
  const start = posts.value.length;
  const next = Array.from({ length: pageSize }, (_, index) => `Release note #${start + index + 1}`);
  posts.value = [...posts.value, ...next];
}
</script>

<template>
  <InfiniteScrollRoot aria-label="Release notes" feed :total :has-more :loader="loadPage">
    <InfiniteScrollItem v-for="(post, index) in posts" :key="post" :index>
      {{ post }}
    </InfiniteScrollItem>
    <InfiniteScrollSentinel />
    <InfiniteScrollLoadMore>Load more notes</InfiniteScrollLoadMore>
    <InfiniteScrollStatus v-slot="{ state }">
      {{ state === "loading" ? "Loading notes…" : state === "complete" ? "All notes loaded" : "" }}
    </InfiniteScrollStatus>
  </InfiniteScrollRoot>
</template>
