export const audioVisualizerRendererFixtures = [
  {
    filename: "AudioVisualizerConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { useTemplateRef } from "vue";
import {
  AudioVisualizer,
  AudioVisualizerBars,
} from "./families/media/audio-visualizer/audio-visualizer.ts";

const player = useTemplateRef<HTMLAudioElement>("player");
</script>

<template>
  <audio ref="player" src="/podcast.mp3" controls>
    <track kind="captions" src="/podcast.vtt" srclang="en" label="English" />
  </audio>
  <AudioVisualizer :source="player" :fft-size="256" aria-label="Playback spectrum">
    <template #default="{ state, waveformPath }">
      <svg viewBox="0 0 100 20" :data-state="state">
        <path :d="waveformPath(100, 20)" />
      </svg>
      <AudioVisualizerBars :count="16" scale="log" />
    </template>
  </AudioVisualizer>
</template>
`,
  },
] as const;
