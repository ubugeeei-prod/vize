<script setup lang="ts">
import { computed } from "vue";
import { useNow } from "@vizejs/composable/use-now";

const { now, pause, resume, isActive } = useNow({ controls: true, interval: 1000, initial: 0 });
const clock = computed(() => now.value.toISOString().slice(11, 19));
</script>

<template>
  <div class="composable-example">
    <p>Show a live clock that updates every second and can be paused.</p>
    <output aria-live="off">{{ clock }} UTC</output>
    <button type="button" @click="() => (isActive ? pause() : resume())">
      {{ isActive ? "Pause clock" : "Resume clock" }}
    </button>
  </div>
</template>
