<!-- Shared player controls; they stay disabled until an AudioPlayer or VideoPlayer media part registers. -->
<script setup lang="ts">
import {
  MediaPlayer,
  MediaPlayerMuteButton,
  MediaPlayerPlayButton,
  MediaPlayerPlaybackRateButton,
  MediaPlayerSeekSlider,
  MediaPlayerTimeDisplay,
  MediaPlayerVolumeSlider,
} from "../media-player.ts";
</script>

<template>
  <MediaPlayer aria-label="Lecture recording" :default-volume="0.8">
    <MediaPlayerPlayButton v-slot="{ paused }">{{
      paused ? "Play" : "Pause"
    }}</MediaPlayerPlayButton>
    <MediaPlayerTimeDisplay mode="current" />
    <MediaPlayerSeekSlider />
    <MediaPlayerTimeDisplay mode="duration" />
    <MediaPlayerMuteButton v-slot="{ muted }">{{
      muted ? "Unmute" : "Mute"
    }}</MediaPlayerMuteButton>
    <MediaPlayerVolumeSlider />
    <MediaPlayerPlaybackRateButton v-slot="{ playbackRate }"
      >{{ playbackRate }}×</MediaPlayerPlaybackRateButton
    >
  </MediaPlayer>
</template>
