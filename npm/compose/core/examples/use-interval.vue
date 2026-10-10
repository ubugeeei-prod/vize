<script setup lang="ts">
import { useInterval } from "@vizejs/composable/use-interval";

const { counter, isActive, pause, resume, reset } = useInterval(1000, {
  controls: true,
  immediate: false,
});
</script>

<template>
  <div class="composable-example">
    <p>Keep track of time spent reviewing a draft.</p>
    <output aria-live="polite"
      >{{ counter }} {{ counter === 1 ? "second" : "seconds" }} of review time</output
    >
    <p>{{ isActive ? "Review timer is running." : "Review timer is paused." }}</p>
    <div class="example-actions">
      <button type="button" :disabled="isActive" @click="resume">Start review timer</button>
      <button type="button" :disabled="!isActive" @click="pause">Pause review timer</button>
      <button type="button" @click="reset">Reset review time</button>
    </div>
  </div>
</template>
