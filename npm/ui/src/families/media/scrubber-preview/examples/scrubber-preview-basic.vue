<!-- Seek bar that previews sprite-sheet thumbnails and the hovered time, seeking on release. -->
<script setup lang="ts">
import { ref, useId } from "vue";

import {
  ScrubberPreview,
  ScrubberPreviewThumbnail,
  ScrubberPreviewTime,
  ScrubberPreviewTrack,
} from "../scrubber-preview.ts";
import type { ScrubberPreviewSprite } from "../scrubber-preview.ts";

const duration = 180;
const currentTime = ref(0);
const seekId = useId();
const sprite: ScrubberPreviewSprite = {
  src: "/media/trailer-thumbnails.jpg",
  columns: 6,
  rows: 6,
  interval: 5,
  width: 160,
  height: 90,
};

function seek(time: number): void {
  currentTime.value = Math.round(time);
}
</script>

<template>
  <ScrubberPreview :duration :sprite @seek="seek">
    <ScrubberPreviewTrack>
      <label :for="seekId">Seek</label>
      <input
        :id="seekId"
        v-model.number="currentTime"
        type="range"
        min="0"
        :max="duration"
        step="1"
      />
    </ScrubberPreviewTrack>
    <ScrubberPreviewThumbnail />
    <ScrubberPreviewTime />
  </ScrubberPreview>
  <output>{{ currentTime }} s</output>
</template>
