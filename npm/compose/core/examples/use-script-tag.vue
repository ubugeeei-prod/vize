<script setup lang="ts">
import { useScriptTag } from "@vizejs/composable/use-script-tag";

// Point this at a real third-party widget in your app.
const widget =
  "data:text/javascript,window.exampleWidget%20%3D%20%7B%20version%3A%20%222.1%22%20%7D%3B";
const { status, load, unload } = useScriptTag(widget, { immediate: false });
</script>

<template>
  <div class="composable-example">
    <p>Load a third-party script only when the reader asks for the widget.</p>
    <div class="example-actions">
      <button type="button" :disabled="status === 'loaded'" @click="() => load()">
        Load widget
      </button>
      <button type="button" :disabled="status !== 'loaded'" @click="() => unload()">
        Remove widget
      </button>
    </div>
    <output aria-live="polite">Script status: {{ status }}</output>
  </div>
</template>
