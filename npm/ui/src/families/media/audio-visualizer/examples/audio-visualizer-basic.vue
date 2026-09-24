<!-- Frequency bars for a native audio element; the analyser stays idle until the listener opts in, so nothing connects to Web Audio during setup or SSR. -->
<script setup lang="ts">
import { shallowRef, useTemplateRef } from "vue";

import { AudioVisualizer, AudioVisualizerBars } from "../audio-visualizer.ts";

const audio = useTemplateRef<HTMLAudioElement>("audio");
const source = shallowRef<HTMLAudioElement | null>(null);

function visualize(): void {
  source.value = audio.value;
}
</script>

<template>
  <div>
    <audio ref="audio" src="/media/episode-42.mp3" preload="none" controls>
      <track kind="captions" src="/media/episode-42.vtt" srclang="en" label="English" />
    </audio>
    <button type="button" :disabled="source !== null" @click="visualize">Show visualizer</button>
    <AudioVisualizer v-slot="{ state }" :source aria-label="Episode 42 frequency spectrum">
      <AudioVisualizerBars :count="16" />
      <output>{{ state }}</output>
    </AudioVisualizer>
  </div>
</template>
