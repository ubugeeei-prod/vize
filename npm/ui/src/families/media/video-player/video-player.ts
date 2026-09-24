/**
 * Headless video player: the shared MediaPlayer controls under `VideoPlayer*`
 * names plus the native `<video>` part, fullscreen, and picture-in-picture.
 * Shared types and helpers are exported from `@vizejs/ui/media-player`.
 */
export { default as VideoPlayerCaptionsButton } from "../media-player/media-player-captions-button.vue";
export { default as VideoPlayerLoadingIndicator } from "../media-player/media-player-loading-indicator.vue";
export { default as VideoPlayerMuteButton } from "../media-player/media-player-mute-button.vue";
export { default as VideoPlayerPlayButton } from "../media-player/media-player-play-button.vue";
export { default as VideoPlayerPlaybackRateButton } from "../media-player/media-player-playback-rate-button.vue";
export {
  default as VideoPlayer,
  default as VideoPlayerRoot,
} from "../media-player/media-player-root.vue";
export { default as VideoPlayerSeekSlider } from "../media-player/media-player-seek-slider.vue";
export { default as VideoPlayerTimeDisplay } from "../media-player/media-player-time-display.vue";
export { default as VideoPlayerVolumeSlider } from "../media-player/media-player-volume-slider.vue";
export { default as VideoPlayerFullscreenButton } from "./video-player-fullscreen-button.vue";
export { default as VideoPlayerPictureInPictureButton } from "./video-player-picture-in-picture-button.vue";
export { default as VideoPlayerVideo } from "./video-player-video.vue";
export type { VideoPlayerUnsupportedBehavior } from "./video-player-types.ts";
