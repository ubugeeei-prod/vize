<script setup lang="ts">
import { computed, ref } from "vue";
import { useMounted } from "@vizejs/composable/use-mounted";
import { useObjectUrl } from "@vizejs/composable/use-object-url";

const notes = ref("Talk notes for Vue Fes Japan");
const file = computed(() => new Blob([notes.value], { type: "text/plain" }));
const url = useObjectUrl(file);
// Object URLs exist only in the browser; render the link after hydration.
const mounted = useMounted();
// Only object URLs are expected here; anything else falls back to a blank page.
const safeUrl = (value: string) => (value.startsWith("blob:") ? value : "about:blank");
</script>

<template>
  <div class="composable-example">
    <p>Offer text the reader typed as a downloadable file.</p>
    <label>
      Notes
      <textarea v-model="notes" rows="2"></textarea>
    </label>
    <a v-if="mounted && url" :href="safeUrl(url)" download="notes.txt">Download notes.txt</a>
    <output aria-live="polite">{{ file.size }} bytes ready to download</output>
  </div>
</template>
