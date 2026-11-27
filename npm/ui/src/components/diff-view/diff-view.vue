<script setup lang="ts">
import { computed } from "vue";

import { keyDiffLines, parseUnifiedDiff } from "./diff-parse.ts";
import type { DiffViewProps } from "./types.ts";

const { label = "Diff", patch = "", lines = undefined } = defineProps<DiffViewProps>();

const rendered = computed(() =>
  lines === undefined ? parseUnifiedDiff(patch) : keyDiffLines(lines),
);
</script>

<template>
  <table data-vize-ui="diff-view" role="table" :aria-label="label">
    <tbody>
      <tr v-for="line in rendered" :key="line.key" role="row" :data-kind="line.kind">
        <td role="cell">{{ line.text }}</td>
      </tr>
    </tbody>
  </table>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
