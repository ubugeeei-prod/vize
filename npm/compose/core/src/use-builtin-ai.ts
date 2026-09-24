import {
  computed,
  hasInjectionContext,
  readonly,
  ref,
  shallowRef,
  toValue,
  unref,
  watch,
  watchPostEffect,
} from "vue";
import type { ComputedRef, MaybeRef, MaybeRefOrGetter, Ref, ShallowRef } from "vue";

import { tryOnScopeDispose } from "./scope.ts";

/** Model availability reported by the built-in AI APIs. */
export type BuiltinAiAvailability = "unavailable" | "downloadable" | "downloading" | "available";

/** Lifecycle of a built-in AI session owned by a composable. */
export type BuiltinAiStatus =
  | "unsupported"
  | "idle"
  | "downloading"
  | "creating"
  | "ready"
  | "error";

/** `downloadprogress` event: `loaded` is a fraction of `total` (usually 1). */
export interface BuiltinAiDownloadProgressEvent {
  /** Downloaded amount. */
  readonly loaded: number;

  /** Total amount (1 in current browsers). */
  readonly total?: number | undefined;
}

/** Monitor handed to the `monitor` callback of `create()`. */
export interface BuiltinAiMonitorLike {
  /** Subscribe to model download progress. */
  addEventListener(
    type: "downloadprogress",
    listener: (event: BuiltinAiDownloadProgressEvent) => void,
  ): void;
}

/** Options added by the composables to every `create()` call. */
export interface BuiltinAiCreateMonitorOptions {
  /** Receives the download monitor. */
  readonly monitor: (monitor: BuiltinAiMonitorLike) => void;

  /** Aborts creation when the session is destroyed. */
  readonly signal: AbortSignal;
}

/** Streaming output of a built-in AI call (a `ReadableStream<string>`). */
export type BuiltinAiTextStream =
  | AsyncIterable<string>
  | {
      /** Lock the stream and read chunks. */
      getReader(): {
        /** Read the next chunk. */
        read(): Promise<{ readonly done: boolean; readonly value?: string | undefined }>;
        /** Release the lock. */
        releaseLock(): void;
      };
    };

/** Common state and actions of every built-in AI composable. */
export interface BuiltinAiSessionControls {
  /** Whether the API's global constructor is available. */
  readonly supported: ComputedRef<boolean>;

  /** Session lifecycle. */
  readonly status: ComputedRef<BuiltinAiStatus>;

  /** Model download progress (0..1) during `create()`. */
  readonly downloadProgress: Readonly<Ref<number>>;

  /** Most recent failure, cleared by the next successful call. */
  readonly error: Readonly<ShallowRef<unknown>>;

  /**
   * Ask whether the model for the current options is usable.
   *
   * @returns The availability; `"unavailable"` when unsupported or on failure.
   */
  readonly availability: () => Promise<BuiltinAiAvailability>;

  /**
   * Create the session (downloading the model when needed). Browsers require
   * a user gesture for a download. Operations create it lazily.
   *
   * @returns Whether the session is ready.
   */
  readonly create: () => Promise<boolean>;

  /** Destroy the session and abort a pending creation. Repeated calls are safe. */
  readonly destroy: () => void;
}

/** Minimal `Translator` instance. */
export interface TranslatorLike {
  /** Translate `input`. */
  translate(input: string): Promise<string>;

  /** Translate `input`, streaming chunks. */
  translateStreaming(input: string): BuiltinAiTextStream;

  /** Release the session. */
  destroy(): void;
}

/** Language pair of a translator. */
export interface TranslatorLanguages {
  /** BCP 47 source language. */
  readonly sourceLanguage: string;

  /** BCP 47 target language. */
  readonly targetLanguage: string;
}

/** Minimal global `Translator` class (static methods). */
export interface TranslatorHost {
  /** Availability of a language pair. */
  availability(options: TranslatorLanguages): Promise<BuiltinAiAvailability>;

  /** Create a translator. */
  create(options: TranslatorLanguages & BuiltinAiCreateMonitorOptions): Promise<TranslatorLike>;
}

/** Options for {@link useTranslator}. */
export interface UseTranslatorOptions {
  /** BCP 47 source language; a change destroys the current session. */
  readonly sourceLanguage: MaybeRefOrGetter<string>;

  /** BCP 47 target language; a change destroys the current session. */
  readonly targetLanguage: MaybeRefOrGetter<string>;

  /**
   * `Translator` class. A ref (not a getter) because the host is a class.
   *
   * @default window.Translator when it exists
   */
  readonly host?: MaybeRef<TranslatorHost | null | undefined>;
}

/** State and actions returned by {@link useTranslator}. */
export interface TranslatorControls extends BuiltinAiSessionControls {
  /**
   * Translate text, creating the session on first use. Rejects with a
   * tagged `Error` when unsupported or with the host's failure.
   *
   * @param text Text to translate.
   * @returns The translation.
   */
  readonly translate: (text: string) => Promise<string>;

  /**
   * Translate text as a stream of chunks.
   *
   * @param text Text to translate.
   * @returns Async iterable of translated chunks.
   */
  readonly translateStreaming: (text: string) => AsyncIterable<string>;
}

/** One candidate returned by `LanguageDetector.detect`. */
export interface LanguageDetectionResult {
  /** BCP 47 language tag (`"und"` for unknown). */
  readonly detectedLanguage: string;

  /** Confidence between 0 and 1. */
  readonly confidence: number;
}

/** Minimal `LanguageDetector` instance. */
export interface LanguageDetectorLike {
  /** Rank candidate languages for `input`. */
  detect(input: string): Promise<readonly LanguageDetectionResult[]>;

  /** Release the session. */
  destroy(): void;
}

/** Creation options of a language detector. */
export interface LanguageDetectorCreateOptions {
  /**
   * Languages the input is expected to be in.
   *
   * @default undefined
   */
  readonly expectedInputLanguages?: readonly string[];
}

/** Minimal global `LanguageDetector` class (static methods). */
export interface LanguageDetectorHost {
  /** Availability of the detector. */
  availability(options: LanguageDetectorCreateOptions): Promise<BuiltinAiAvailability>;

  /** Create a detector. */
  create(
    options: LanguageDetectorCreateOptions & BuiltinAiCreateMonitorOptions,
  ): Promise<LanguageDetectorLike>;
}

/** Options for {@link useLanguageDetector}. */
export interface UseLanguageDetectorOptions {
  /**
   * Languages the input is expected to be in; a change destroys the session.
   *
   * @default undefined
   */
  readonly expectedInputLanguages?: MaybeRefOrGetter<readonly string[] | undefined>;

  /**
   * `LanguageDetector` class.
   *
   * @default window.LanguageDetector when it exists
   */
  readonly host?: MaybeRef<LanguageDetectorHost | null | undefined>;
}

/** State and actions returned by {@link useLanguageDetector}. */
export interface LanguageDetectorControls extends BuiltinAiSessionControls {
  /**
   * Detect the language of text, creating the session on first use.
   *
   * @param text Text to inspect.
   * @returns Candidates ordered by confidence.
   */
  readonly detect: (text: string) => Promise<readonly LanguageDetectionResult[]>;
}

/** Summary kinds. */
export type SummarizerType = "tldr" | "teaser" | "key-points" | "headline";

/** Summary output formats. */
export type SummarizerFormat = "plain-text" | "markdown";

/** Summary lengths. */
export type SummarizerLength = "short" | "medium" | "long";

/** Creation options of a summarizer. */
export interface SummarizerCreateOptions {
  /** Summary kind. */
  readonly type: SummarizerType;

  /** Output format. */
  readonly format: SummarizerFormat;

  /** Output length. */
  readonly length: SummarizerLength;

  /**
   * Background shared by every summary.
   *
   * @default undefined
   */
  readonly sharedContext?: string;
}

/** Per-call options of `summarize`. */
export interface SummarizeCallOptions {
  /**
   * Background for this input only.
   *
   * @default undefined
   */
  readonly context?: string;
}

/** Minimal `Summarizer` instance. */
export interface SummarizerLike {
  /** Summarize `input`. */
  summarize(input: string, options?: SummarizeCallOptions): Promise<string>;

  /** Summarize `input`, streaming chunks. */
  summarizeStreaming(input: string, options?: SummarizeCallOptions): BuiltinAiTextStream;

  /** Release the session. */
  destroy(): void;
}

/** Minimal global `Summarizer` class (static methods). */
export interface SummarizerHost {
  /** Availability of the summarizer. */
  availability(options: SummarizerCreateOptions): Promise<BuiltinAiAvailability>;

  /** Create a summarizer. */
  create(options: SummarizerCreateOptions & BuiltinAiCreateMonitorOptions): Promise<SummarizerLike>;
}

/** Options for {@link useSummarizer}. */
export interface UseSummarizerOptions {
  /**
   * Summary kind; a change destroys the session.
   *
   * @default "key-points"
   */
  readonly type?: MaybeRefOrGetter<SummarizerType>;

  /**
   * Output format; a change destroys the session.
   *
   * @default "markdown"
   */
  readonly format?: MaybeRefOrGetter<SummarizerFormat>;

  /**
   * Output length; a change destroys the session.
   *
   * @default "short"
   */
  readonly length?: MaybeRefOrGetter<SummarizerLength>;

  /**
   * Background shared by every summary; a change destroys the session.
   *
   * @default undefined
   */
  readonly sharedContext?: MaybeRefOrGetter<string | undefined>;

  /**
   * `Summarizer` class.
   *
   * @default window.Summarizer when it exists
   */
  readonly host?: MaybeRef<SummarizerHost | null | undefined>;
}

/** State and actions returned by {@link useSummarizer}. */
export interface SummarizerControls extends BuiltinAiSessionControls {
  /**
   * Summarize text, creating the session on first use.
   *
   * @param text Text to summarize.
   * @param options Per-call context.
   * @returns The summary.
   */
  readonly summarize: (text: string, options?: SummarizeCallOptions) => Promise<string>;

  /**
   * Summarize text as a stream of chunks.
   *
   * @param text Text to summarize.
   * @param options Per-call context.
   * @returns Async iterable of summary chunks.
   */
  readonly summarizeStreaming: (
    text: string,
    options?: SummarizeCallOptions,
  ) => AsyncIterable<string>;
}

interface AiFactory<Instance, CreateOptions> {
  availability(options: CreateOptions): Promise<BuiltinAiAvailability>;
  create(options: CreateOptions & BuiltinAiCreateMonitorOptions): Promise<Instance>;
}

interface AiSession<Instance> {
  readonly controls: BuiltinAiSessionControls;
  readonly run: <Value>(action: (instance: Instance) => Promise<Value>) => Promise<Value>;
  readonly stream: (open: (instance: Instance) => BuiltinAiTextStream) => AsyncIterable<string>;
}

// A global class cannot be verified structurally beyond its static methods;
// this guard checks those and trusts the platform for the instance shape.
function isFactory<Factory>(candidate: unknown): candidate is Factory {
  return (
    typeof candidate === "function" &&
    typeof Reflect.get(candidate, "create") === "function" &&
    typeof Reflect.get(candidate, "availability") === "function"
  );
}

function browserFactory<Factory>(name: string): Factory | undefined {
  if (typeof window === "undefined") return undefined;
  const candidate: unknown = Reflect.get(window, name);
  return isFactory<Factory>(candidate) ? candidate : undefined;
}

function useAiSession<Instance extends { destroy(): void }, CreateOptions>(
  name: string,
  host: MaybeRef<AiFactory<Instance, CreateOptions> | null | undefined> | undefined,
  createOptions: () => CreateOptions,
): AiSession<Instance> {
  const phase = ref<Exclude<BuiltinAiStatus, "unsupported">>("idle");
  const downloadProgress = ref(0);
  const error = shallowRef<unknown>(undefined);
  let instance: Instance | undefined;
  let creating: Promise<Instance> | undefined;
  let controller: AbortController | undefined;
  let generation = 0;

  // Inside a component the host is resolved only after mounting, so a
  // hydrating client renders the server's unsupported state first. Outside
  // components it resolves synchronously.
  const hydrated = shallowRef(!hasInjectionContext());
  if (!hydrated.value) {
    watchPostEffect(() => {
      hydrated.value = true;
    });
  }

  const resolveFactory = (): AiFactory<Instance, CreateOptions> | undefined =>
    host === undefined ? browserFactory(name) : (unref(host) ?? undefined);

  const fail = (cause: unknown): never => {
    error.value = cause;
    throw cause;
  };

  const ensure = (): Promise<Instance> => {
    if (instance) return Promise.resolve(instance);
    if (creating) return creating;
    const factory = resolveFactory();
    if (!factory) {
      return Promise.reject(
        new Error(`[VIZE_COMPOSE_BUILTIN_AI_UNSUPPORTED] ${name} is not available.`),
      ).catch(fail);
    }
    const current = ++generation;
    const abort = new AbortController();
    controller = abort;
    phase.value = "creating";
    downloadProgress.value = 0;
    const monitor = (target: BuiltinAiMonitorLike): void => {
      target.addEventListener("downloadprogress", ({ loaded, total }) => {
        if (current !== generation) return;
        const fraction = total !== undefined && total > 0 ? loaded / total : loaded;
        downloadProgress.value = Math.min(1, Math.max(0, fraction));
        phase.value = "downloading";
      });
    };
    let requested: Promise<Instance>;
    try {
      requested = factory.create({ ...createOptions(), monitor, signal: abort.signal });
    } catch (cause) {
      controller = undefined;
      phase.value = "error";
      error.value = cause;
      return Promise.reject(cause);
    }
    creating = requested.then(
      (created) => {
        if (current !== generation) {
          created.destroy();
          throw new DOMException("The session was destroyed.", "AbortError");
        }
        instance = created;
        creating = undefined;
        phase.value = "ready";
        downloadProgress.value = 1;
        error.value = undefined;
        return created;
      },
      (cause: unknown) => {
        if (current === generation) {
          creating = undefined;
          phase.value = "error";
          error.value = cause;
        }
        throw cause;
      },
    );
    return creating;
  };

  const destroy = (): void => {
    generation += 1;
    controller?.abort(new DOMException("The session was destroyed.", "AbortError"));
    controller = undefined;
    creating = undefined;
    instance?.destroy();
    instance = undefined;
    phase.value = "idle";
    downloadProgress.value = 0;
  };

  const run = async <Value>(action: (session: Instance) => Promise<Value>): Promise<Value> => {
    const session = await ensure();
    try {
      const value = await action(session);
      error.value = undefined;
      return value;
    } catch (cause) {
      return fail(cause);
    }
  };

  async function* stream(open: (session: Instance) => BuiltinAiTextStream): AsyncGenerator<string> {
    const session = await ensure();
    try {
      const source = open(session);
      if (Symbol.asyncIterator in source) {
        for await (const chunk of source) yield chunk;
      } else {
        const reader = source.getReader();
        try {
          for (;;) {
            const { done, value } = await reader.read();
            if (done) break;
            if (value !== undefined) yield value;
          }
        } finally {
          reader.releaseLock();
        }
      }
      error.value = undefined;
    } catch (cause) {
      fail(cause);
    }
  }

  const availability = async (): Promise<BuiltinAiAvailability> => {
    const factory = resolveFactory();
    if (!factory) return "unavailable";
    try {
      return await factory.availability(createOptions());
    } catch (cause) {
      error.value = cause;
      return "unavailable";
    }
  };

  watch(createOptions, () => {
    if (instance || creating) destroy();
  });
  tryOnScopeDispose(destroy);

  const supported = computed(() => hydrated.value && resolveFactory() !== undefined);
  return {
    controls: {
      supported,
      status: computed(() => (supported.value ? phase.value : "unsupported")),
      downloadProgress: readonly(downloadProgress),
      error: readonly(error),
      availability,
      create: () =>
        ensure().then(
          () => true,
          () => false,
        ),
      destroy,
    },
    run,
    stream,
  };
}

/**
 * Translate text on-device with Chrome's built-in `Translator` API.
 *
 * The session is created lazily by the first `translate` (or explicitly by
 * `create()`, which browsers require inside a user gesture when the model
 * must be downloaded); `downloadProgress` follows the download. Changing a
 * language destroys the session. The session is destroyed and a pending
 * creation aborted when the owning reactive scope stops; outside a scope
 * call `destroy()`.
 *
 * Server rendering: nothing is created, `supported` is false and `status`
 * is `"unsupported"`.
 * Inside a component `supported` turns true only after mounting, so
 * hydration renders this server state first.
 *
 * @example
 * ```ts
 * const translator = useTranslator({ sourceLanguage: "en", targetLanguage: "ja" });
 * const japanese = await translator.translate("Hello");
 * ```
 *
 * @param options Language pair and class override.
 * @returns Translator state and actions.
 */
export function useTranslator(options: UseTranslatorOptions): TranslatorControls {
  const session = useAiSession<TranslatorLike, TranslatorLanguages>(
    "Translator",
    options.host,
    () => ({
      sourceLanguage: toValue(options.sourceLanguage),
      targetLanguage: toValue(options.targetLanguage),
    }),
  );
  return {
    ...session.controls,
    translate: (text) => session.run((translator) => translator.translate(text)),
    translateStreaming: (text) =>
      session.stream((translator) => translator.translateStreaming(text)),
  };
}

/**
 * Detect the language of text on-device with Chrome's built-in
 * `LanguageDetector` API.
 *
 * Shares the lifecycle of {@link useTranslator}: lazy creation, download
 * progress, destruction with the owning reactive scope.
 *
 * Server rendering: nothing is created, `supported` is false and `status`
 * is `"unsupported"`.
 * Inside a component `supported` turns true only after mounting, so
 * hydration renders this server state first.
 *
 * @example
 * ```ts
 * const detector = useLanguageDetector();
 * const [best] = await detector.detect(input.value);
 * ```
 *
 * @param options Expected languages and class override.
 * @default options {}
 * @returns Detector state and actions.
 */
export function useLanguageDetector(
  options: UseLanguageDetectorOptions = {},
): LanguageDetectorControls {
  const session = useAiSession<LanguageDetectorLike, LanguageDetectorCreateOptions>(
    "LanguageDetector",
    options.host,
    () => {
      const expectedInputLanguages = toValue(options.expectedInputLanguages);
      return expectedInputLanguages ? { expectedInputLanguages } : {};
    },
  );
  return {
    ...session.controls,
    detect: (text) => session.run((detector) => detector.detect(text)),
  };
}

/**
 * Summarize text on-device with Chrome's built-in `Summarizer` API.
 *
 * Shares the lifecycle of {@link useTranslator}: lazy creation, download
 * progress, destruction with the owning reactive scope; changing `type`,
 * `format`, `length` or `sharedContext` destroys the session.
 *
 * Server rendering: nothing is created, `supported` is false and `status`
 * is `"unsupported"`.
 * Inside a component `supported` turns true only after mounting, so
 * hydration renders this server state first.
 *
 * @example
 * ```ts
 * const summarizer = useSummarizer({ type: "tldr", length: "medium" });
 * for await (const chunk of summarizer.summarizeStreaming(article.value)) output.value += chunk;
 * ```
 *
 * @param options Summary shape and class override.
 * @default options {}
 * @returns Summarizer state and actions.
 */
export function useSummarizer(options: UseSummarizerOptions = {}): SummarizerControls {
  const session = useAiSession<SummarizerLike, SummarizerCreateOptions>(
    "Summarizer",
    options.host,
    () => {
      const base = {
        type: toValue(options.type) ?? "key-points",
        format: toValue(options.format) ?? "markdown",
        length: toValue(options.length) ?? "short",
      };
      const sharedContext = toValue(options.sharedContext);
      return sharedContext === undefined ? base : { ...base, sharedContext };
    },
  );
  return {
    ...session.controls,
    summarize: (text, callOptions) =>
      session.run((summarizer) => summarizer.summarize(text, callOptions)),
    summarizeStreaming: (text, callOptions) =>
      session.stream((summarizer) => summarizer.summarizeStreaming(text, callOptions)),
  };
}
