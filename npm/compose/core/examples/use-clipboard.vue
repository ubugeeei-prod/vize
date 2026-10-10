<script setup lang="ts">
import { ref, useId } from "vue";
import { useClipboard } from "@vizejs/composable/use-clipboard";

const commandId = useId();
const command = ref("vp add @vizejs/composable vue");
const outcome = ref("No clipboard operation yet.");
const { supported, text, copied, error, readPermission, copy, readText } = useClipboard();

async function copyCommand(): Promise<void> {
  const result = await copy(command.value);
  outcome.value =
    result.status === "success"
      ? `Copied using ${result.method}.`
      : `Copy did not complete: ${result.status}.`;
}

async function readClipboard(): Promise<void> {
  const result = await readText();
  outcome.value =
    result.status === "success"
      ? "Read the clipboard text."
      : `Read did not complete: ${result.status}.`;
}
</script>

<template>
  <div class="composable-example">
    <label :for="commandId">Install command</label>
    <input :id="commandId" v-model="command" />
    <div class="example-actions">
      <button type="button" :disabled="!supported" @click="copyCommand">
        Copy install command
      </button>
      <button type="button" :disabled="!supported" @click="readClipboard">
        Read clipboard text
      </button>
    </div>
    <output aria-live="polite">{{ outcome }} Recently copied: {{ copied ? "yes" : "no" }}.</output>
    <p class="clipboard-result">
      Last successful text: <code>{{ text || "None yet." }}</code>
    </p>
    <p>
      Clipboard read permission: {{ readPermission }} · Last failure: {{ error?.status || "none" }}
    </p>
    <p>
      Copy writes the editable command to your clipboard. Read requests your clipboard text only
      when you activate its button. Permission failures keep the previous successful text.
      Recent-copy feedback expires after 1.5 seconds; reading does not restart it.
    </p>
    <p v-if="!supported">Clipboard support is not available in this context.</p>
  </div>
</template>

<style scoped>
.clipboard-result {
  overflow-wrap: anywhere;
}
</style>
