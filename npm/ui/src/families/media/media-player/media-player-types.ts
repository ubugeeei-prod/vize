/**
 * Playback state mirrored by every MediaPlayer part through `data-state`.
 *
 * - `paused`: playback is paused before the end.
 * - `playing`: playback is running (it may still be `loading`).
 * - `ended`: playback reached the end.
 */
export type MediaPlayerPlaybackState = "ended" | "paused" | "playing";

/** Kind of native element registered with the root. */
export type MediaPlayerMediaKind = "audio" | "video";

/** Why the playback position changed. */
export type MediaPlayerSeekReason = "api" | "keyboard" | "pointer";

/** Native `preload` hints accepted by the media element parts. */
export type MediaPlayerPreload = "auto" | "metadata" | "none";

/** Native CORS policies accepted by the media element parts. */
export type MediaPlayerCrossOrigin = "" | "anonymous" | "use-credentials";

/** Display mode of MediaPlayerTimeDisplay. */
export type MediaPlayerTimeDisplayMode = "current" | "duration" | "remaining";

/** Keyboard shortcut actions handled by the root while focus is inside it. */
export type MediaPlayerShortcut =
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
  | "volume-up";

/** One buffered range in seconds. */
export interface MediaPlayerTimeRange {
  readonly start: number;
  readonly end: number;
}

/** Snapshot of one text track exposed by the media element. */
export interface MediaPlayerTextTrack {
  /** Position in `media.textTracks`. */
  readonly index: number;

  /** Native track kind, e.g. `captions` or `subtitles`. */
  readonly kind: string;

  /** Human-readable track label. */
  readonly label: string;

  /** BCP 47 language tag. */
  readonly language: string;

  /** Whether the track is currently rendered. */
  readonly showing: boolean;
}

/**
 * Accessible labels used by the control parts. Every key is optional; missing
 * keys fall back to English defaults. Parameterized labels are functions.
 */
export interface MediaPlayerMessages {
  /** Play button label while paused. @default "Play" */
  readonly play?: string;
  /** Play button label while playing. @default "Pause" */
  readonly pause?: string;
  /** Play button label after the end. @default "Replay" */
  readonly replay?: string;
  /** Mute button label while audible. @default "Mute" */
  readonly mute?: string;
  /** Mute button label while muted. @default "Unmute" */
  readonly unmute?: string;
  /** Seek slider label. @default "Seek" */
  readonly seek?: string;
  /** Volume slider label. @default "Volume" */
  readonly volume?: string;
  /** Seek slider value text. @default "1:23 of 4:56" */
  readonly seekValueText?: (current: string, duration: string) => string;
  /** Volume slider value text. @default "50%" or "Muted" */
  readonly volumeValueText?: (percent: number, muted: boolean) => string;
  /** Captions button label while captions are hidden. @default "Show captions" */
  readonly showCaptions?: string;
  /** Captions button label while captions are showing. @default "Hide captions" */
  readonly hideCaptions?: string;
  /** Playback rate button label. @default "Playback speed 1×" */
  readonly playbackRate?: (rate: number) => string;
  /** Fullscreen button label while inline. @default "Enter fullscreen" */
  readonly enterFullscreen?: string;
  /** Fullscreen button label while fullscreen. @default "Exit fullscreen" */
  readonly exitFullscreen?: string;
  /** Picture-in-picture button label while inline. @default "Enter picture-in-picture" */
  readonly enterPictureInPicture?: string;
  /** Picture-in-picture button label while floating. @default "Exit picture-in-picture" */
  readonly exitPictureInPicture?: string;
  /** Loading indicator label. @default "Loading" */
  readonly loading?: string;
}

/** Fully resolved messages with every default applied. */
export type MediaPlayerResolvedMessages = {
  readonly [Key in keyof MediaPlayerMessages]-?: Exclude<MediaPlayerMessages[Key], undefined>;
};

/** State exposed to every MediaPlayer slot. */
export interface MediaPlayerSlotState {
  /** Playback state. */
  readonly state: MediaPlayerPlaybackState;

  /** Whether playback is paused (also `true` after the end). */
  readonly paused: boolean;

  /** Whether playback stalled while buffering or seeking. */
  readonly loading: boolean;

  /** Whether a seek is in progress. */
  readonly seeking: boolean;

  /** Current position in seconds. */
  readonly currentTime: number;

  /** Duration in seconds; `0` while unknown, `Infinity` for live streams. */
  readonly duration: number;

  /** Buffered ranges in seconds. */
  readonly buffered: readonly MediaPlayerTimeRange[];

  /** Volume from `0` to `1`. */
  readonly volume: number;

  /** Whether audio is muted. */
  readonly muted: boolean;

  /** Playback rate multiplier. */
  readonly playbackRate: number;

  /** Text tracks exposed by the media element. */
  readonly textTracks: readonly MediaPlayerTextTrack[];

  /** Index of the showing caption or subtitle track, or `-1`. */
  readonly captionTrack: number;

  /** Whether the root is the fullscreen element. */
  readonly fullscreen: boolean;

  /** Whether the video floats in picture-in-picture. */
  readonly pictureInPicture: boolean;

  /** Last media error code (`MediaError.code`), or `null`. */
  readonly error: number | null;
}

/** Public instance exposed by MediaPlayerRoot. */
export interface MediaPlayerRootExpose extends MediaPlayerSlotState {
  /** Rendered root element. */
  readonly element: HTMLDivElement | null;

  /** Registered native media element. */
  readonly media: HTMLMediaElement | null;

  /** Start playback. Resolves `false` when the browser rejects it (autoplay policy). */
  readonly play: () => Promise<boolean>;

  /** Pause playback. */
  readonly pause: () => void;

  /** Toggle playback; replays from the start after the end. */
  readonly togglePlay: () => Promise<boolean>;

  /** Seek to an absolute position in seconds (clamped to the duration). */
  readonly seek: (time: number) => void;

  /** Seek relative to the current position in seconds. */
  readonly seekBy: (delta: number) => void;

  /** Request a volume from `0` to `1`. */
  readonly setVolume: (volume: number) => boolean;

  /** Request a muted state. */
  readonly setMuted: (muted: boolean) => boolean;

  /** Request a playback rate. */
  readonly setPlaybackRate: (rate: number) => boolean;

  /** Show one caption/subtitle track by index, or hide all with `-1`. */
  readonly setCaptionTrack: (index: number) => boolean;

  /** Toggle between hidden captions and the last (or first) caption track. */
  readonly toggleCaptions: () => boolean;

  /** Enter or leave fullscreen on the root. Resolves whether the request succeeded. */
  readonly toggleFullscreen: () => Promise<boolean>;

  /** Enter or leave picture-in-picture for a video. Resolves whether the request succeeded. */
  readonly togglePictureInPicture: () => Promise<boolean>;
}

/** Public instance exposed by button parts. */
export interface MediaPlayerButtonExpose {
  /** Rendered native button. */
  readonly element: HTMLButtonElement | null;

  /** Whether the button is disabled. */
  readonly disabled: boolean;
}

/** Public instance exposed by slider parts. */
export interface MediaPlayerSliderExpose {
  /** Rendered track element. */
  readonly element: HTMLDivElement | null;

  /** Rendered focusable slider thumb. */
  readonly thumb: HTMLDivElement | null;

  /** Whether a pointer is scrubbing. */
  readonly dragging: boolean;

  /** Move focus to the slider thumb. */
  readonly focus: (options?: FocusOptions) => void;
}

/** Public instance exposed by media element parts. */
export interface MediaPlayerMediaExpose<Element extends HTMLMediaElement> {
  /** Rendered native media element. */
  readonly element: Element | null;
}
