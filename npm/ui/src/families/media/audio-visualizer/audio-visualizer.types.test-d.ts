/** Compile-only assertions for the public AudioVisualizer contract. */

import { effectScope } from "vue";

import {
  AudioVisualizer,
  AudioVisualizerBars,
  toBars,
  toWaveformPath,
  useAudioAnalyser,
  type AudioAnalyser,
  type AudioAnalyserData,
  type AudioAnalyserPrecision,
  type AudioAnalyserState,
  type AudioVisualizerBarsExpose,
  type AudioVisualizerBarsSlotState,
  type AudioVisualizerExpose,
} from "./audio-visualizer.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const element: HTMLAudioElement;
const scope = effectScope();
const byte = scope.run(() => useAudioAnalyser({ source: element }));
const float = scope.run(() => useAudioAnalyser({ source: element, precision: "float" }));

type _PrecisionIsLiteral = Expect<Equal<AudioAnalyserPrecision, "byte" | "float">>;
type _ByteData = Expect<Equal<AudioAnalyserData<"byte">, Uint8Array<ArrayBuffer>>>;
type _FloatData = Expect<Equal<AudioAnalyserData<"float">, Float32Array<ArrayBuffer>>>;
type _DefaultIsByte = Expect<Equal<typeof byte, AudioAnalyser<"byte"> | undefined>>;
type _FloatInferred = Expect<Equal<typeof float, AudioAnalyser<"float"> | undefined>>;
type _FloatFrequency = Expect<
  Equal<NonNullable<typeof float>["frequency"]["value"], Float32Array<ArrayBuffer>>
>;
type _StateIsLiteral = Expect<
  Equal<AudioAnalyserState, "error" | "idle" | "paused" | "running" | "suspended" | "unsupported">
>;
type _ResumeResolvesBoolean = Expect<Equal<AudioAnalyser["resume"], () => Promise<boolean>>>;
type _BarsAreReadonly = Expect<Equal<ReturnType<typeof toBars>, readonly number[]>>;
type _PathIsString = Expect<Equal<ReturnType<typeof toWaveformPath>, string>>;
type _BarsSlot = Expect<Equal<AudioVisualizerBarsSlotState["bars"], readonly number[]>>;
type _ExposeElement = Expect<Equal<AudioVisualizerExpose["element"], HTMLDivElement | null>>;
type _BarsExposeElement = Expect<
  Equal<AudioVisualizerBarsExpose["element"], HTMLDivElement | null>
>;

void AudioVisualizer;
void AudioVisualizerBars;

// @ts-expect-error precision is a closed union.
scope.run(() => useAudioAnalyser({ source: element, precision: "double" }));
// @ts-expect-error sources must be media elements, streams, or audio nodes.
scope.run(() => useAudioAnalyser({ source: "track.mp3" }));
// @ts-expect-error analyser data refs are read-only.
if (byte) byte.level.value = 1;
