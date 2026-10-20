<script lang="ts">
import { ref } from "vue";

export function setupPanel() {
  const isOpen = ref(false);

  function open(): void {
    isOpen.value = true;
  }

  return { isOpen, open };
}
</script>

<script setup lang="ts">
defineProps<{ open: boolean }>();
</script>

<template>
  <aside v-if="open">panel</aside>
</template>
