<script setup lang="ts">
import { ref } from "vue";
import { useRequestIdleCallback } from "@vizejs/composable/use-request-idle-callback";

const queued = ref(0);
const sent = ref(0);
const { isPending, start } = useRequestIdleCallback(
  () => {
    sent.value += queued.value;
    queued.value = 0;
  },
  { immediate: false },
);

function track() {
  queued.value += 1;
  start();
}
</script>

<template>
  <div class="composable-example">
    <p>Send analytics events when the browser is idle instead of during interactions.</p>
    <button type="button" @click="track">Record a click</button>
    <output aria-live="polite"
      >{{ queued }} waiting · {{ sent }} sent{{ isPending ? " · flush scheduled" : "" }}</output
    >
  </div>
</template>
