import { computed, readonly, ref, shallowRef, toValue, watch } from "vue";
import type { ComputedRef, MaybeRefOrGetter, Ref, ShallowRef } from "vue";

import { tryOnScopeDispose } from "./scope.ts";

/** Minimal `SpeechSynthesisVoice`. */
export interface SpeechSynthesisVoiceLike {
  /** Human-readable voice name. */
  readonly name: string;

  /** BCP 47 language tag. */
  readonly lang: string;

  /** Stable voice identifier. */
  readonly voiceURI: string;

  /** Whether synthesis runs locally. */
  readonly localService: boolean;

  /** Whether this is the platform default voice. */
  readonly default: boolean;
}

/** Minimal `SpeechSynthesisUtterance`. */
export interface SpeechSynthesisUtteranceLike extends EventTarget {
  /** Text to speak. */
  text: string;

  /** BCP 47 language tag. */
  lang: string;

  /** Pitch from 0 to 2. */
  pitch: number;

  /** Rate from 0.1 to 10. */
  rate: number;

  /** Volume from 0 to 1. */
  volume: number;

  /** Voice, or `null` for the platform default. */
  voice: SpeechSynthesisVoiceLike | null;
}

/** Minimal `speechSynthesis` controller. */
export interface SpeechSynthesisLike extends EventTarget {
  /** Queue an utterance. */
  speak(utterance: SpeechSynthesisUtteranceLike): void;

  /** Clear the queue and stop speaking. */
  cancel(): void;

  /** Pause speaking. */
  pause(): void;

  /** Resume speaking. */
  resume(): void;

  /** Voices available right now (may be empty until `voiceschanged`). */
  getVoices(): readonly SpeechSynthesisVoiceLike[];
}

/** Capabilities used by {@link useSpeechSynthesis}. */
export interface SpeechSynthesisHost {
  /** Synthesis controller (`window.speechSynthesis`). */
  readonly synthesis: SpeechSynthesisLike;

  /** Utterance constructor (`window.SpeechSynthesisUtterance`). */
  readonly Utterance: new (text?: string) => SpeechSynthesisUtteranceLike;
}

/** Playback state of {@link useSpeechSynthesis}. */
export type SpeechSynthesisStatus = "idle" | "speaking" | "paused" | "ended" | "error";

/** Error codes defined by the Web Speech API, plus `"unknown"` for future codes. */
export type SpeechSynthesisErrorCode =
  | "audio-busy"
  | "audio-hardware"
  | "invalid-argument"
  | "language-unavailable"
  | "network"
  | "not-allowed"
  | "synthesis-failed"
  | "synthesis-unavailable"
  | "text-too-long"
  | "voice-unavailable"
  | "unknown";

/** Options for {@link useSpeechSynthesis}. */
export interface UseSpeechSynthesisOptions {
  /**
   * BCP 47 language tag.
   *
   * @default "en-US"
   */
  readonly lang?: MaybeRefOrGetter<string>;

  /**
   * Pitch from 0 to 2.
   *
   * @default 1
   */
  readonly pitch?: MaybeRefOrGetter<number>;

  /**
   * Rate from 0.1 to 10.
   *
   * @default 1
   */
  readonly rate?: MaybeRefOrGetter<number>;

  /**
   * Volume from 0 to 1.
   *
   * @default 1
   */
  readonly volume?: MaybeRefOrGetter<number>;

  /**
   * Voice to use; `undefined` uses the platform default for `lang`.
   *
   * @default undefined
   */
  readonly voice?: MaybeRefOrGetter<SpeechSynthesisVoiceLike | undefined>;

  /**
   * Synthesis capabilities for alternate runtimes and tests.
   *
   * @default window.speechSynthesis and window.SpeechSynthesisUtterance
   */
  readonly host?: MaybeRefOrGetter<SpeechSynthesisHost | null | undefined>;
}

/** Reactive state and actions returned by {@link useSpeechSynthesis}. */
export interface SpeechSynthesisControls {
  /** Whether speech synthesis is available. */
  readonly supported: ComputedRef<boolean>;

  /** Playback state of the utterance spoken by this composable. */
  readonly status: Readonly<Ref<SpeechSynthesisStatus>>;

  /** Voices offered by the platform, updated on `voiceschanged`. */
  readonly voices: Readonly<ShallowRef<readonly SpeechSynthesisVoiceLike[]>>;

  /** Most recent synthesis error, cleared by the next `speak`. */
  readonly error: Readonly<ShallowRef<SpeechSynthesisErrorCode | undefined>>;

  /**
   * Speak the current text, cancelling anything this composable is speaking.
   *
   * @returns Whether an utterance was queued.
   * @throws {RangeError} `[VIZE_COMPOSE_SPEECH_SYNTHESIS_INVALID_OPTION]` when
   * pitch, rate, or volume is out of range.
   */
  readonly speak: () => boolean;

  /** Pause speaking. */
  readonly pause: () => void;

  /** Resume paused speech. */
  readonly resume: () => void;

  /** Stop speaking and clear the queue. */
  readonly cancel: () => void;
}

const synthesisErrorCodes: ReadonlySet<string> = new Set<SpeechSynthesisErrorCode>([
  "audio-busy",
  "audio-hardware",
  "invalid-argument",
  "language-unavailable",
  "network",
  "not-allowed",
  "synthesis-failed",
  "synthesis-unavailable",
  "text-too-long",
  "voice-unavailable",
]);

function isSynthesisErrorCode(code: string): code is SpeechSynthesisErrorCode {
  return synthesisErrorCodes.has(code);
}

function browserSynthesisHost(): SpeechSynthesisHost | undefined {
  if (typeof window === "undefined") return undefined;
  if (!("speechSynthesis" in window) || !("SpeechSynthesisUtterance" in window)) return undefined;
  return { synthesis: window.speechSynthesis, Utterance: window.SpeechSynthesisUtterance };
}

function checkedRange(name: string, value: number, minimum: number, maximum: number): number {
  if (!Number.isFinite(value) || value < minimum || value > maximum) {
    throw new RangeError(
      `[VIZE_COMPOSE_SPEECH_SYNTHESIS_INVALID_OPTION] ${name} must be between ${String(minimum)} and ${String(maximum)}; received ${String(value)}`,
    );
  }
  return value;
}

/**
 * Speak text with the Web Speech API.
 *
 * Each `speak` creates a fresh utterance from the current reactive text,
 * language, voice, pitch, rate, and volume (validated ranges), and cancels
 * speech previously started by this composable. `status` follows the
 * utterance lifecycle; interruptions caused by `cancel` return to `"idle"`
 * instead of reporting an error. The available voices are kept current via
 * `voiceschanged`. Speech started by this composable is cancelled and the
 * listener removed when the owning reactive scope stops.
 *
 * Server rendering: nothing is spoken; `supported` is false, `status` is
 * `"idle"`, and `voices` is empty.
 *
 * @example
 * ```ts
 * const text = ref("Hello, Vize");
 * const { speak, status } = useSpeechSynthesis(text, { rate: 1.2 });
 * speak();
 * ```
 *
 * @param text Reactive text to speak.
 * @param options Voice parameters and synthesis capabilities.
 * @default options {}
 * @returns Synthesis state and actions.
 */
export function useSpeechSynthesis(
  text: MaybeRefOrGetter<string>,
  options: UseSpeechSynthesisOptions = {},
): SpeechSynthesisControls {
  const status = ref<SpeechSynthesisStatus>("idle");
  const voices = shallowRef<readonly SpeechSynthesisVoiceLike[]>([]);
  const error = shallowRef<SpeechSynthesisErrorCode | undefined>(undefined);
  let current: SpeechSynthesisUtteranceLike | undefined;

  const resolveHost = (): SpeechSynthesisHost | undefined =>
    options.host === undefined ? browserSynthesisHost() : (toValue(options.host) ?? undefined);

  const readParameters = (): { pitch: number; rate: number; volume: number } => ({
    pitch: checkedRange("pitch", toValue(options.pitch) ?? 1, 0, 2),
    rate: checkedRange("rate", toValue(options.rate) ?? 1, 0.1, 10),
    volume: checkedRange("volume", toValue(options.volume) ?? 1, 0, 1),
  });
  // Fail fast on statically invalid options.
  readParameters();

  watch(
    () => resolveHost()?.synthesis,
    (synthesis, _previous, onCleanup) => {
      voices.value = [];
      if (!synthesis) return;
      const update = (): void => {
        voices.value = [...synthesis.getVoices()];
      };
      update();
      synthesis.addEventListener("voiceschanged", update);
      onCleanup(() => synthesis.removeEventListener("voiceschanged", update));
    },
    { immediate: true, flush: "sync" },
  );

  const cancel = (): void => {
    const host = resolveHost();
    const active = current;
    current = undefined;
    if (host && active) host.synthesis.cancel();
    status.value = "idle";
  };

  const speak = (): boolean => {
    const host = resolveHost();
    const parameters = readParameters();
    if (!host) return false;
    if (current) cancel();
    const utterance = new host.Utterance(toValue(text));
    utterance.lang = toValue(options.lang) ?? "en-US";
    utterance.pitch = parameters.pitch;
    utterance.rate = parameters.rate;
    utterance.volume = parameters.volume;
    utterance.voice = toValue(options.voice) ?? null;
    const own = (): boolean => current === utterance;
    utterance.addEventListener("start", () => {
      if (own()) status.value = "speaking";
    });
    utterance.addEventListener("pause", () => {
      if (own()) status.value = "paused";
    });
    utterance.addEventListener("resume", () => {
      if (own()) status.value = "speaking";
    });
    utterance.addEventListener("end", () => {
      if (!own()) return;
      current = undefined;
      status.value = "ended";
    });
    utterance.addEventListener("error", (event) => {
      if (!own()) return;
      current = undefined;
      const code: unknown = "error" in event ? event.error : undefined;
      if (code === "canceled" || code === "interrupted") {
        status.value = "idle";
        return;
      }
      error.value = typeof code === "string" && isSynthesisErrorCode(code) ? code : "unknown";
      status.value = "error";
    });
    error.value = undefined;
    current = utterance;
    host.synthesis.speak(utterance);
    return true;
  };

  const pause = (): void => {
    if (current) resolveHost()?.synthesis.pause();
  };

  const resume = (): void => {
    if (current) resolveHost()?.synthesis.resume();
  };

  tryOnScopeDispose(() => {
    if (current) cancel();
  });

  return {
    supported: computed(() => resolveHost() !== undefined),
    status: readonly(status),
    voices,
    error,
    speak,
    pause,
    resume,
    cancel,
  };
}
