<script setup lang="ts">
import { ref } from "vue";
import { useMounted } from "@vizejs/composable/use-mounted";
import { useSchedulerPostTask } from "@vizejs/composable/use-scheduler-post-task";

const { postTask, supported } = useSchedulerPostTask();
const order = ref<string[]>([]);
const mounted = useMounted();

async function runTasks() {
  order.value = [];
  await Promise.all([
    postTask(() => order.value.push("prefetch next page"), { priority: "background" }),
    postTask(() => order.value.push("update the list"), { priority: "user-visible" }),
    postTask(() => order.value.push("respond to the click"), { priority: "user-blocking" }),
  ]);
}
</script>

<template>
  <div class="composable-example">
    <p>Schedule work by priority so the most urgent task runs first.</p>
    <button type="button" @click="runTasks">Queue three tasks</button>
    <ol aria-label="Run order">
      <li v-for="task in order" :key="task">{{ task }}</li>
    </ol>
    <output aria-live="polite">{{
      !mounted
        ? "Checking the scheduler…"
        : supported
          ? "Using the browser's task scheduler."
          : "Using a timer fallback."
    }}</output>
  </div>
</template>
