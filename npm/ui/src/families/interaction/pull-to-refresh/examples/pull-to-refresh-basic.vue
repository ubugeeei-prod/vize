<!-- Scrollable inbox that reloads by pulling down or with the keyboard-accessible refresh button. -->
<script setup lang="ts">
import { ref } from "vue";

import { PullToRefresh, PullToRefreshTrigger } from "../pull-to-refresh.ts";

const messages = ref(["Welcome to your inbox", "Your order has shipped", "Weekly digest"]);
const refreshCount = ref(0);
// The root is the scroll container, so it needs a bounded block size to scroll.
const scrollerSize = { blockSize: "12rem", overflow: "auto" };

async function loadNewMessages(): Promise<void> {
  await Promise.resolve();
  refreshCount.value += 1;
  messages.value = [`New message ${refreshCount.value}`, ...messages.value];
}
</script>

<template>
  <PullToRefresh
    v-slot="{ state, refreshing }"
    :refresh-action="loadNewMessages"
    :style="scrollerSize"
  >
    <p role="status">
      {{ refreshing ? "Refreshing…" : state === "armed" ? "Release to refresh" : "" }}
    </p>
    <PullToRefreshTrigger v-slot="{ refreshing: busy }">
      {{ busy ? "Refreshing" : "Refresh inbox" }}
    </PullToRefreshTrigger>
    <ul aria-label="Messages">
      <li v-for="message in messages" :key="message">{{ message }}</li>
    </ul>
  </PullToRefresh>
</template>
