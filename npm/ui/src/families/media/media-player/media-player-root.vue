<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, shallowRef, useTemplateRef, watch } from "vue";

import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import {
  deriveDeterministicId,
  useDeterministicId,
} from "../../foundations/id/deterministic-id.ts";
import { mediaPlayerContext } from "./media-player-context.ts";
import type { MediaPlayerContextValue } from "./media-player-context.ts";
import {
  clampMediaValue,
  isCaptionTrack,
  isEditableTarget,
  normalizeMediaDuration,
  readTimeRanges,
  resolveMediaPlayerMessages,
  resolveMediaShortcut,
} from "./media-player-format.ts";
import {
  canRequestFullscreen,
  canRequestPictureInPicture,
  currentFullscreenElement,
  currentPictureInPictureElement,
  exitDocumentFullscreen,
  requestElementFullscreen,
  togglePictureInPictureFor,
} from "./media-player-platform.ts";
import type {
  MediaPlayerMediaKind,
  MediaPlayerMessages,
  MediaPlayerPlaybackState,
  MediaPlayerRootExpose,
  MediaPlayerSeekReason,
  MediaPlayerSlotState,
  MediaPlayerTextTrack,
  MediaPlayerTimeRange,
} from "./media-player-types.ts";

const {
  id = undefined,
  volume = undefined,
  defaultVolume = 1,
  muted = undefined,
  defaultMuted = false,
  playbackRate = undefined,
  defaultPlaybackRate = 1,
  currentTime = undefined,
  keyboardShortcuts = true,
  seekStep = 5,
  skipStep = 10,
  volumeStep = 0.05,
  messages = undefined,
  dir = "ltr",
  ariaLabel = undefined,
  ariaLabelledby = undefined,
} = defineProps<{
  /**
   * Consumer-owned base id. `null` and `undefined` select a deterministic fallback.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /**
   * Controlled volume from `0` to `1` (`v-model:volume`).
   *
   * @default undefined
   */
  readonly volume?: number;

  /**
   * Initial volume for uncontrolled use.
   *
   * @default 1
   */
  readonly defaultVolume?: number;

  /**
   * Controlled muted state (`v-model:muted`).
   *
   * @default undefined
   */
  readonly muted?: boolean;

  /**
   * Initial muted state for uncontrolled use. Rendered as the native `muted`
   * attribute, so muted autoplay works before hydration.
   *
   * @default false
   */
  readonly defaultMuted?: boolean;

  /**
   * Controlled playback rate (`v-model:playbackRate`).
   *
   * @default undefined
   */
  readonly playbackRate?: number;

  /**
   * Initial playback rate for uncontrolled use.
   *
   * @default 1
   */
  readonly defaultPlaybackRate?: number;

  /**
   * Synchronized playback position (`v-model:currentTime`). The media element
   * stays the source of truth: `timeupdate` emits the position, and a parent
   * value more than half a second away from it seeks.
   *
   * @default undefined
   */
  readonly currentTime?: number;

  /**
   * Handle media keyboard shortcuts while focus is inside the player.
   *
   * @default true
   */
  readonly keyboardShortcuts?: boolean;

  /**
   * Seconds moved by arrow keys on the seek slider and root shortcuts.
   *
   * @default 5
   */
  readonly seekStep?: number;

  /**
   * Seconds skipped by the J and L shortcuts.
   *
   * @default 10
   */
  readonly skipStep?: number;

  /**
   * Volume change for arrow keys, from `0` to `1`.
   *
   * @default 0.05
   */
  readonly volumeStep?: number;

  /**
   * Localized accessible labels for every control part. Missing keys use English defaults.
   *
   * @default undefined
   */
  readonly messages?: MediaPlayerMessages;

  /**
   * Reading direction used by horizontal sliders and arrow keys.
   *
   * @default "ltr"
   */
  readonly dir?: "ltr" | "rtl";

  /**
   * Accessible player name; adds `role="group"`.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;

  /**
   * Ids that name the player; adds `role="group"`.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string;
}>();

const emit = defineEmits<{
  /** Fired when the volume requests a new controlled value. */
  "update:volume": [volume: number];
  /** Fired when the muted state requests a new controlled value. */
  "update:muted": [muted: boolean];
  /** Fired when the playback rate requests a new controlled value. */
  "update:playbackRate": [rate: number];
  /** Fired on every native `timeupdate` and seek with the current position. */
  "update:currentTime": [time: number];
  /** Fired after playback starts. */
  play: [];
  /** Fired after playback pauses. */
  pause: [];
  /** Fired after playback reaches the end. */
  ended: [];
  /** Fired when `play()` is rejected, e.g. by an autoplay policy. */
  playRejected: [reason: unknown];
  /** Fired when the media element reports an error (`MediaError.code`). */
  error: [code: number | null];
}>();

defineSlots<{
  /** Media element and controls. Receives the complete playback state. */
  default(props: MediaPlayerSlotState): unknown;
}>();

const element = useTemplateRef<HTMLDivElement>("element");
const baseId = useDeterministicId({ id: () => id, hint: "media-player" });
const media = shallowRef<HTMLMediaElement | null>(null);
const kind = shallowRef<MediaPlayerMediaKind | null>(null);
const paused = shallowRef(true);
const ended = shallowRef(false);
const waiting = shallowRef(false);
const seeking = shallowRef(false);
const scrubbing = shallowRef(false);
const position = shallowRef(currentTime ?? 0);
const duration = shallowRef(0);
const buffered = shallowRef<readonly MediaPlayerTimeRange[]>([]);
const textTracks = shallowRef<readonly MediaPlayerTextTrack[]>([]);
const fullscreen = shallowRef(false);
const pictureInPicture = shallowRef(false);
const errorCode = shallowRef<number | null>(null);
const fullscreenSupported = shallowRef(false);
const pictureInPictureSupported = shallowRef(false);
let lastCaptionTrack = -1;
let resumeAfterScrub = false;

const volumeState = useControllableState<number>({
  value: () => volume,
  defaultValue: () => defaultVolume,
});
const mutedState = useControllableState<boolean>({
  value: () => muted,
  defaultValue: () => defaultMuted,
});
const rateState = useControllableState<number>({
  value: () => playbackRate,
  defaultValue: () => defaultPlaybackRate,
});
const volumeValue = computed(() => clampMediaValue(volumeState.value.value, 0, 1));
const mutedValue = computed(() => mutedState.value.value);
const rateValue = computed(() => rateState.value.value);
const state = computed<MediaPlayerPlaybackState>(() => {
  if (ended.value) return "ended";
  return paused.value ? "paused" : "playing";
});
const loading = computed(() => seeking.value || (!paused.value && waiting.value));
const captionTrack = computed(() => textTracks.value.find((track) => track.showing)?.index ?? -1);
const messagesValue = computed(() => resolveMediaPlayerMessages(messages));
const slotState = computed<MediaPlayerSlotState>(() => ({
  buffered: buffered.value,
  captionTrack: captionTrack.value,
  currentTime: position.value,
  duration: duration.value,
  error: errorCode.value,
  fullscreen: fullscreen.value,
  loading: loading.value,
  muted: mutedValue.value,
  paused: paused.value,
  pictureInPicture: pictureInPicture.value,
  playbackRate: rateValue.value,
  seeking: seeking.value,
  state: state.value,
  textTracks: textTracks.value,
  volume: volumeValue.value,
}));

function readTextTracks(target: HTMLMediaElement): MediaPlayerTextTrack[] {
  const list = target.textTracks;
  const tracks: MediaPlayerTextTrack[] = [];
  if (list === undefined || list === null) return tracks;
  for (let index = 0; index < list.length; index += 1) {
    const track = list[index];
    if (track === undefined) continue;
    tracks.push(
      Object.freeze({
        index,
        kind: track.kind,
        label: track.label,
        language: track.language,
        showing: track.mode === "showing",
      }),
    );
  }
  return tracks;
}

function syncTracks(): void {
  if (media.value === null) return;
  textTracks.value = readTextTracks(media.value);
  if (captionTrack.value >= 0) lastCaptionTrack = captionTrack.value;
}

function syncFromMedia(): void {
  if (media.value === null) return;
  paused.value = media.value.paused;
  ended.value = media.value.ended;
  seeking.value = media.value.seeking;
  duration.value = normalizeMediaDuration(media.value.duration);
  if (!scrubbing.value) position.value = media.value.currentTime;
  buffered.value = readTimeRanges(media.value.buffered);
}

function applyToMedia(): void {
  if (media.value === null) return;
  if (media.value.volume !== volumeValue.value) media.value.volume = volumeValue.value;
  if (media.value.muted !== mutedValue.value) media.value.muted = mutedValue.value;
  if (media.value.playbackRate !== rateValue.value) media.value.playbackRate = rateValue.value;
}

function setVolume(next: number, unmute = true): boolean {
  // Round away floating-point drift from repeated keyboard steps.
  const target = Math.round(clampMediaValue(next, 0, 1) * 10000) / 10000;
  const changed = volumeState.set(target);
  if (changed) emit("update:volume", target);
  // Raising the volume from a control is an explicit request to hear the media.
  if (unmute && target > 0 && mutedValue.value) setMuted(false);
  return changed;
}

function setMuted(next: boolean): boolean {
  const changed = mutedState.set(next);
  if (changed) emit("update:muted", next);
  return changed;
}

function setPlaybackRate(next: number): boolean {
  if (!Number.isFinite(next) || next <= 0) return false;
  const changed = rateState.set(next);
  if (changed) emit("update:playbackRate", next);
  return changed;
}

watch([volumeValue, mutedValue, rateValue], applyToMedia, { flush: "sync" });

const handlers: Readonly<Record<string, (event: Event) => void>> = {
  play: () => {
    syncFromMedia();
    emit("play");
  },
  playing: () => {
    waiting.value = false;
    syncFromMedia();
  },
  pause: () => {
    syncFromMedia();
    emit("pause");
  },
  ended: () => {
    syncFromMedia();
    emit("ended");
  },
  waiting: () => {
    waiting.value = true;
  },
  canplay: () => {
    waiting.value = false;
    syncFromMedia();
  },
  seeking: syncFromMedia,
  seeked: syncFromMedia,
  timeupdate: () => {
    syncFromMedia();
    if (!scrubbing.value) emit("update:currentTime", position.value);
  },
  durationchange: syncFromMedia,
  loadedmetadata: () => {
    syncFromMedia();
    syncTracks();
  },
  progress: syncFromMedia,
  emptied: () => {
    errorCode.value = null;
    syncFromMedia();
  },
  volumechange: () => {
    if (media.value === null) return;
    if (media.value.muted !== mutedValue.value) setMuted(media.value.muted);
    if (media.value.volume !== volumeValue.value) setVolume(media.value.volume, false);
  },
  ratechange: () => {
    if (media.value !== null && media.value.playbackRate !== rateValue.value) {
      setPlaybackRate(media.value.playbackRate);
    }
  },
  error: () => {
    errorCode.value = media.value?.error?.code ?? null;
    waiting.value = false;
    emit("error", errorCode.value);
  },
  enterpictureinpicture: () => {
    pictureInPicture.value = true;
  },
  leavepictureinpicture: () => {
    pictureInPicture.value = false;
  },
  webkitbeginfullscreen: () => {
    fullscreen.value = true;
  },
  webkitendfullscreen: () => {
    fullscreen.value = false;
  },
};

function refreshCapabilities(): void {
  fullscreenSupported.value =
    element.value !== null && canRequestFullscreen(element.value, media.value);
  pictureInPictureSupported.value = canRequestPictureInPicture(media.value);
}

function registerMedia(target: HTMLMediaElement, mediaKind: MediaPlayerMediaKind): () => void {
  media.value = target;
  kind.value = mediaKind;
  for (const [name, handler] of Object.entries(handlers)) target.addEventListener(name, handler);
  const tracks = target.textTracks;
  tracks?.addEventListener("addtrack", syncTracks);
  tracks?.addEventListener("removetrack", syncTracks);
  tracks?.addEventListener("change", syncTracks);
  applyToMedia();
  syncFromMedia();
  syncTracks();
  if (currentTime !== undefined && currentTime > 0) seek(currentTime, "api");
  if (target.error !== null) handlers.error?.(new Event("error"));
  refreshCapabilities();
  return () => {
    for (const [name, handler] of Object.entries(handlers)) {
      target.removeEventListener(name, handler);
    }
    tracks?.removeEventListener("addtrack", syncTracks);
    tracks?.removeEventListener("removetrack", syncTracks);
    tracks?.removeEventListener("change", syncTracks);
    if (media.value === target) {
      media.value = null;
      kind.value = null;
      refreshCapabilities();
    }
  };
}

async function play(): Promise<boolean> {
  if (media.value === null) return false;
  try {
    await media.value.play();
    return true;
  } catch (reason) {
    syncFromMedia();
    emit("playRejected", reason);
    return false;
  }
}

function pause(): void {
  media.value?.pause();
}

function togglePlay(): Promise<boolean> {
  if (media.value === null) return Promise.resolve(false);
  if (media.value.paused || media.value.ended) return play();
  pause();
  return Promise.resolve(true);
}

function seek(time: number, _reason: MediaPlayerSeekReason = "api"): void {
  if (media.value === null || Number.isNaN(time)) return;
  const upper = duration.value > 0 ? duration.value : Number.POSITIVE_INFINITY;
  const target = clampMediaValue(time, 0, upper);
  media.value.currentTime = target;
  position.value = target;
  ended.value = media.value.ended;
  emit("update:currentTime", target);
}

function setScrubbing(next: boolean): void {
  if (scrubbing.value === next) return;
  scrubbing.value = next;
  if (media.value === null) return;
  if (next) {
    // Pause while scrubbing so audio does not stutter, then resume where it was.
    resumeAfterScrub = !media.value.paused;
    if (resumeAfterScrub) media.value.pause();
  } else if (resumeAfterScrub) {
    resumeAfterScrub = false;
    void play();
  }
}

function setCaptionTrack(index: number): boolean {
  const list = media.value?.textTracks;
  if (list === undefined || list === null) return false;
  let changed = false;
  for (let trackIndex = 0; trackIndex < list.length; trackIndex += 1) {
    const track = list[trackIndex];
    if (track === undefined || !isCaptionTrack(track)) continue;
    const mode: TextTrackMode = trackIndex === index ? "showing" : "disabled";
    if (track.mode !== mode && (mode === "showing" || track.mode === "showing")) {
      track.mode = mode;
      changed = true;
    }
  }
  if (index >= 0) lastCaptionTrack = index;
  syncTracks();
  return changed;
}

function toggleCaptions(): boolean {
  if (captionTrack.value >= 0) return setCaptionTrack(-1);
  const captions = textTracks.value.filter(isCaptionTrack);
  const preferred =
    captions.find((track) => track.index === lastCaptionTrack) ?? captions[0] ?? null;
  return preferred === null ? false : setCaptionTrack(preferred.index);
}

async function toggleFullscreen(): Promise<boolean> {
  if (element.value === null) return false;
  if (fullscreen.value || currentFullscreenElement(element.value.ownerDocument) === element.value) {
    return exitDocumentFullscreen(element.value.ownerDocument);
  }
  return requestElementFullscreen(element.value, media.value);
}

function togglePictureInPicture(): Promise<boolean> {
  return media.value === null ? Promise.resolve(false) : togglePictureInPictureFor(media.value);
}

function onFullscreenChange(): void {
  if (element.value === null) return;
  fullscreen.value = currentFullscreenElement(element.value.ownerDocument) === element.value;
}

function isActivationTarget(target: EventTarget | null): boolean {
  return (
    target instanceof HTMLButtonElement ||
    target instanceof HTMLAnchorElement ||
    (target instanceof HTMLElement && target.getAttribute("role") === "button")
  );
}

function onKeydown(event: KeyboardEvent): void {
  if (!keyboardShortcuts || event.defaultPrevented || isEditableTarget(event.target)) return;
  const intent = resolveMediaShortcut(event);
  if (intent === null) return;
  if (intent.action === "toggle-play" && event.key === " " && isActivationTarget(event.target)) {
    return;
  }
  const direction = dir === "rtl" ? -1 : 1;
  switch (intent.action) {
    case "toggle-play":
      void togglePlay();
      break;
    case "skip-backward":
      seek(position.value - skipStep, "keyboard");
      break;
    case "skip-forward":
      seek(position.value + skipStep, "keyboard");
      break;
    case "seek-backward":
      seek(position.value - seekStep * direction, "keyboard");
      break;
    case "seek-forward":
      seek(position.value + seekStep * direction, "keyboard");
      break;
    case "volume-up":
      setVolume(volumeValue.value + volumeStep);
      break;
    case "volume-down":
      setVolume(volumeValue.value - volumeStep);
      break;
    case "mute":
      setMuted(!mutedValue.value);
      break;
    case "fullscreen":
      if (!fullscreenSupported.value) return;
      void toggleFullscreen();
      break;
    case "captions":
      if (!toggleCaptions()) return;
      break;
    case "seek-percent":
      if (!(duration.value > 0) || !Number.isFinite(duration.value)) return;
      seek((duration.value * (intent.percent ?? 0)) / 100, "keyboard");
      break;
  }
  event.preventDefault();
}

watch(
  () => currentTime,
  (next) => {
    if (next !== undefined && Math.abs(next - position.value) > 0.5) seek(next, "api");
  },
);

// Listeners attach on the client only, keeping server markup free of handlers.
onMounted(() => {
  element.value?.addEventListener("keydown", onKeydown);
  document.addEventListener("fullscreenchange", onFullscreenChange);
  document.addEventListener("webkitfullscreenchange", onFullscreenChange);
  refreshCapabilities();
  if (media.value !== null) {
    pictureInPicture.value = currentPictureInPictureElement(document) === media.value;
  }
});

onBeforeUnmount(() => {
  element.value?.removeEventListener("keydown", onKeydown);
  document.removeEventListener("fullscreenchange", onFullscreenChange);
  document.removeEventListener("webkitfullscreenchange", onFullscreenChange);
});

mediaPlayerContext.provide({
  dir: computed(() => dir),
  fullscreenSupported,
  getPartId: (part) => deriveDeterministicId(baseId.value, part),
  id: baseId,
  kind,
  media,
  messages: messagesValue,
  muted: mutedValue,
  pause,
  pictureInPictureSupported,
  play,
  registerMedia,
  seek,
  seekStep: computed(() => seekStep),
  setCaptionTrack,
  setMuted,
  setPlaybackRate,
  setScrubbing,
  setVolume: (next) => setVolume(next),
  slotState,
  state,
  toggleCaptions,
  toggleFullscreen,
  togglePictureInPicture,
  togglePlay,
  volumeStep: computed(() => volumeStep),
} satisfies MediaPlayerContextValue);

type MediaPlayerRootSetupExpose = Omit<
  MediaPlayerRootExpose,
  keyof MediaPlayerSlotState | "element" | "media"
> & {
  readonly [Key in keyof MediaPlayerSlotState]: { readonly value: MediaPlayerSlotState[Key] };
} & {
  readonly element: typeof element;
  readonly media: typeof media;
};

const exposed = {
  buffered,
  captionTrack,
  currentTime: position,
  duration,
  element,
  error: errorCode,
  fullscreen,
  loading,
  media,
  muted: mutedValue,
  pause,
  paused,
  pictureInPicture,
  play,
  playbackRate: rateValue,
  seek: (time: number) => seek(time, "api"),
  seekBy: (delta: number) => seek(position.value + delta, "api"),
  seeking,
  setCaptionTrack,
  setMuted,
  setPlaybackRate,
  setVolume: (next: number) => setVolume(next),
  state,
  textTracks,
  toggleCaptions,
  toggleFullscreen,
  togglePictureInPicture,
  togglePlay,
  volume: volumeValue,
} satisfies MediaPlayerRootSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    :id="baseId"
    ref="element"
    :role="ariaLabel !== undefined || ariaLabelledby !== undefined ? 'group' : undefined"
    :aria-label="ariaLabel"
    :aria-labelledby="ariaLabelledby"
    :dir
    data-vize-ui="media-player-root"
    part="root"
    :data-state="state"
    :data-media-kind="kind ?? undefined"
    :data-loading="loading ? 'true' : undefined"
    :data-muted="mutedValue ? 'true' : undefined"
    :data-fullscreen="fullscreen ? 'true' : undefined"
    :data-picture-in-picture="pictureInPicture ? 'true' : undefined"
    :data-captions="captionTrack >= 0 ? 'true' : undefined"
  >
    <slot v-bind="slotState" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
