<script setup lang="ts">
import { useFileDialog } from "@vizejs/composable/use-file-dialog";

const { files, open, reset } = useFileDialog({ accept: ".csv,text/csv", multiple: false });
</script>

<template>
  <div class="composable-example">
    <p>Open the file picker from your own button and show the chosen file.</p>
    <div class="example-actions">
      <button type="button" @click="() => open()">Choose a CSV file</button>
      <button type="button" :disabled="!files" @click="() => reset()">Clear</button>
    </div>
    <output aria-live="polite">{{
      files?.[0] ? `Selected ${files[0].name} (${files[0].size} bytes)` : "No file selected."
    }}</output>
  </div>
</template>
