<script setup lang="ts">
import { ref } from "vue";
import { useSegmenter } from "@vizejs/composable/use-segmenter";

const text = ref("Ship it 🚀👩‍👩‍👧");
const { graphemeCount, wordCount, truncate } = useSegmenter(text, { granularity: "grapheme" });
</script>

<template>
  <div class="composable-example">
    <p>Count what a reader sees as characters, so emoji count once.</p>
    <label>
      Status update
      <input v-model="text" />
    </label>
    <output aria-live="polite">{{ graphemeCount }} characters · {{ wordCount }} words</output>
    <p>Preview: {{ truncate(10) }}</p>
  </div>
</template>
