<script setup lang="ts">
import { useStyleTag } from "@vizejs/composable/use-style-tag";

const { loaded, load, unload } = useStyleTag(".example-note { outline: 3px solid #f97316; }", {
  immediate: false,
});
</script>

<template>
  <div class="composable-example">
    <p>Add page-wide CSS only while a feature is on, and remove it when it is off.</p>
    <button type="button" @click="() => (loaded ? unload() : load())">
      {{ loaded ? "Remove highlight styles" : "Add highlight styles" }}
    </button>
    <p class="example-note">This note is outlined while the style tag is in the document.</p>
    <output aria-live="polite">{{ loaded ? "Style tag added." : "No style tag." }}</output>
  </div>
</template>
