import { computed, readonly, ref, shallowRef, toValue, unref, watch } from "vue";
import type { ComputedRef, MaybeRef, MaybeRefOrGetter, Ref, ShallowRef } from "vue";

/** One recognition hypothesis. */
export interface SpeechRecognitionAlternativeLike {
  /** Recognized text. */
  readonly transcript: string;

  /** Confidence between 0 and 1. */
  readonly confidence: number;
}

/** One recognition result: an array-like list of alternatives. */
export interface SpeechRecognitionResultLike {
  /** Number of alternatives. */
  readonly length: number;

  /** Whether the result is final (no longer interim). */
  readonly isFinal: boolean;

  /** Alternative at `index`, most likely first. */
  readonly [index: number]: SpeechRecognitionAlternativeLike;
}

/** Array-like list of every result of the current session. */
export interface SpeechRecognitionResultListLike {
  /** Number of results. */
  readonly length: number;

  /** Result at `index`. */
  readonly [index: number]: SpeechRecognitionResultLike;
}

/** Minimal `SpeechRecognition` instance consumed by {@link useSpeechRecognition}. */
export interface SpeechRecognitionLike extends EventTarget {
  /** BCP 47 language tag. */
  lang: string;

  /** Keep listening after the first final result. */
  continuous: boolean;

  /** Report interim (non-final) results. */
  interimResults: boolean;

  /** Maximum alternatives per result. */
  maxAlternatives: number;

  /** Start listening. Throws `InvalidStateError` when already started. */
  start(): void;

  /** Stop listening and deliver a final result. */
  stop(): void;

  /** Stop listening without delivering a result. */
  abort(): void;
}

/** Minimal `SpeechRecognition` constructor. */
export interface SpeechRecognitionHost {
  /** Create a recognizer. */
  new (): SpeechRecognitionLike;
}

/** Error codes defined by the Web Speech API, plus `"unknown"` for future codes. */
export type SpeechRecognitionErrorCode =
  | "aborted"
  | "audio-capture"
  | "bad-grammar"
  | "language-not-supported"
  | "network"
  | "no-speech"
  | "not-allowed"
  | "phrases-not-supported"
  | "service-not-allowed"
  | "unknown";

/** Recognition failure reported by {@link useSpeechRecognition}. */
export interface SpeechRecognitionFailure {
  /** Normalized error code. */
  readonly code: SpeechRecognitionErrorCode;

  /** Implementation-provided message, possibly empty. */
  readonly message: string;
}

/** Options for {@link useSpeechRecognition}. */
export interface UseSpeechRecognitionOptions {
  /**
   * Recognition language, applied when listening starts.
   *
   * @default "en-US"
   */
  readonly lang?: MaybeRefOrGetter<string>;

  /**
   * Keep listening after the first final result.
   *
   * @default true
   */
  readonly continuous?: boolean;

  /**
   * Report interim results through `interimResult`.
   *
   * @default true
   */
  readonly interimResults?: boolean;

  /**
   * Maximum alternatives per result.
   *
   * @default 1
   */
  readonly maxAlternatives?: number;

  /**
   * Recognizer constructor for alternate runtimes and tests. A ref (not a
   * getter) because the host itself is a constructor function.
   *
   * @default window.SpeechRecognition ?? window.webkitSpeechRecognition
   */
  readonly host?: MaybeRef<SpeechRecognitionHost | null | undefined>;
}

/** Reactive state and actions returned by {@link useSpeechRecognition}. */
export interface SpeechRecognitionControls {
  /** Whether a speech recognizer is available. */
  readonly supported: ComputedRef<boolean>;

  /** Whether the recognizer is listening. */
  readonly listening: Readonly<Ref<boolean>>;

  /** Concatenated final transcript of the current session. */
  readonly result: Readonly<Ref<string>>;

  /** Concatenated interim transcript not yet final. */
  readonly interimResult: Readonly<Ref<string>>;

  /** Alternatives of the most recent result. */
  readonly alternatives: Readonly<ShallowRef<readonly SpeechRecognitionAlternativeLike[]>>;

  /** Most recent recognition failure, cleared when listening starts. */
  readonly error: Readonly<ShallowRef<SpeechRecognitionFailure | undefined>>;

  /**
   * Start listening with the current options, clearing previous results.
   *
   * @returns Whether listening started.
   */
  readonly start: () => boolean;

  /** Stop listening and keep the final result. */
  readonly stop: () => void;

  /** Stop listening immediately without a final result. */
  readonly abort: () => void;

  /**
   * Start or stop listening.
   *
   * @param value Force listening on or off.
   * @default value !listening
   */
  readonly toggle: (value?: boolean) => void;
}

const errorCodes: ReadonlySet<string> = new Set<SpeechRecognitionErrorCode>([
  "aborted",
  "audio-capture",
  "bad-grammar",
  "language-not-supported",
  "network",
  "no-speech",
  "not-allowed",
  "phrases-not-supported",
  "service-not-allowed",
]);

function isErrorCode(code: string): code is SpeechRecognitionErrorCode {
  return errorCodes.has(code);
}

function isRecognitionConstructor(candidate: unknown): candidate is SpeechRecognitionHost {
  return typeof candidate === "function";
}

function browserRecognitionHost(): SpeechRecognitionHost | undefined {
  if (typeof window === "undefined") return undefined;
  const candidate: unknown =
    Reflect.get(window, "SpeechRecognition") ?? Reflect.get(window, "webkitSpeechRecognition");
  return isRecognitionConstructor(candidate) ? candidate : undefined;
}

interface ResultEventLike extends Event {
  readonly results: SpeechRecognitionResultListLike;
}

function isResultEvent(event: Event): event is ResultEventLike {
  if (!("results" in event)) return false;
  const { results } = event;
  return typeof results === "object" && results !== null && "length" in results;
}

function readString(event: Event, key: "error" | "message"): string {
  const value: unknown = key in event ? Reflect.get(event, key) : undefined;
  return typeof value === "string" ? value : "";
}

/**
 * Transcribe speech with the Web Speech API.
 *
 * `result` accumulates the final transcript of the current session and
 * `interimResult` the text still being recognized. Options are applied each
 * time listening starts, so a reactive `lang` takes effect on the next
 * `start`. Errors are normalized to a closed {@link SpeechRecognitionErrorCode}
 * union and exposed through `error` rather than thrown. The recognizer is
 * aborted and its listeners removed when the owning reactive scope stops.
 *
 * Server rendering: no recognizer is created; `supported` and `listening`
 * are false and the transcripts are empty.
 *
 * @example
 * ```ts
 * const speech = useSpeechRecognition({ lang: "ja-JP" });
 * speech.start();
 * watch(speech.result, (text) => console.log(text));
 * ```
 *
 * @param options Recognition settings and recognizer constructor.
 * @default options {}
 * @returns Recognition state and actions.
 */
export function useSpeechRecognition(
  options: UseSpeechRecognitionOptions = {},
): SpeechRecognitionControls {
  const listening = ref(false);
  const result = ref("");
  const interimResult = ref("");
  const alternatives = shallowRef<readonly SpeechRecognitionAlternativeLike[]>([]);
  const error = shallowRef<SpeechRecognitionFailure | undefined>(undefined);
  let recognition: SpeechRecognitionLike | undefined;

  const resolveHost = (): SpeechRecognitionHost | undefined =>
    options.host === undefined ? browserRecognitionHost() : (unref(options.host) ?? undefined);

  const onResult = (event: Event): void => {
    if (!isResultEvent(event)) return;
    const finals: string[] = [];
    const interim: string[] = [];
    let last: SpeechRecognitionResultLike | undefined;
    for (let index = 0; index < event.results.length; index += 1) {
      const entry = event.results[index];
      if (!entry) continue;
      last = entry;
      const best = entry[0];
      if (best) (entry.isFinal ? finals : interim).push(best.transcript);
    }
    result.value = finals.join("");
    interimResult.value = interim.join("");
    const collected: SpeechRecognitionAlternativeLike[] = [];
    for (let index = 0; last && index < last.length; index += 1) {
      const alternative = last[index];
      if (alternative) collected.push(alternative);
    }
    alternatives.value = collected;
  };

  const onError = (event: Event): void => {
    const code = readString(event, "error");
    error.value = {
      code: isErrorCode(code) ? code : "unknown",
      message: readString(event, "message"),
    };
  };

  const onStart = (): void => {
    listening.value = true;
  };

  const onEnd = (): void => {
    listening.value = false;
    interimResult.value = "";
  };

  watch(
    resolveHost,
    (Host, _previous, onCleanup) => {
      listening.value = false;
      recognition = undefined;
      if (!Host) return;
      const instance = new Host();
      instance.addEventListener("result", onResult);
      instance.addEventListener("error", onError);
      instance.addEventListener("start", onStart);
      instance.addEventListener("end", onEnd);
      recognition = instance;
      onCleanup(() => {
        instance.removeEventListener("result", onResult);
        instance.removeEventListener("error", onError);
        instance.removeEventListener("start", onStart);
        instance.removeEventListener("end", onEnd);
        if (listening.value) instance.abort();
        listening.value = false;
        if (recognition === instance) recognition = undefined;
      });
    },
    { immediate: true, flush: "sync" },
  );

  const start = (): boolean => {
    const instance = recognition;
    if (!instance || listening.value) return listening.value;
    instance.lang = toValue(options.lang) ?? "en-US";
    instance.continuous = options.continuous ?? true;
    instance.interimResults = options.interimResults ?? true;
    instance.maxAlternatives = options.maxAlternatives ?? 1;
    result.value = "";
    interimResult.value = "";
    alternatives.value = [];
    error.value = undefined;
    try {
      instance.start();
    } catch {
      // InvalidStateError: the recognizer is already running.
      return false;
    }
    listening.value = true;
    return true;
  };

  const stop = (): void => {
    recognition?.stop();
  };

  const abort = (): void => {
    recognition?.abort();
    listening.value = false;
  };

  const toggle = (value: boolean = !listening.value): void => {
    if (value) start();
    else stop();
  };

  return {
    supported: computed(() => resolveHost() !== undefined),
    listening: readonly(listening),
    result: readonly(result),
    interimResult: readonly(interimResult),
    alternatives,
    error,
    start,
    stop,
    abort,
    toggle,
  };
}
