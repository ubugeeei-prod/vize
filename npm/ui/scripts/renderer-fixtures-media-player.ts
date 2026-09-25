export const mediaPlayerRendererFixtures = [
  {
    filename: "MediaPlayerConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import AudioPlayerAudio from "./families/media/audio-player/audio-player-audio.vue";
import {
  MediaPlayerCaptionsButton,
  MediaPlayerLoadingIndicator,
  MediaPlayerMuteButton,
  MediaPlayerPlayButton,
  MediaPlayerPlaybackRateButton,
  MediaPlayerRoot,
  MediaPlayerSeekSlider,
  MediaPlayerTimeDisplay,
  MediaPlayerVolumeSlider,
} from "./families/media/media-player/media-player.ts";

const volume = ref(0.8);
const muted = ref(false);
const rate = ref(1);
</script>

<template>
  <MediaPlayerRoot
    v-model:volume="volume"
    v-model:muted="muted"
    v-model:playback-rate="rate"
    aria-label="Episode player"
    :messages="{ play: 'Play episode' }"
  >
    <template #default="{ state }">
      <AudioPlayerAudio src="/episode.mp3" />
      <MediaPlayerPlayButton>{{ state }}</MediaPlayerPlayButton>
      <MediaPlayerMuteButton />
      <MediaPlayerSeekSlider />
      <MediaPlayerVolumeSlider />
      <MediaPlayerTimeDisplay mode="remaining" />
      <MediaPlayerPlaybackRateButton :rates="[1, 1.5, 2]" />
      <MediaPlayerCaptionsButton />
      <MediaPlayerLoadingIndicator />
    </template>
  </MediaPlayerRoot>
</template>
`,
  },
] as const;
