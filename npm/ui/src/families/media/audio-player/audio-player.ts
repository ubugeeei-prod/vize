/**
 * Headless audio player: the shared MediaPlayer controls under `AudioPlayer*`
 * names plus the native `<audio>` part. Shared types and helpers are exported
 * from `@vizejs/ui/media-player`.
 */
export { default as AudioPlayerAudio } from "./audio-player-audio.vue";
export { default as AudioPlayerCaptionsButton } from "../media-player/media-player-captions-button.vue";
export { default as AudioPlayerLoadingIndicator } from "../media-player/media-player-loading-indicator.vue";
export { default as AudioPlayerMuteButton } from "../media-player/media-player-mute-button.vue";
export { default as AudioPlayerPlayButton } from "../media-player/media-player-play-button.vue";
export { default as AudioPlayerPlaybackRateButton } from "../media-player/media-player-playback-rate-button.vue";
export {
  default as AudioPlayer,
  default as AudioPlayerRoot,
} from "../media-player/media-player-root.vue";
export { default as AudioPlayerSeekSlider } from "../media-player/media-player-seek-slider.vue";
export { default as AudioPlayerTimeDisplay } from "../media-player/media-player-time-display.vue";
export { default as AudioPlayerVolumeSlider } from "../media-player/media-player-volume-slider.vue";
