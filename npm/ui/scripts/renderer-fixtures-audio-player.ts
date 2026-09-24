export const audioPlayerRendererFixtures = [
  {
    filename: "AudioPlayerConsumer.vue",
    source: String.raw`<script setup lang="ts">
import {
  AudioPlayerAudio,
  AudioPlayerPlayButton,
  AudioPlayerRoot,
  AudioPlayerSeekSlider,
  AudioPlayerTimeDisplay,
} from "./families/media/audio-player/audio-player.ts";
</script>

<template>
  <AudioPlayerRoot aria-labelledby="track-title">
    <template #default="{ paused, duration }">
      <h3 id="track-title">Nocturne</h3>
      <AudioPlayerAudio src="/nocturne.ogg" preload="none" />
      <AudioPlayerPlayButton :aria-label="null">{{ paused ? "Play" : "Pause" }}</AudioPlayerPlayButton>
      <AudioPlayerSeekSlider />
      <AudioPlayerTimeDisplay mode="duration" />
      <span>{{ duration }}</span>
    </template>
  </AudioPlayerRoot>
</template>
`,
  },
] as const;
