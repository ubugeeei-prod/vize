import type {
  MediaPlayerMessages,
  MediaPlayerResolvedMessages,
  MediaPlayerShortcut,
  MediaPlayerTextTrack,
  MediaPlayerTimeRange,
} from "./media-player-types.ts";

/** English defaults for {@link MediaPlayerMessages}. */
export const mediaPlayerDefaultMessages: MediaPlayerResolvedMessages = Object.freeze({
  play: "Play",
  pause: "Pause",
  replay: "Replay",
  mute: "Mute",
  unmute: "Unmute",
  seek: "Seek",
  volume: "Volume",
  seekValueText: (current: string, duration: string) => `${current} of ${duration}`,
  volumeValueText: (percent: number, muted: boolean) => (muted ? "Muted" : `${percent}%`),
  showCaptions: "Show captions",
  hideCaptions: "Hide captions",
  playbackRate: (rate: number) => `Playback speed ${rate}×`,
  enterFullscreen: "Enter fullscreen",
  exitFullscreen: "Exit fullscreen",
  enterPictureInPicture: "Enter picture-in-picture",
  exitPictureInPicture: "Exit picture-in-picture",
  loading: "Loading",
});

/** Playback rates cycled by MediaPlayerPlaybackRateButton by default. */
export const MEDIA_PLAYER_DEFAULT_RATES: readonly number[] = Object.freeze([0.5, 1, 1.25, 1.5, 2]);

/** Merge consumer messages over the English defaults. */
export function resolveMediaPlayerMessages(
  messages: MediaPlayerMessages | undefined,
): MediaPlayerResolvedMessages {
  const defaults = mediaPlayerDefaultMessages;
  if (messages === undefined) return defaults;
  return Object.freeze({
    play: messages.play ?? defaults.play,
    pause: messages.pause ?? defaults.pause,
    replay: messages.replay ?? defaults.replay,
    mute: messages.mute ?? defaults.mute,
    unmute: messages.unmute ?? defaults.unmute,
    seek: messages.seek ?? defaults.seek,
    volume: messages.volume ?? defaults.volume,
    seekValueText: messages.seekValueText ?? defaults.seekValueText,
    volumeValueText: messages.volumeValueText ?? defaults.volumeValueText,
    showCaptions: messages.showCaptions ?? defaults.showCaptions,
    hideCaptions: messages.hideCaptions ?? defaults.hideCaptions,
    playbackRate: messages.playbackRate ?? defaults.playbackRate,
    enterFullscreen: messages.enterFullscreen ?? defaults.enterFullscreen,
    exitFullscreen: messages.exitFullscreen ?? defaults.exitFullscreen,
    enterPictureInPicture: messages.enterPictureInPicture ?? defaults.enterPictureInPicture,
    exitPictureInPicture: messages.exitPictureInPicture ?? defaults.exitPictureInPicture,
    loading: messages.loading ?? defaults.loading,
  });
}

/** Clamp a number into `[min, max]`, mapping non-finite input to `min`. */
export function clampMediaValue(value: number, min: number, max: number): number {
  if (Number.isNaN(value)) return min;
  return Math.min(max, Math.max(min, value));
}

/** Normalize a native duration: `NaN` and negatives become `0`, live streams stay `Infinity`. */
export function normalizeMediaDuration(duration: number): number {
  if (duration === Number.POSITIVE_INFINITY) return duration;
  return Number.isFinite(duration) && duration > 0 ? duration : 0;
}

/**
 * Format seconds as `m:ss` or `h:mm:ss`. The hour field appears when either the
 * value or `reference` (usually the duration) reaches one hour, so current and
 * total times align. Negative values render with a leading minus sign; unknown
 * or infinite values render as `--:--`.
 */
export function formatMediaTime(seconds: number, reference = seconds): string {
  if (!Number.isFinite(seconds)) return "--:--";
  const negative = seconds < 0;
  const total = Math.floor(Math.abs(seconds));
  const hours = Math.floor(total / 3600);
  const minutes = Math.floor((total % 3600) / 60);
  const secs = total % 60;
  const withHours = hours > 0 || (Number.isFinite(reference) && Math.abs(reference) >= 3600);
  const body = withHours
    ? `${hours}:${String(minutes).padStart(2, "0")}:${String(secs).padStart(2, "0")}`
    : `${minutes}:${String(secs).padStart(2, "0")}`;
  return negative ? `-${body}` : body;
}

/** Copy a native `TimeRanges` object into immutable plain ranges. */
export function readTimeRanges(ranges: TimeRanges | null | undefined): MediaPlayerTimeRange[] {
  const result: MediaPlayerTimeRange[] = [];
  if (ranges === null || ranges === undefined) return result;
  for (let index = 0; index < ranges.length; index += 1) {
    result.push(Object.freeze({ start: ranges.start(index), end: ranges.end(index) }));
  }
  return result;
}

/** End of the buffered range that contains `time`, or `time` when none does. */
export function bufferedEnd(ranges: readonly MediaPlayerTimeRange[], time: number): number {
  for (const range of ranges) {
    if (time >= range.start && time <= range.end) return range.end;
  }
  return time;
}

/** Whether a text track can be toggled as captions. */
export function isCaptionTrack(track: Pick<MediaPlayerTextTrack, "kind">): boolean {
  return track.kind === "captions" || track.kind === "subtitles";
}

/** Whether a keyboard event target edits text, so shortcuts must not steal its keys. */
export function isEditableTarget(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false;
  if (target.isContentEditable) return true;
  if (target instanceof HTMLTextAreaElement || target instanceof HTMLSelectElement) return true;
  if (!(target instanceof HTMLInputElement)) return false;
  return ![
    "button",
    "checkbox",
    "color",
    "file",
    "image",
    "radio",
    "range",
    "reset",
    "submit",
  ].includes(target.type);
}

/** One resolved keyboard shortcut. `percent` is set for `seek-percent`. */
export interface MediaPlayerShortcutIntent {
  readonly action: MediaPlayerShortcut;
  readonly percent?: number;
}

/**
 * Resolve a keydown into a media shortcut, or `null` when the key is not a
 * shortcut. Modified keys (Ctrl, Meta, Alt) are never shortcuts.
 *
 * Space/K toggle play, J/L skip ±10 s, ←/→ seek ±5 s, ↑/↓ change volume, M
 * mutes, F toggles fullscreen, C toggles captions, and 0–9 seek to 0–90 %.
 */
export function resolveMediaShortcut(
  event: Pick<KeyboardEvent, "altKey" | "ctrlKey" | "key" | "metaKey">,
): MediaPlayerShortcutIntent | null {
  if (event.altKey || event.ctrlKey || event.metaKey) return null;
  const key = event.key.length === 1 ? event.key.toLowerCase() : event.key;
  switch (key) {
    case " ":
    case "k":
      return { action: "toggle-play" };
    case "j":
      return { action: "skip-backward" };
    case "l":
      return { action: "skip-forward" };
    case "ArrowLeft":
      return { action: "seek-backward" };
    case "ArrowRight":
      return { action: "seek-forward" };
    case "ArrowUp":
      return { action: "volume-up" };
    case "ArrowDown":
      return { action: "volume-down" };
    case "m":
      return { action: "mute" };
    case "f":
      return { action: "fullscreen" };
    case "c":
      return { action: "captions" };
    default:
      if (/^[0-9]$/.test(key)) return { action: "seek-percent", percent: Number(key) * 10 };
      return null;
  }
}

/** Minimal rectangle read from `getBoundingClientRect()`. */
export interface MediaPlayerRect {
  readonly left: number;
  readonly width: number;
}

/** Horizontal pointer fraction in reading direction, clamped to `[0, 1]`. */
export function pointerRatio(rect: MediaPlayerRect, clientX: number, rtl: boolean): number {
  const raw = rect.width > 0 ? (clientX - rect.left) / rect.width : 0;
  const ratio = clampMediaValue(raw, 0, 1);
  return rtl ? 1 - ratio : ratio;
}

/** Percentage (`0`–`100`, two decimals) of `value` within `[0, max]`. */
export function toPercent(value: number, max: number): number {
  if (!(max > 0) || !Number.isFinite(max)) return 0;
  return Math.round(clampMediaValue(value / max, 0, 1) * 10000) / 100;
}

/** Keyboard intent for a horizontal media slider. */
export type MediaPlayerSliderIntent =
  | { readonly kind: "edge"; readonly edge: "max" | "min" }
  | { readonly kind: "page"; readonly sign: -1 | 1 }
  | { readonly kind: "step"; readonly sign: -1 | 1 };

/**
 * Resolve a slider keydown. Arrow keys step (horizontal arrows invert in RTL),
 * Page Up/Down page, and Home/End jump to the edges. Other keys return `null`.
 */
export function resolveSliderKey(key: string, rtl: boolean): MediaPlayerSliderIntent | null {
  const forward: 1 | -1 = rtl ? -1 : 1;
  switch (key) {
    case "ArrowRight":
      return { kind: "step", sign: forward };
    case "ArrowLeft":
      return { kind: "step", sign: forward === 1 ? -1 : 1 };
    case "ArrowUp":
      return { kind: "step", sign: 1 };
    case "ArrowDown":
      return { kind: "step", sign: -1 };
    case "PageUp":
      return { kind: "page", sign: 1 };
    case "PageDown":
      return { kind: "page", sign: -1 };
    case "Home":
      return { kind: "edge", edge: "min" };
    case "End":
      return { kind: "edge", edge: "max" };
    default:
      return null;
  }
}

/** Capture a pointer, ignoring platforms or pointers that cannot be captured. */
export function captureMediaPointer(element: Element, pointerId: number): void {
  try {
    element.setPointerCapture(pointerId);
  } catch {
    // Synthetic or already-released pointers cannot be captured.
  }
}

/** Release a captured pointer, ignoring pointers that are no longer captured. */
export function releaseMediaPointer(element: Element, pointerId: number): void {
  try {
    if (element.hasPointerCapture(pointerId)) element.releasePointerCapture(pointerId);
  } catch {
    // The pointer may already be released.
  }
}
