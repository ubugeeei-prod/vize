<script setup lang="ts">
import { ref } from "vue";
import { useTimeoutFn } from "@vizejs/composable/use-timeout";

const reminders = ref(0);
const message = ref("No reminder scheduled.");
const { isPending, start, stop } = useTimeoutFn(
  () => {
    reminders.value += 1;
    message.value = "Reminder delivered.";
  },
  1000,
  { immediate: false },
);

function scheduleReminder(): void {
  message.value = "Reminder scheduled for one second from now.";
  start();
}

function cancelReminder(): void {
  if (stop()) message.value = "Reminder canceled.";
}
</script>

<template>
  <div class="composable-example">
    <p>Schedule a local review reminder, or cancel it before it arrives.</p>
    <output aria-live="polite">{{ message }} Delivered: {{ reminders }}</output>
    <div class="example-actions">
      <button type="button" @click="scheduleReminder">Schedule reminder</button>
      <button type="button" :disabled="!isPending" @click="cancelReminder">Cancel reminder</button>
    </div>
    <p>{{ isPending ? "The reminder is pending." : "There is no pending reminder." }}</p>
  </div>
</template>
