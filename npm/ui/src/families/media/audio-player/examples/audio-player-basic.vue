<!-- Podcast player wiring a native audio element to play, seek, time, mute, and volume controls. -->
<script setup lang="ts">
import {
  AudioPlayer,
  AudioPlayerAudio,
  AudioPlayerMuteButton,
  AudioPlayerPlayButton,
  AudioPlayerSeekSlider,
  AudioPlayerTimeDisplay,
  AudioPlayerVolumeSlider,
} from "../audio-player.ts";
</script>

<template>
  <AudioPlayer aria-label="Episode 42: Designing headless components">
    <AudioPlayerAudio src="/media/episode-42.mp3" preload="none" />
    <AudioPlayerPlayButton v-slot="{ paused }">{{
      paused ? "Play" : "Pause"
    }}</AudioPlayerPlayButton>
    <AudioPlayerTimeDisplay mode="current" />
    <AudioPlayerSeekSlider />
    <AudioPlayerTimeDisplay mode="duration" />
    <AudioPlayerMuteButton v-slot="{ muted }">{{
      muted ? "Unmute" : "Mute"
    }}</AudioPlayerMuteButton>
    <AudioPlayerVolumeSlider />
  </AudioPlayer>
</template>
