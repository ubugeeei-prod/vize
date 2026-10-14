<script setup lang="ts">
import { useBroadcastChannel } from "@vizejs/composable/use-broadcast-channel";

// Two channels with the same name stand in for two tabs of your app.
const editor = useBroadcastChannel<string>("docs-example-sync");
const otherTab = useBroadcastChannel<string>("docs-example-sync");

function publish() {
  editor.post("Draft saved at revision 3");
}
</script>

<template>
  <div class="composable-example">
    <p>Tell other open tabs that a draft changed.</p>
    <button type="button" @click="publish">Save in this tab</button>
    <output aria-live="polite"
      >Other tab received: {{ otherTab.data.value ?? "nothing yet" }}</output
    >
  </div>
</template>
