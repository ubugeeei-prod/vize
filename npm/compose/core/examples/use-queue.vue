<script setup lang="ts">
import { ref } from "vue";
import { useQueue } from "@vizejs/composable/use-queue";

const { items, isEmpty, enqueue, dequeue } = useQueue<string>(["Invoice", "Ticket"]);
const printed = ref("Nothing printed yet.");
let next = 1;

function addJob() {
  enqueue(`Report ${next}`);
  next += 1;
}

function printNext() {
  const job = dequeue();
  if (job !== undefined) printed.value = `Printed ${job}.`;
}
</script>

<template>
  <div class="composable-example">
    <p>Send documents to a printer in the order they arrive.</p>
    <div class="example-actions">
      <button type="button" @click="addJob">Add print job</button>
      <button type="button" :disabled="isEmpty" @click="printNext">Print next</button>
    </div>
    <ol aria-label="Print queue">
      <li v-for="(job, index) in items" :key="`${index}-${job}`">{{ job }}</li>
    </ol>
    <output aria-live="polite">{{ printed }}</output>
  </div>
</template>
