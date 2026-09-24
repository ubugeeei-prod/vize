import {
  getCurrentScope,
  onScopeDispose,
  shallowReadonly,
  shallowRef,
  toValue,
  triggerRef,
  watch,
} from "vue";
import type { ShallowRef } from "vue";

import { computeLevel, computePeak, toBars } from "./audio-visualizer-math.ts";
import type {
  AudioAnalyser,
  AudioAnalyserSource,
  AudioAnalyserState,
  AudioAnalyserStreamLike,
  AudioBarsOptions,
  UseAudioAnalyserOptions,
} from "./audio-visualizer-types.ts";

const SETUP_DIAGNOSTIC = "VIZE_UI_AUDIO_ANALYSER_SETUP";
const DEFAULT_FFT_SIZE = 2048;
const REDUCED_MOTION_QUERY = "(prefers-reduced-motion: reduce)";

/** Media element sources are single-use per element and context: share them. */
const elementSources = new WeakMap<BaseAudioContext, WeakMap<HTMLMediaElement, AudioNode>>();
let sharedContext: AudioContext | null = null;

function audioContextConstructor(): (new () => AudioContext) | null {
  return typeof globalThis.AudioContext === "function" ? globalThis.AudioContext : null;
}

function isRealtimeContext(context: BaseAudioContext): context is AudioContext {
  return typeof AudioContext === "function" && context instanceof AudioContext;
}

function isMediaElement(source: AudioAnalyserSource): source is HTMLMediaElement {
  return typeof HTMLMediaElement === "function" && source instanceof HTMLMediaElement;
}

function isAudioNode(source: AudioAnalyserSource): source is AudioNode {
  return typeof AudioNode === "function" && source instanceof AudioNode;
}

function isStream(source: AudioAnalyserSource): source is AudioAnalyserStreamLike & MediaStream {
  return typeof MediaStream === "function" && source instanceof MediaStream;
}

function validFftSize(value: number | undefined): number {
  if (value === undefined) return DEFAULT_FFT_SIZE;
  const valid =
    Number.isInteger(value) && value >= 32 && value <= 32768 && (value & (value - 1)) === 0;
  if (!valid) {
    throw new RangeError(
      "VIZE_UI_AUDIO_ANALYSER_FFT_SIZE: fftSize must be a power of two from 32 to 32768",
    );
  }
  return value;
}

type AnalyserBuffer = Float32Array<ArrayBuffer> | Uint8Array<ArrayBuffer>;

function floatBuffer(length: number): Float32Array<ArrayBuffer> {
  return new Float32Array(length);
}

function byteBuffer(length: number): Uint8Array<ArrayBuffer> {
  return new Uint8Array(length).fill(128);
}

/**
 * Reactive Web Audio analyser for a media element, stream, or audio node.
 *
 * SSR-safe: nothing is created during server rendering, and connection is
 * deferred to a microtask so a hydrating component renders the same `idle`
 * state as the server. Frames are read with `requestAnimationFrame` only while
 * the context runs, the analyser is not paused, and a source is connected.
 * Media-element source nodes are cached per element (the platform allows one
 * per element), and every node is disconnected when the scope is disposed.
 *
 * @throws {Error} Outside component setup or an active effect scope.
 */
export function useAudioAnalyser(
  options: UseAudioAnalyserOptions<"float"> & { readonly precision: "float" },
): AudioAnalyser<"float">;
export function useAudioAnalyser(options: UseAudioAnalyserOptions<"byte">): AudioAnalyser<"byte">;
export function useAudioAnalyser(
  options: UseAudioAnalyserOptions,
): AudioAnalyser<"byte"> | AudioAnalyser<"float"> {
  if (!getCurrentScope()) {
    throw new Error(`${SETUP_DIAGNOSTIC}: use inside component setup or an active effect scope`);
  }
  return options.precision === "float"
    ? createAnalyser(options, floatBuffer)
    : createAnalyser(options, byteBuffer);
}

interface AnalyserHandle<Data extends AnalyserBuffer> extends Omit<
  AudioAnalyser,
  "frequency" | "waveform"
> {
  readonly frequency: Readonly<ShallowRef<Data>>;
  readonly waveform: Readonly<ShallowRef<Data>>;
}

function createAnalyser<Data extends AnalyserBuffer>(
  options: UseAudioAnalyserOptions,
  createBuffer: (length: number) => Data,
): AnalyserHandle<Data> {
  const initialSize = validFftSize(toValue(options.fftSize));
  // Frequency starts empty (0); the waveform starts as silence (128 for bytes, 0 for floats).
  function emptyFrequency(length: number): Data {
    const buffer = createBuffer(length);
    buffer.fill(0);
    return buffer;
  }
  const frequency = shallowRef(emptyFrequency(initialSize / 2));
  const waveform = shallowRef(createBuffer(initialSize));
  const level = shallowRef(0);
  const peak = shallowRef(0);
  const state = shallowRef<AudioAnalyserState>("idle");
  const error = shallowRef<unknown>(undefined);
  const sampleRate = shallowRef<number | null>(null);

  let context: BaseAudioContext | null = null;
  let analyser: AnalyserNode | null = null;
  let input: AudioNode | null = null;
  let routed = false;
  let mediaElement: HTMLMediaElement | null = null;
  let frame: number | null = null;
  let lastRead = Number.NEGATIVE_INFINITY;
  let reducedMotion = false;
  let reducedMotionList: MediaQueryList | null = null;
  let disposed = false;
  let pending = false;

  function currentFrameRate(): number | undefined {
    const mode = toValue(options.reducedMotion) ?? "throttle";
    if (reducedMotion && mode === "throttle") return toValue(options.reducedMotionFrameRate) ?? 8;
    return toValue(options.frameRate);
  }

  function shouldRead(): boolean {
    if (analyser === null || context === null || context.state !== "running") return false;
    if (toValue(options.paused) === true) return false;
    return !(reducedMotion && (toValue(options.reducedMotion) ?? "throttle") === "pause");
  }

  function syncState(): void {
    if (analyser === null || context === null) return;
    if (context.state !== "running") state.value = "suspended";
    else state.value = shouldRead() ? "running" : "paused";
  }

  function read(): void {
    if (analyser === null) return;
    if (analyser.frequencyBinCount !== frequency.value.length) {
      frequency.value = emptyFrequency(analyser.frequencyBinCount);
      waveform.value = createBuffer(analyser.fftSize);
    }
    fill(analyser, frequency.value, waveform.value);
    triggerRef(frequency);
    triggerRef(waveform);
    level.value = computeLevel(waveform.value);
    peak.value = computePeak(waveform.value);
  }

  function stopLoop(): void {
    if (frame !== null && typeof cancelAnimationFrame === "function") cancelAnimationFrame(frame);
    frame = null;
  }

  function tick(time: number): void {
    frame = null;
    if (!shouldRead()) {
      syncState();
      return;
    }
    const rate = currentFrameRate();
    if (rate === undefined || !(rate > 0) || time - lastRead >= 1000 / rate - 1) {
      lastRead = time;
      read();
    }
    frame = requestAnimationFrame(tick);
  }

  function startLoop(): void {
    syncState();
    if (frame !== null || !shouldRead() || typeof requestAnimationFrame !== "function") return;
    frame = requestAnimationFrame(tick);
  }

  function applySettings(): void {
    if (analyser === null) return;
    analyser.fftSize = validFftSize(toValue(options.fftSize));
    analyser.smoothingTimeConstant = toValue(options.smoothingTimeConstant) ?? 0.8;
    const minDecibels = toValue(options.minDecibels) ?? -100;
    const maxDecibels = toValue(options.maxDecibels) ?? -30;
    // Assign in an order that never inverts the range mid-update.
    if (minDecibels >= analyser.maxDecibels) {
      analyser.maxDecibels = maxDecibels;
      analyser.minDecibels = minDecibels;
    } else {
      analyser.minDecibels = minDecibels;
      analyser.maxDecibels = maxDecibels;
    }
  }

  function onPlay(): void {
    void resume();
  }

  function disconnect(): void {
    stopLoop();
    if (input !== null) {
      try {
        if (analyser !== null) input.disconnect(analyser);
      } catch {
        // Already disconnected.
      }
    }
    if (routed && input !== null && context !== null) {
      try {
        input.disconnect(context.destination);
      } catch {
        // Already disconnected.
      }
    }
    mediaElement?.removeEventListener("play", onPlay);
    mediaElement = null;
    input = null;
    routed = false;
    analyser?.disconnect();
    analyser = null;
  }

  function resolveContext(): BaseAudioContext | null {
    if (options.context !== undefined) return options.context;
    if (sharedContext !== null) return sharedContext;
    const Constructor = audioContextConstructor();
    if (Constructor === null) return null;
    sharedContext = new Constructor();
    return sharedContext;
  }

  function inputFor(target: BaseAudioContext, source: AudioAnalyserSource): AudioNode | null {
    if (isAudioNode(source)) return source;
    if (isMediaElement(source)) {
      let cache = elementSources.get(target);
      if (cache === undefined) {
        cache = new WeakMap();
        elementSources.set(target, cache);
      }
      const cached = cache.get(source);
      if (cached !== undefined) return cached;
      if (!isRealtimeContext(target)) return null;
      const node = target.createMediaElementSource(source);
      cache.set(source, node);
      return node;
    }
    if (isStream(source) && isRealtimeContext(target))
      return target.createMediaStreamSource(source);
    return null;
  }

  function connect(): void {
    pending = false;
    if (disposed) return;
    disconnect();
    error.value = undefined;
    const source = toValue(options.source);
    if (source === null || source === undefined) {
      state.value = "idle";
      return;
    }
    try {
      context = resolveContext();
      if (context === null) {
        state.value = "unsupported";
        return;
      }
      const node = inputFor(context, source);
      if (node === null) {
        state.value = "unsupported";
        return;
      }
      analyser = context.createAnalyser();
      applySettings();
      node.connect(analyser);
      input = node;
      if (isMediaElement(source)) {
        mediaElement = source;
        source.addEventListener("play", onPlay);
        if (options.connectToDestination !== false) {
          node.connect(context.destination);
          routed = true;
        }
      }
      sampleRate.value = context.sampleRate;
      read();
      startLoop();
    } catch (reason) {
      disconnect();
      error.value = reason;
      state.value = "error";
    }
  }

  function schedule(): void {
    if (typeof window === "undefined" || pending || disposed) return;
    pending = true;
    void Promise.resolve().then(connect);
  }

  async function resume(): Promise<boolean> {
    if (context === null || analyser === null) return false;
    if (
      context.state !== "running" &&
      "resume" in context &&
      typeof context.resume === "function"
    ) {
      try {
        await context.resume();
      } catch (reason) {
        error.value = reason;
      }
    }
    startLoop();
    return context.state === "running";
  }

  function onReducedMotionChange(event: MediaQueryListEvent): void {
    reducedMotion = event.matches;
    stopLoop();
    startLoop();
  }

  function onContextStateChange(): void {
    stopLoop();
    startLoop();
  }

  watch(() => toValue(options.source), schedule, { immediate: true });
  watch(
    [
      () => toValue(options.fftSize),
      () => toValue(options.smoothingTimeConstant),
      () => toValue(options.minDecibels),
      () => toValue(options.maxDecibels),
    ],
    () => {
      if (analyser === null) return;
      try {
        applySettings();
      } catch (reason) {
        error.value = reason;
        state.value = "error";
      }
    },
  );
  watch([() => toValue(options.paused), () => toValue(options.reducedMotion)], () => {
    stopLoop();
    startLoop();
  });

  if (typeof window !== "undefined" && typeof globalThis.matchMedia === "function") {
    void Promise.resolve().then(() => {
      if (disposed) return;
      reducedMotionList = globalThis.matchMedia(REDUCED_MOTION_QUERY);
      reducedMotion = reducedMotionList.matches;
      reducedMotionList.addEventListener("change", onReducedMotionChange);
    });
  }

  let watchedContext: BaseAudioContext | null = null;
  watch(state, () => {
    if (context === watchedContext) return;
    watchedContext?.removeEventListener("statechange", onContextStateChange);
    watchedContext = context;
    watchedContext?.addEventListener("statechange", onContextStateChange);
  });

  function dispose(): void {
    if (disposed) return;
    disconnect();
    disposed = true;
    watchedContext?.removeEventListener("statechange", onContextStateChange);
    watchedContext = null;
    reducedMotionList?.removeEventListener("change", onReducedMotionChange);
    reducedMotionList = null;
    state.value = "idle";
  }

  onScopeDispose(dispose);

  return Object.freeze({
    bands: (count: number, bandOptions: AudioBarsOptions = {}) =>
      toBars(frequency.value, count, {
        decibels: [toValue(options.minDecibels) ?? -100, toValue(options.maxDecibels) ?? -30],
        ...bandOptions,
      }),
    dispose,
    error: shallowReadonly(error),
    frequency: shallowReadonly(frequency),
    level: shallowReadonly(level),
    peak: shallowReadonly(peak),
    resume,
    sampleRate: shallowReadonly(sampleRate),
    state: shallowReadonly(state),
    waveform: shallowReadonly(waveform),
  });
}

function fill(analyser: AnalyserNode, frequency: AnalyserBuffer, waveform: AnalyserBuffer): void {
  if (frequency instanceof Float32Array) analyser.getFloatFrequencyData(frequency);
  else analyser.getByteFrequencyData(frequency);
  if (waveform instanceof Float32Array) analyser.getFloatTimeDomainData(waveform);
  else analyser.getByteTimeDomainData(waveform);
}

/** Test hook: forget the shared page-wide context. Not exported from the entry. */
export function resetSharedAudioContextForTesting(): void {
  sharedContext = null;
}
