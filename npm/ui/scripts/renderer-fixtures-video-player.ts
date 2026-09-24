export const videoPlayerRendererFixtures = [
  {
    filename: "VideoPlayerConsumer.vue",
    source: String.raw`<script setup lang="ts">
import {
  VideoPlayerCaptionsButton,
  VideoPlayerFullscreenButton,
  VideoPlayerPictureInPictureButton,
  VideoPlayerPlayButton,
  VideoPlayerRoot,
  VideoPlayerSeekSlider,
  VideoPlayerTimeDisplay,
  VideoPlayerVideo,
  VideoPlayerVolumeSlider,
} from "./families/media/video-player/video-player.ts";

function onEnded(): void {
  void 0;
}
</script>

<template>
  <VideoPlayerRoot aria-label="Product tour" default-muted @ended="onEnded">
    <VideoPlayerVideo src="/tour.mp4" poster="/tour.jpg" autoplay>
      <track kind="captions" src="/tour.en.vtt" srclang="en" label="English" default />
    </VideoPlayerVideo>
    <VideoPlayerPlayButton />
    <VideoPlayerSeekSlider />
    <VideoPlayerTimeDisplay />
    <VideoPlayerVolumeSlider />
    <VideoPlayerCaptionsButton />
    <VideoPlayerPictureInPictureButton unsupported="hide" />
    <VideoPlayerFullscreenButton />
  </VideoPlayerRoot>
</template>
`,
  },
] as const;
