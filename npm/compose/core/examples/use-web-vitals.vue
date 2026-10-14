<script setup lang="ts">
import { computed } from "vue";
import { useMounted } from "@vizejs/composable/use-mounted";
import { useWebVitals } from "@vizejs/composable/use-web-vitals";

const { metrics, supported } = useWebVitals({ reportAllChanges: true });
const mounted = useMounted();
const rows = computed(() =>
  Object.values(metrics.value).map(
    (metric) => `${metric?.name} ${Math.round(metric?.value ?? 0)} (${metric?.rating})`,
  ),
);
</script>

<template>
  <div class="composable-example">
    <p>Report Core Web Vitals from real visits to your analytics.</p>
    <ul aria-label="Web vitals">
      <li v-for="row in rows" :key="row">{{ row }}</li>
    </ul>
    <output aria-live="polite">{{
      !mounted
        ? "Waiting for the first metric…"
        : !supported
          ? "This browser does not report Web Vitals."
          : rows.length === 0
            ? "Waiting for the first metric…"
            : `${rows.length} metrics reported`
    }}</output>
  </div>
</template>
