import { computed, readonly, ref, shallowRef, toValue, watch } from "vue";
import type { ComputedRef, MaybeRefOrGetter, Ref, ShallowRef } from "vue";

/** Minimal `TimeRanges`. */
export interface TimeRangesLike {
  /** Number of ranges. */
  readonly length: number;

  /** Start of range `index` in seconds. */
  start(index: number): number;

  /** End of range `index` in seconds. */
  end(index: number): number;
}

/** Minimal `TextTrack`. */
export interface TextTrackLike {
  /** Track identifier. */
  readonly id: string;

  /** `"subtitles"`, `"captions"`, `"descriptions"`, `"chapters"`, or `"metadata"`. */
  readonly kind: string;

  /** Human-readable label. */
  readonly label: string;

  /** BCP 47 language tag. */
  readonly language: string;

  /** `"disabled"`, `"hidden"`, or `"showing"`. */
  mode: string;
}

/** Minimal `TextTrackList`. */
export interface TextTrackListLike extends EventTarget {
  /** Number of tracks. */
  readonly length: number;

  /** Track at `index`. */
  readonly [index: number]: TextTrackLike;
}

/** Minimal `HTMLMediaElement` (`<video>` or `<audio>`). */
export interface MediaElementLike extends EventTarget {
  /** Playback position in seconds. */
  currentTime: number;

  /** Volume from 0 to 1. */
  volume: number;

  /** Whether audio is muted. */
  muted: boolean;

  /** Playback speed multiplier. */
  playbackRate: number;

  /** Media source URL. */
  src: string;

  /** Duration in seconds (`NaN` until metadata loaded). */
  readonly duration: number;

  /** Whether playback is paused. */
  readonly paused: boolean;

  /** Whether playback reached the end. */
  readonly ended: boolean;

  /** Whether the element is seeking. */
  readonly seeking: boolean;

  /** Buffered time ranges. */
  readonly buffered: TimeRangesLike;

  /** Text tracks (`<track>` children). */
  readonly textTracks?: TextTrackListLike;

  /** Start playback; rejects when autoplay is blocked. */
  play(): Promise<void>;

  /** Pause playback. */
  pause(): void;

  /** Whether the element can play `type` (`""`, `"maybe"`, `"probably"`). */
  canPlayType?(type: string): string;

  /** Enter picture-in-picture (video only). */
  requestPictureInPicture?(): Promise<unknown>;
}

/** Document capabilities used for picture-in-picture. */
export interface PictureInPictureDocumentLike {
  /** Whether picture-in-picture is allowed. */
  readonly pictureInPictureEnabled?: boolean;

  /** Element currently in picture-in-picture. */
  readonly pictureInPictureElement?: unknown;

  /** Leave picture-in-picture. */
  exitPictureInPicture?(): Promise<void>;
}

/** One candidate media source. */
export interface MediaSourceCandidate {
  /** Source URL. */
  readonly src: string;

  /** MIME type used to test playability with `canPlayType`. */
  readonly type?: string;
}

/** Summary of one text track. */
export interface MediaTextTrack {
  /** Index in the element's `textTracks`. */
  readonly index: number;

  /** Track identifier. */
  readonly id: string;

  /** Track kind. */
  readonly kind: string;

  /** Human-readable label. */
  readonly label: string;

  /** BCP 47 language tag. */
  readonly language: string;

  /** Current mode. */
  readonly mode: string;
}

/** Options for {@link useMediaControls}. */
export interface UseMediaControlsOptions {
  /**
   * Media source: a URL, or candidates of which the first playable one (per
   * `canPlayType`) is used. Omit to leave the element's `src` untouched.
   *
   * @default undefined
   */
  readonly src?: MaybeRefOrGetter<string | readonly MediaSourceCandidate[] | undefined>;

  /**
   * Document used for picture-in-picture.
   *
   * @default window.document when a browser window exists
   */
  readonly document?: MaybeRefOrGetter<PictureInPictureDocumentLike | null | undefined>;
}

/** Reactive state and actions returned by {@link useMediaControls}. */
export interface MediaControls {
  /** Whether media is playing; assign to play or pause. */
  readonly playing: Ref<boolean>;

  /** Playback position in seconds; assign to seek. */
  readonly currentTime: Ref<number>;

  /** Volume from 0 to 1; assign to change it. */
  readonly volume: Ref<number>;

  /** Whether audio is muted; assign to toggle. */
  readonly muted: Ref<boolean>;

  /** Playback speed; assign to change it. */
  readonly rate: Ref<number>;

  /** Duration in seconds (0 until known). */
  readonly duration: Readonly<Ref<number>>;

  /** Buffered ranges as `[start, end]` pairs. */
  readonly buffered: Readonly<ShallowRef<readonly (readonly [number, number])[]>>;

  /** Whether the element is seeking. */
  readonly seeking: Readonly<Ref<boolean>>;

  /** Whether playback waits for data. */
  readonly waiting: Readonly<Ref<boolean>>;

  /** Whether playback reached the end. */
  readonly ended: Readonly<Ref<boolean>>;

  /** Whether data delivery stalled. */
  readonly stalled: Readonly<Ref<boolean>>;

  /** Most recent `play()` rejection (for example blocked autoplay). */
  readonly error: Readonly<ShallowRef<unknown>>;

  /** Text tracks of the element. */
  readonly tracks: Readonly<ShallowRef<readonly MediaTextTrack[]>>;

  /** Index of the showing text track, or `-1`. */
  readonly selectedTrack: Readonly<Ref<number>>;

  /** Whether picture-in-picture can be requested for the element. */
  readonly supportsPictureInPicture: ComputedRef<boolean>;

  /** Whether the element is in picture-in-picture. */
  readonly isPictureInPicture: Readonly<Ref<boolean>>;

  /**
   * Start playback.
   *
   * @returns Whether playback started.
   */
  readonly play: () => Promise<boolean>;

  /** Pause playback. */
  readonly pause: () => void;

  /**
   * Show text track `index` and hide the others; `-1` hides all.
   *
   * @param index Track index.
   */
  readonly selectTrack: (index: number) => void;

  /**
   * Enter or leave picture-in-picture.
   *
   * @returns Whether the element is in picture-in-picture afterwards.
   */
  readonly togglePictureInPicture: () => Promise<boolean>;
}

function browserDocument(): PictureInPictureDocumentLike | undefined {
  return typeof window === "undefined" ? undefined : window.document;
}

function readRanges(ranges: TimeRangesLike): readonly (readonly [number, number])[] {
  const result: (readonly [number, number])[] = [];
  for (let index = 0; index < ranges.length; index += 1) {
    result.push([ranges.start(index), ranges.end(index)]);
  }
  return result;
}

function chooseSource(
  element: MediaElementLike,
  source: string | readonly MediaSourceCandidate[],
): string | undefined {
  if (typeof source === "string") return source;
  for (const candidate of source) {
    if (candidate.type === undefined || !element.canPlayType) return candidate.src;
    if (element.canPlayType(candidate.type) !== "") return candidate.src;
  }
  return undefined;
}

/**
 * Two-way reactive controls for a `<video>` or `<audio>` element.
 *
 * `playing`, `currentTime`, `volume`, `muted`, and `rate` mirror the
 * element and are writable: assigning them drives the element, while
 * element events update them without echoing the change back. The element
 * is the source of truth whenever a new target attaches. Read-only state
 * covers duration, buffering, seeking, waiting, stalls, text tracks, and
 * picture-in-picture. A rejected `play()` (blocked autoplay) resets
 * `playing` and is exposed through `error`. Listeners are removed when the
 * target changes and when the owning reactive scope stops.
 *
 * Server rendering: the template ref is empty, so every value keeps its
 * neutral default (paused, time 0, volume 1) and nothing is touched.
 *
 * @example
 * ```ts
 * const video = useTemplateRef<HTMLVideoElement>("video");
 * const { playing, currentTime, duration } = useMediaControls(video, {
 *   src: [{ src: "/clip.webm", type: "video/webm" }, { src: "/clip.mp4", type: "video/mp4" }],
 * });
 * ```
 *
 * @param target Reactive media element.
 * @param options Source selection and picture-in-picture document.
 * @default options {}
 * @returns Reactive media state and actions.
 */
export function useMediaControls(
  target: MaybeRefOrGetter<MediaElementLike | null | undefined>,
  options: UseMediaControlsOptions = {},
): MediaControls {
  const playing = ref(false);
  const currentTime = ref(0);
  const volume = ref(1);
  const muted = ref(false);
  const rate = ref(1);
  const duration = ref(0);
  const buffered = shallowRef<readonly (readonly [number, number])[]>([]);
  const seeking = ref(false);
  const waiting = ref(false);
  const ended = ref(false);
  const stalled = ref(false);
  const error = shallowRef<unknown>(undefined);
  const tracks = shallowRef<readonly MediaTextTrack[]>([]);
  const selectedTrack = ref(-1);
  const isPictureInPicture = ref(false);
  // True while element events write into the refs, so the write-back
  // watchers below do not echo the change to the element.
  let syncing = false;

  const resolveElement = (): MediaElementLike | undefined => toValue(target) ?? undefined;
  const resolveDocument = (): PictureInPictureDocumentLike | undefined =>
    options.document === undefined ? browserDocument() : (toValue(options.document) ?? undefined);

  const fromElement = (update: () => void): void => {
    syncing = true;
    try {
      update();
    } finally {
      syncing = false;
    }
  };

  const readTracks = (element: MediaElementLike): void => {
    const list = element.textTracks;
    const summary: MediaTextTrack[] = [];
    let showing = -1;
    for (let index = 0; list && index < list.length; index += 1) {
      const track = list[index];
      if (!track) continue;
      summary.push({
        index,
        id: track.id,
        kind: track.kind,
        label: track.label,
        language: track.language,
        mode: track.mode,
      });
      if (track.mode === "showing" && showing === -1) showing = index;
    }
    tracks.value = summary;
    selectedTrack.value = showing;
  };

  const readAll = (element: MediaElementLike): void => {
    fromElement(() => {
      playing.value = !element.paused;
      currentTime.value = element.currentTime;
      volume.value = element.volume;
      muted.value = element.muted;
      rate.value = element.playbackRate;
    });
    duration.value = Number.isFinite(element.duration) ? element.duration : 0;
    buffered.value = readRanges(element.buffered);
    seeking.value = element.seeking;
    ended.value = element.ended;
    readTracks(element);
    const document = resolveDocument();
    isPictureInPicture.value =
      document?.pictureInPictureElement !== undefined &&
      document.pictureInPictureElement === element;
  };

  const play = async (): Promise<boolean> => {
    const element = resolveElement();
    if (!element) return false;
    try {
      await element.play();
      error.value = undefined;
      return true;
    } catch (cause) {
      error.value = cause;
      fromElement(() => {
        playing.value = !element.paused;
      });
      return false;
    }
  };

  const pause = (): void => {
    resolveElement()?.pause();
  };

  watch(
    resolveElement,
    (element, _previous, onCleanup) => {
      if (!element) return;
      readAll(element);
      const handlers: Record<string, () => void> = {
        timeupdate: () => fromElement(() => (currentTime.value = element.currentTime)),
        durationchange: () => {
          duration.value = Number.isFinite(element.duration) ? element.duration : 0;
        },
        loadedmetadata: () => readAll(element),
        progress: () => {
          buffered.value = readRanges(element.buffered);
          stalled.value = false;
        },
        seeking: () => (seeking.value = true),
        seeked: () => (seeking.value = false),
        waiting: () => (waiting.value = true),
        canplay: () => (waiting.value = false),
        playing: () => {
          waiting.value = false;
          stalled.value = false;
          ended.value = false;
        },
        play: () => fromElement(() => (playing.value = true)),
        pause: () => fromElement(() => (playing.value = false)),
        ended: () => {
          ended.value = true;
          fromElement(() => (playing.value = false));
        },
        stalled: () => (stalled.value = true),
        volumechange: () =>
          fromElement(() => {
            volume.value = element.volume;
            muted.value = element.muted;
          }),
        ratechange: () => fromElement(() => (rate.value = element.playbackRate)),
        enterpictureinpicture: () => (isPictureInPicture.value = true),
        leavepictureinpicture: () => (isPictureInPicture.value = false),
      };
      for (const [event, handler] of Object.entries(handlers)) {
        element.addEventListener(event, handler);
      }
      const onTracksChange = (): void => readTracks(element);
      element.textTracks?.addEventListener("change", onTracksChange);
      element.textTracks?.addEventListener("addtrack", onTracksChange);
      element.textTracks?.addEventListener("removetrack", onTracksChange);
      onCleanup(() => {
        for (const [event, handler] of Object.entries(handlers)) {
          element.removeEventListener(event, handler);
        }
        element.textTracks?.removeEventListener("change", onTracksChange);
        element.textTracks?.removeEventListener("addtrack", onTracksChange);
        element.textTracks?.removeEventListener("removetrack", onTracksChange);
      });
    },
    { immediate: true, flush: "sync" },
  );

  watch(
    [resolveElement, () => toValue(options.src)],
    ([element, source]) => {
      if (!element || source === undefined) return;
      const chosen = chooseSource(element, source);
      if (chosen !== undefined && element.src !== chosen) element.src = chosen;
    },
    { immediate: true, flush: "sync" },
  );

  watch(
    playing,
    (value) => {
      if (syncing) return;
      if (value) void play();
      else pause();
    },
    { flush: "sync" },
  );
  watch(
    currentTime,
    (value) => {
      const element = resolveElement();
      if (!syncing && element) element.currentTime = value;
    },
    { flush: "sync" },
  );
  watch(
    volume,
    (value) => {
      const element = resolveElement();
      if (!syncing && element) element.volume = value;
    },
    { flush: "sync" },
  );
  watch(
    muted,
    (value) => {
      const element = resolveElement();
      if (!syncing && element) element.muted = value;
    },
    { flush: "sync" },
  );
  watch(
    rate,
    (value) => {
      const element = resolveElement();
      if (!syncing && element) element.playbackRate = value;
    },
    { flush: "sync" },
  );

  const selectTrack = (index: number): void => {
    const element = resolveElement();
    const list = element?.textTracks;
    if (!element || !list) return;
    for (let position = 0; position < list.length; position += 1) {
      const track = list[position];
      if (track) track.mode = position === index ? "showing" : "disabled";
    }
    readTracks(element);
  };

  const supportsPictureInPicture = computed(() => {
    const element = resolveElement();
    const document = resolveDocument();
    return (
      typeof element?.requestPictureInPicture === "function" &&
      document?.pictureInPictureEnabled !== false
    );
  });

  const togglePictureInPicture = async (): Promise<boolean> => {
    const element = resolveElement();
    const document = resolveDocument();
    if (!element) return false;
    try {
      if (isPictureInPicture.value) {
        await document?.exitPictureInPicture?.();
        isPictureInPicture.value = false;
      } else if (element.requestPictureInPicture) {
        await element.requestPictureInPicture();
        isPictureInPicture.value = true;
      }
    } catch (cause) {
      error.value = cause;
    }
    return isPictureInPicture.value;
  };

  return {
    playing,
    currentTime,
    volume,
    muted,
    rate,
    duration: readonly(duration),
    buffered,
    seeking: readonly(seeking),
    waiting: readonly(waiting),
    ended: readonly(ended),
    stalled: readonly(stalled),
    error,
    tracks,
    selectedTrack: readonly(selectedTrack),
    supportsPictureInPicture,
    isPictureInPicture: readonly(isPictureInPicture),
    play,
    pause,
    selectTrack,
    togglePictureInPicture,
  };
}
