<!-- Metrics dashboard whose widgets can be dragged and resized on a twelve-column grid. -->
<script setup lang="ts">
import { ref } from "vue";

import { DashboardGrid, DashboardGridItem } from "../dashboard-grid.ts";
import type { DashboardLayout } from "../dashboard-grid.ts";

const layout = ref<DashboardLayout>([
  { id: "revenue", x: 0, y: 0, w: 6, h: 2 },
  { id: "signups", x: 6, y: 0, w: 3, h: 2 },
  { id: "churn", x: 9, y: 0, w: 3, h: 2 },
]);
const titles: ReadonlyMap<string, string> = new Map([
  ["revenue", "Revenue"],
  ["signups", "Sign-ups"],
  ["churn", "Churn"],
]);

function titleOf(id: string): string {
  return titles.get(id) ?? id;
}
</script>

<template>
  <DashboardGrid v-model:layout="layout" :columns="12" :row-height="48" label="Metrics">
    <DashboardGridItem
      v-for="item in layout"
      :id="item.id"
      :key="item.id"
      :label="titleOf(item.id)"
    >
      <template #default="{ handleProps, resizeHandleProps }">
        <h3 v-bind="handleProps">{{ titleOf(item.id) }}</h3>
        <p>{{ item.w }} × {{ item.h }} cells</p>
        <span v-bind="resizeHandleProps">⤡</span>
      </template>
    </DashboardGridItem>
  </DashboardGrid>
</template>
