<script setup lang="ts">
import { useFileSystemAccess } from "@vizejs/composable/use-file-system-access";
import { useMounted } from "@vizejs/composable/use-mounted";

const mounted = useMounted();
const { supported, data, fileName, open, save } = useFileSystemAccess({
  dataType: "text",
  types: [{ description: "Text files", accept: { "text/plain": [".txt", ".md"] } }],
});
</script>

<template>
  <div class="composable-example">
    <p>Open a local text file, edit it, and save it back in place.</p>
    <div class="example-actions">
      <button type="button" :disabled="!mounted || !supported" @click="() => open()">
        Open file
      </button>
      <button type="button" :disabled="!fileName" @click="() => save()">Save</button>
    </div>
    <label v-if="fileName">
      {{ fileName }}
      <textarea v-model="data" rows="3"></textarea>
    </label>
    <output aria-live="polite">{{
      !mounted
        ? "Checking browser support…"
        : supported
          ? fileName
            ? `Editing ${fileName}`
            : "No file open."
          : "The File System Access API is not available; use a download instead."
    }}</output>
  </div>
</template>
