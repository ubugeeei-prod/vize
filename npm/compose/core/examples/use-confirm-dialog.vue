<script setup lang="ts">
import { ref } from "vue";
import { useConfirmDialog } from "@vizejs/composable/use-confirm-dialog";

const { isRevealed, reveal, confirm, cancel } = useConfirmDialog();
const message = ref("The draft is saved.");

async function deleteDraft() {
  const { isCanceled } = await reveal();
  message.value = isCanceled ? "Kept the draft." : "Deleted the draft.";
}
</script>

<template>
  <div class="composable-example">
    <p>Ask before a destructive action and continue with the reader's answer.</p>
    <button type="button" :disabled="isRevealed" @click="deleteDraft">Delete draft</button>
    <div v-if="isRevealed" role="alertdialog" aria-label="Delete the draft?">
      <p>Delete the draft? This cannot be undone.</p>
      <div class="example-actions">
        <button type="button" @click="() => confirm()">Delete</button>
        <button type="button" @click="() => cancel()">Keep draft</button>
      </div>
    </div>
    <output aria-live="polite">{{ message }}</output>
  </div>
</template>
