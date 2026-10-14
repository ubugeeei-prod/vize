<script setup lang="ts">
import { ref } from "vue";
import { usePerformanceObserver } from "@vizejs/composable/use-performance-observer";

const requests = ref<string[]>([]);
usePerformanceObserver("resource", (entries) => {
  for (const entry of entries) {
    if (entry.initiatorType !== "fetch") continue;
    const url = new URL(entry.name);
    requests.value = [...requests.value, `${url.search} in ${Math.round(entry.duration)} ms`];
  }
});
let count = 0;

async function request() {
  count += 1;
  // Stands in for an API call; any same-origin request produces a resource entry.
  await fetch(`${location.pathname}?request=${count}`);
}
</script>

<template>
  <div class="composable-example">
    <p>Watch how long the page's requests take, as the browser records them.</p>
    <button type="button" @click="request">Send a request</button>
    <ul aria-label="Recorded requests">
      <li v-for="item in requests" :key="item">{{ item }}</li>
    </ul>
    <output aria-live="polite">{{ requests.length }} requests recorded</output>
  </div>
</template>
