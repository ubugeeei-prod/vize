<script setup lang="ts">
import { ref } from "vue";
import { useCompressionStream } from "@vizejs/composable/use-compression-stream";

const { compress, decompressText, pending } = useCompressionStream({ format: "gzip" });
const text = ref("Vue ".repeat(200));
const summary = ref("Not compressed yet.");

async function roundTrip() {
  const bytes = await compress(text.value);
  const restored = await decompressText(bytes);
  summary.value = `${new TextEncoder().encode(text.value).length} bytes → ${bytes.length} bytes gzip · restored ${restored === text.value ? "exactly" : "with differences"}`;
}
</script>

<template>
  <div class="composable-example">
    <p>Compress text in the browser before uploading it, and check it restores exactly.</p>
    <label>
      Text to compress
      <textarea v-model="text" rows="2"></textarea>
    </label>
    <button type="button" :disabled="pending" @click="roundTrip">Compress and restore</button>
    <output aria-live="polite">{{ summary }}</output>
  </div>
</template>
