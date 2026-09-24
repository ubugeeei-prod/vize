/** Compile-only assertions for the public MediaPlayer contract. */

import {
  MediaPlayer,
  MediaPlayerRoot,
  formatMediaTime,
  resolveMediaPlayerMessages,
  resolveMediaShortcut,
  type MediaPlayerButtonExpose,
  type MediaPlayerMediaExpose,
  type MediaPlayerMediaKind,
  type MediaPlayerMessages,
  type MediaPlayerPlaybackState,
  type MediaPlayerResolvedMessages,
  type MediaPlayerRootExpose,
  type MediaPlayerSeekReason,
  type MediaPlayerShortcut,
  type MediaPlayerSliderExpose,
  type MediaPlayerSlotState,
  type MediaPlayerTextTrack,
  type MediaPlayerTimeDisplayMode,
  type MediaPlayerTimeRange,
} from "./media-player.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const root: MediaPlayerRootExpose;
declare const slider: MediaPlayerSliderExpose;
declare const button: MediaPlayerButtonExpose;
declare const video: MediaPlayerMediaExpose<HTMLVideoElement>;
declare const resolved: MediaPlayerResolvedMessages;

type _StateIsLiteral = Expect<Equal<MediaPlayerPlaybackState, "ended" | "paused" | "playing">>;
type _KindIsLiteral = Expect<Equal<MediaPlayerMediaKind, "audio" | "video">>;
type _ReasonIsLiteral = Expect<Equal<MediaPlayerSeekReason, "api" | "keyboard" | "pointer">>;
type _ModeIsLiteral = Expect<
  Equal<MediaPlayerTimeDisplayMode, "current" | "duration" | "remaining">
>;
type _ShortcutsAreClosed = Expect<
  Equal<
    MediaPlayerShortcut,
    | "captions"
    | "fullscreen"
    | "mute"
    | "seek-backward"
    | "seek-forward"
    | "seek-percent"
    | "skip-backward"
    | "skip-forward"
    | "toggle-play"
    | "volume-down"
    | "volume-up"
  >
>;
type _RangeIsReadonly = Expect<
  Equal<MediaPlayerTimeRange, { readonly start: number; readonly end: number }>
>;
type _TracksInSlot = Expect<
  Equal<MediaPlayerSlotState["textTracks"], readonly MediaPlayerTextTrack[]>
>;
type _PlayResolvesBoolean = Expect<Equal<typeof root.play, () => Promise<boolean>>>;
type _MediaIsNative = Expect<Equal<typeof root.media, HTMLMediaElement | null>>;
type _SliderThumb = Expect<Equal<typeof slider.thumb, HTMLDivElement | null>>;
type _ButtonElement = Expect<Equal<typeof button.element, HTMLButtonElement | null>>;
type _VideoElement = Expect<Equal<typeof video.element, HTMLVideoElement | null>>;
type _ResolvedIsTotal = Expect<Equal<typeof resolved.play, string>>;
type _ResolvedFunctions = Expect<
  Equal<typeof resolved.seekValueText, (current: string, duration: string) => string>
>;
type _AliasIsRoot = Expect<Equal<typeof MediaPlayer, typeof MediaPlayerRoot>>;
type _FormatReturnsString = Expect<Equal<ReturnType<typeof formatMediaTime>, string>>;
type _ShortcutNullable = Expect<
  Equal<
    ReturnType<typeof resolveMediaShortcut>,
    { readonly action: MediaPlayerShortcut; readonly percent?: number } | null
  >
>;

const partial: MediaPlayerMessages = { play: "Play" };
void resolveMediaPlayerMessages(partial);

// @ts-expect-error playback states are a closed union.
const _unknownState: MediaPlayerPlaybackState = "buffering";
// @ts-expect-error parameterized messages must be functions.
const _badMessage: MediaPlayerMessages = { seekValueText: "1 of 2" };
// @ts-expect-error exposed state is read-only.
root.volume = 0.5;
