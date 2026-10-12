<script setup lang="ts">
import { ref, useId } from "vue";
import { useThrottled } from "@vizejs/composable/use-throttled";

const draft = ref("Vize release notes");
const titleId = useId();
const { throttled, pending, flush, cancel } = useThrottled(draft, 800);
const actionMessage = ref("Compare the draft and its preview.");

function showLatest(): void {
  if (flush()) actionMessage.value = "Newest draft shown now.";
}

function cancelWaiting(): void {
  if (cancel()) actionMessage.value = "Waiting preview update canceled; draft kept.";
}
</script>

<template>
  <div class="composable-example">
    <label :for="titleId">Draft release title</label>
    <input :id="titleId" v-model="draft" type="text" />
    <p>
      The first edit updates the preview immediately. Further edits in the same 800 ms window wait.
    </p>
    <p>Release title preview</p>
    <output aria-label="Release title preview" aria-live="polite">{{ throttled }}</output>
    <p role="status" aria-label="Preview update status">
      {{ pending ? "Newest draft is waiting." : "No update is waiting." }}
    </p>
    <div class="example-actions">
      <button type="button" :disabled="!pending" @click="showLatest">Update preview now</button>
      <button type="button" :disabled="!pending" @click="cancelWaiting">
        Cancel waiting update
      </button>
    </div>
    <p aria-live="polite">{{ actionMessage }}</p>
  </div>
</template>
