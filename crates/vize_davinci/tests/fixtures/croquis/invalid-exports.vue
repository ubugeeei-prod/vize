<script setup lang="ts">
import { ref, computed } from "vue";

export interface RowState {
  id: string;
  selected: boolean;
}

// Value exports are invalid inside <script setup>; croquis records them
// under [errors].
export const rows = ref<RowState[]>([]);
export function clearRows() {
  rows.value = [];
}

const selectedCount = computed(() => rows.value.filter((r) => r.selected).length);
</script>

<template>
  <p>{{ selectedCount }} selected</p>
  <button @click="clearRows">clear</button>
</template>
