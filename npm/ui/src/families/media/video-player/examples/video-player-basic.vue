<!-- Video player wiring a native video with captions to play, seek, volume, captions, and fullscreen controls. -->
<script setup lang="ts">
import {
  VideoPlayer,
  VideoPlayerCaptionsButton,
  VideoPlayerFullscreenButton,
  VideoPlayerMuteButton,
  VideoPlayerPictureInPictureButton,
  VideoPlayerPlayButton,
  VideoPlayerSeekSlider,
  VideoPlayerTimeDisplay,
  VideoPlayerVideo,
  VideoPlayerVolumeSlider,
} from "../video-player.ts";
</script>

<template>
  <VideoPlayer aria-label="Product tour">
    <VideoPlayerVideo
      src="/media/product-tour.mp4"
      poster="/media/product-tour.jpg"
      preload="none"
      :width="640"
      :height="360"
    >
      <track kind="captions" src="/media/product-tour.en.vtt" srclang="en" label="English" />
    </VideoPlayerVideo>
    <VideoPlayerPlayButton v-slot="{ paused }">{{
      paused ? "Play" : "Pause"
    }}</VideoPlayerPlayButton>
    <VideoPlayerTimeDisplay mode="current" />
    <VideoPlayerSeekSlider />
    <VideoPlayerTimeDisplay mode="duration" />
    <VideoPlayerMuteButton v-slot="{ muted }">{{
      muted ? "Unmute" : "Mute"
    }}</VideoPlayerMuteButton>
    <VideoPlayerVolumeSlider />
    <VideoPlayerCaptionsButton>Captions</VideoPlayerCaptionsButton>
    <VideoPlayerPictureInPictureButton unsupported="hide"
      >Picture-in-picture</VideoPlayerPictureInPictureButton
    >
    <VideoPlayerFullscreenButton v-slot="{ fullscreen }">{{
      fullscreen ? "Exit fullscreen" : "Fullscreen"
    }}</VideoPlayerFullscreenButton>
  </VideoPlayer>
</template>
