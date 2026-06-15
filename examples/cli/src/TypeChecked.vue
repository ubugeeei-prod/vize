<script setup lang="ts">
import { computed } from "vue";

type Status = "idle" | "loading" | "ready";

const props = withDefaults(
  defineProps<{
    label: string;
    count?: number;
    status?: Status;
  }>(),
  {
    count: 0,
    status: "idle",
  },
);

const emit = defineEmits<{
  refresh: [nextStatus: Status];
}>();

const badge = computed(() => `${props.label}: ${props.count}`);

function refresh() {
  emit("refresh", "loading");
}
</script>

<template>
  <section class="type-checked">
    <p>{{ badge }}</p>
    <button type="button" :disabled="status === 'loading'" @click="refresh">Refresh</button>
  </section>
</template>

<style scoped>
.type-checked {
  display: grid;
  gap: 8px;
}
</style>
