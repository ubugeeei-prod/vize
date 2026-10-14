<script setup lang="ts">
import { ref } from "vue";
import { useWebLocks } from "@vizejs/composable/use-web-locks";

const { request, held } = useWebLocks();
const message = ref("Idle.");
let finish: (() => void) | undefined;

function startExport() {
  void request("export", () => new Promise<void>((resolve) => (finish = resolve)));
  message.value = "Export running in this tab.";
}

async function tryExport() {
  const attempt = await request("export", () => "second export", { ifAvailable: true });
  message.value = attempt.acquired
    ? "Started another export."
    : "Another export is already running.";
}

function stopExport() {
  finish?.();
  message.value = "Export finished.";
}
</script>

<template>
  <div class="composable-example">
    <p>Make sure only one tab runs an export at a time.</p>
    <div class="example-actions">
      <button type="button" @click="startExport">Start export</button>
      <button type="button" @click="tryExport">Try another export</button>
      <button type="button" @click="stopExport">Finish export</button>
    </div>
    <output aria-live="polite">{{ message }} Locks held: {{ held.join(", ") || "none" }}</output>
  </div>
</template>
