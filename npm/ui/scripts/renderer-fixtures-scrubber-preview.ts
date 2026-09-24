export const scrubberPreviewRendererFixtures = [
  {
    filename: "ScrubberPreviewConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import {
  ScrubberPreviewRoot,
  ScrubberPreviewThumbnail,
  ScrubberPreviewTime,
  ScrubberPreviewTrack,
} from "./families/media/scrubber-preview/scrubber-preview.ts";

const previewTime = ref<number | null>(null);
const sprite = { src: "/thumbs/sheet.jpg", columns: 10, rows: 10, interval: 5, width: 160, height: 90 };

function onSeek(time: number): void {
  void time;
}
</script>

<template>
  <ScrubberPreviewRoot v-model:time="previewTime" :duration="600" :sprite="sprite" @seek="onSeek">
    <template #default="{ active }">
      <ScrubberPreviewTrack>
        <input type="range" min="0" max="600" aria-label="Seek" />
      </ScrubberPreviewTrack>
      <ScrubberPreviewThumbnail>
        <template #default="{ status }">{{ active ? status : "" }}</template>
      </ScrubberPreviewThumbnail>
      <ScrubberPreviewTime />
    </template>
  </ScrubberPreviewRoot>
</template>
`,
  },
] as const;
