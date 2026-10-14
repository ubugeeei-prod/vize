<script setup lang="ts">
import { computed, reactive } from "vue";
import { oneOf, useRouteQuery } from "@vizejs/composable/use-route-query";

// In an app, pass `useRoute()` and navigate with `router.replace({ query })`.
const route = reactive({ params: {}, query: {} as Record<string, string> });
const queryText = computed(() => new URLSearchParams(route.query).toString() || "(none)");
const tab = useRouteQuery(route, "tab", {
  parse: oneOf(["overview", "pricing", "faq"] as const),
  default: "overview",
  navigate: (query) => {
    route.query = { ...(query as Record<string, string>) };
  },
});
</script>

<template>
  <div class="composable-example">
    <p>Keep the selected tab in the URL query so the view can be shared and restored.</p>
    <div role="tablist" aria-label="Product sections">
      <button
        v-for="name in ['overview', 'pricing', 'faq'] as const"
        :key="name"
        role="tab"
        type="button"
        :aria-selected="tab === name"
        @click="() => (tab = name)"
      >
        {{ name }}
      </button>
    </div>
    <output aria-live="polite">Query: {{ queryText }}</output>
  </div>
</template>
