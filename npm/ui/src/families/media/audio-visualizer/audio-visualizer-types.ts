import type { MaybeRefOrGetter, ShallowRef } from "vue";

/** Sample precision read from the analyser. */
export type AudioAnalyserPrecision = "byte" | "float";

/**
 * Typed analyser buffer for a precision.
 *
 * - `byte`: `Uint8Array` values `0..255` (frequency: scaled decibels; waveform: `128` is silence).
 * - `float`: `Float32Array` values in decibels (frequency) or `-1..1` (waveform).
 */
export type AudioAnalyserData<Precision extends AudioAnalyserPrecision> = Precision extends "float"
  ? Float32Array<ArrayBuffer>
  : Uint8Array<ArrayBuffer>;

/** Minimal structural `MediaStream`, e.g. the stream returned by `useUserMedia()`. */
export interface AudioAnalyserStreamLike {
  /** Every track of the stream. */
  getTracks(): readonly unknown[];
}

/** Audio input accepted by {@link useAudioAnalyser}. */
export type AudioAnalyserSource = AudioAnalyserStreamLike | AudioNode | HTMLMediaElement;

/**
 * Analyser lifecycle.
 *
 * - `idle`: no source, or not yet connected (always the server state).
 * - `suspended`: connected, but the audio context waits for a user gesture (`resume()`).
 * - `running`: frames are being read.
 * - `paused`: connected and running, but frame reads are paused.
 * - `unsupported`: Web Audio (or the source kind) is unavailable.
 * - `error`: connecting the source failed; see `error`.
 */
export type AudioAnalyserState =
  | "error"
  | "idle"
  | "paused"
  | "running"
  | "suspended"
  | "unsupported";

/** How `prefers-reduced-motion: reduce` affects frame reads. */
export type AudioAnalyserReducedMotion = "ignore" | "pause" | "throttle";

/** Frequency-to-bar grouping used by {@link toBars}. */
export type AudioBarScale = "linear" | "log";

/** Options for {@link toBars}. */
export interface AudioBarsOptions {
  /**
   * Bin grouping. `log` spaces bars logarithmically, matching perceived pitch.
   *
   * @default "log"
   */
  readonly scale?: AudioBarScale;

  /**
   * Decibel range used to normalize `float` frequency data.
   *
   * @default [-100, -30]
   */
  readonly decibels?: readonly [min: number, max: number];
}

/** Options for {@link useAudioAnalyser}. */
export interface UseAudioAnalyserOptions<
  Precision extends AudioAnalyserPrecision = AudioAnalyserPrecision,
> {
  /** Reactive audio source. `null`/`undefined` disconnects. */
  readonly source: MaybeRefOrGetter<AudioAnalyserSource | null | undefined>;

  /**
   * Sample precision; selects the typed array type of `frequency` and `waveform`.
   *
   * @default "byte"
   */
  readonly precision?: Precision;

  /**
   * FFT size: a power of two from 32 to 32768. `frequency` has `fftSize / 2` bins.
   *
   * @default 2048
   */
  readonly fftSize?: MaybeRefOrGetter<number | undefined>;

  /**
   * Averaging constant between frames, `0..1`.
   *
   * @default 0.8
   */
  readonly smoothingTimeConstant?: MaybeRefOrGetter<number | undefined>;

  /**
   * Lower bound of the byte-scaled decibel range.
   *
   * @default -100
   */
  readonly minDecibels?: MaybeRefOrGetter<number | undefined>;

  /**
   * Upper bound of the byte-scaled decibel range.
   *
   * @default -30
   */
  readonly maxDecibels?: MaybeRefOrGetter<number | undefined>;

  /**
   * Maximum frames read per second. `undefined` reads every animation frame.
   *
   * @default undefined
   */
  readonly frameRate?: MaybeRefOrGetter<number | undefined>;

  /**
   * Stop reading frames while `true`; the last frame stays published.
   *
   * @default false
   */
  readonly paused?: MaybeRefOrGetter<boolean | undefined>;

  /**
   * Behavior under `prefers-reduced-motion: reduce`.
   *
   * @default "throttle"
   */
  readonly reducedMotion?: MaybeRefOrGetter<AudioAnalyserReducedMotion | undefined>;

  /**
   * Frame rate used when {@link reducedMotion} is `throttle` and the preference matches.
   *
   * @default 8
   */
  readonly reducedMotionFrameRate?: MaybeRefOrGetter<number | undefined>;

  /**
   * Route media-element audio on to the speakers. Creating an element source
   * detaches it from its default output, so this defaults to `true` for elements.
   * Streams and nodes are never routed to the destination (no microphone feedback).
   *
   * @default true
   */
  readonly connectToDestination?: boolean;

  /**
   * Audio context to use. `undefined` shares one lazily created page-wide context.
   *
   * @default undefined
   */
  readonly context?: BaseAudioContext;
}

/** Reactive analyser returned by {@link useAudioAnalyser}. */
export interface AudioAnalyser<Precision extends AudioAnalyserPrecision = "byte"> {
  /** Latest frequency data (`fftSize / 2` bins). Same buffer each frame; the ref triggers per read. */
  readonly frequency: Readonly<ShallowRef<AudioAnalyserData<Precision>>>;

  /** Latest time-domain data (`fftSize` samples). */
  readonly waveform: Readonly<ShallowRef<AudioAnalyserData<Precision>>>;

  /** Root-mean-square level of the latest waveform, `0..1`. */
  readonly level: Readonly<ShallowRef<number>>;

  /** Absolute peak of the latest waveform, `0..1`. */
  readonly peak: Readonly<ShallowRef<number>>;

  /** Lifecycle state. */
  readonly state: Readonly<ShallowRef<AudioAnalyserState>>;

  /** Failure reported while connecting, or `undefined`. */
  readonly error: Readonly<ShallowRef<unknown>>;

  /** Sample rate of the audio context once connected, else `null`. */
  readonly sampleRate: Readonly<ShallowRef<number | null>>;

  /** Resume a suspended audio context; call from a user gesture. Resolves whether it runs. */
  readonly resume: () => Promise<boolean>;

  /** Group the latest frequency data into `count` normalized (`0..1`) bars. */
  readonly bands: (count: number, options?: AudioBarsOptions) => readonly number[];

  /** Disconnect nodes and stop reading frames. Called automatically on scope dispose. */
  readonly dispose: () => void;
}

/** State exposed to AudioVisualizer slots. */
export interface AudioVisualizerSlotState<Precision extends AudioAnalyserPrecision = "byte"> {
  /** Latest frequency data. */
  readonly frequency: AudioAnalyserData<Precision>;

  /** Latest time-domain data. */
  readonly waveform: AudioAnalyserData<Precision>;

  /** RMS level, `0..1`. */
  readonly level: number;

  /** Absolute peak, `0..1`. */
  readonly peak: number;

  /** Lifecycle state. */
  readonly state: AudioAnalyserState;

  /** Group the latest frequency data into normalized bars. */
  readonly bands: (count: number, options?: AudioBarsOptions) => readonly number[];

  /** Build an SVG path of the latest waveform for a `width` x `height` box. */
  readonly waveformPath: (width: number, height: number) => string;

  /** Resume a suspended context from a user gesture. */
  readonly resume: () => Promise<boolean>;
}

/** State exposed to AudioVisualizerBars slots. */
export interface AudioVisualizerBarsSlotState {
  /** Normalized bar heights, `0..1`. */
  readonly bars: readonly number[];

  /** Lifecycle state of the parent visualizer. */
  readonly state: AudioAnalyserState;
}

/** Public instance exposed by AudioVisualizer. */
export interface AudioVisualizerExpose {
  /** Rendered root element. */
  readonly element: HTMLDivElement | null;

  /** Lifecycle state. */
  readonly state: AudioAnalyserState;

  /** RMS level, `0..1`. */
  readonly level: number;

  /** Resume a suspended context from a user gesture. */
  readonly resume: () => Promise<boolean>;
}

/** Public instance exposed by AudioVisualizerBars. */
export interface AudioVisualizerBarsExpose {
  /** Rendered bars container. */
  readonly element: HTMLDivElement | null;

  /** Normalized bar heights, `0..1`. */
  readonly bars: readonly number[];
}
