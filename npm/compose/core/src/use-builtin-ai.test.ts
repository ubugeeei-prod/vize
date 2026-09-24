import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope, nextTick, ref } from "vue";

import { renderComposableOnServer } from "./testing/ssr-harness.ts";
import { useLanguageDetector, useSummarizer, useTranslator } from "./use-builtin-ai.ts";
import type {
  BuiltinAiAvailability,
  BuiltinAiCreateMonitorOptions,
  BuiltinAiDownloadProgressEvent,
  BuiltinAiTextStream,
  LanguageDetectionResult,
  LanguageDetectorCreateOptions,
  SummarizeCallOptions,
  SummarizerCreateOptions,
  TranslatorLanguages,
} from "./use-builtin-ai.ts";

class FakeMonitor {
  readonly listeners: ((event: BuiltinAiDownloadProgressEvent) => void)[] = [];

  addEventListener(
    _type: "downloadprogress",
    listener: (event: BuiltinAiDownloadProgressEvent) => void,
  ): void {
    this.listeners.push(listener);
  }

  emit(loaded: number): void {
    for (const listener of this.listeners) listener({ loaded, total: 1 });
  }
}

function chunks(values: readonly string[]): BuiltinAiTextStream {
  return new ReadableStream<string>({
    start(controller) {
      for (const value of values) controller.enqueue(value);
      controller.close();
    },
  });
}

class FakeTranslator {
  static created: FakeTranslator[] = [];
  static availabilityResult: BuiltinAiAvailability = "available";
  static lastMonitor: FakeMonitor | undefined;
  static gate: Promise<void> = Promise.resolve();
  static failure: unknown = undefined;

  destroyed = false;
  readonly options: TranslatorLanguages;

  constructor(options: TranslatorLanguages) {
    this.options = options;
  }

  static availability(_options: TranslatorLanguages): Promise<BuiltinAiAvailability> {
    return Promise.resolve(FakeTranslator.availabilityResult);
  }

  static async create(
    options: TranslatorLanguages & BuiltinAiCreateMonitorOptions,
  ): Promise<FakeTranslator> {
    const monitor = new FakeMonitor();
    FakeTranslator.lastMonitor = monitor;
    options.monitor(monitor);
    await FakeTranslator.gate;
    if (FakeTranslator.failure !== undefined) throw FakeTranslator.failure;
    options.signal.throwIfAborted();
    const translator = new FakeTranslator({
      sourceLanguage: options.sourceLanguage,
      targetLanguage: options.targetLanguage,
    });
    FakeTranslator.created.push(translator);
    return translator;
  }

  translate(input: string): Promise<string> {
    return Promise.resolve(`${this.options.targetLanguage}:${input}`);
  }

  translateStreaming(input: string): BuiltinAiTextStream {
    return chunks([`${this.options.targetLanguage}:`, input]);
  }

  destroy(): void {
    this.destroyed = true;
  }

  static reset(): void {
    FakeTranslator.created = [];
    FakeTranslator.availabilityResult = "available";
    FakeTranslator.gate = Promise.resolve();
    FakeTranslator.failure = undefined;
  }
}

class FakeLanguageDetector {
  static lastOptions: LanguageDetectorCreateOptions | undefined;

  static availability(): Promise<BuiltinAiAvailability> {
    return Promise.reject(new Error("probe failed"));
  }

  static create(
    options: LanguageDetectorCreateOptions & BuiltinAiCreateMonitorOptions,
  ): Promise<FakeLanguageDetector> {
    FakeLanguageDetector.lastOptions = options;
    return Promise.resolve(new FakeLanguageDetector());
  }

  detect(input: string): Promise<readonly LanguageDetectionResult[]> {
    return input === "fail"
      ? Promise.reject(new Error("detect failed"))
      : Promise.resolve([
          { detectedLanguage: "fr", confidence: 0.9 },
          { detectedLanguage: "und", confidence: 0.1 },
        ]);
  }

  destroy(): void {}
}

class FakeSummarizer {
  static lastOptions: SummarizerCreateOptions | undefined;

  static availability(options: SummarizerCreateOptions): Promise<BuiltinAiAvailability> {
    return Promise.resolve(options.length === "long" ? "downloadable" : "available");
  }

  static create(
    options: SummarizerCreateOptions & BuiltinAiCreateMonitorOptions,
  ): Promise<FakeSummarizer> {
    FakeSummarizer.lastOptions = options;
    return Promise.resolve(new FakeSummarizer());
  }

  summarize(input: string, options?: SummarizeCallOptions): Promise<string> {
    return Promise.resolve(`${options?.context ?? ""}${input.slice(0, 3)}`);
  }

  summarizeStreaming(input: string): BuiltinAiTextStream {
    return (async function* () {
      yield input.slice(0, 1);
      yield input.slice(1, 2);
    })();
  }

  destroy(): void {}
}

async function collect(source: AsyncIterable<string>): Promise<string[]> {
  const values: string[] = [];
  for await (const value of source) values.push(value);
  return values;
}

void test("translator creates lazily, translates and streams", async () => {
  FakeTranslator.reset();
  const translator = useTranslator({
    sourceLanguage: "en",
    targetLanguage: "ja",
    host: FakeTranslator,
  });
  assert.equal(translator.supported.value, true);
  assert.equal(translator.status.value, "idle");
  assert.equal(await translator.availability(), "available");
  assert.equal(await translator.translate("hi"), "ja:hi");
  assert.equal(translator.status.value, "ready");
  assert.equal(translator.downloadProgress.value, 1);
  assert.deepEqual(await collect(translator.translateStreaming("yo")), ["ja:", "yo"]);
  assert.equal(FakeTranslator.created.length, 1, "the session is reused");
});

void test("download progress and status follow the monitor", async () => {
  FakeTranslator.reset();
  let release = (): void => {};
  FakeTranslator.gate = new Promise((resolve) => {
    release = resolve;
  });
  const translator = useTranslator({
    sourceLanguage: "en",
    targetLanguage: "de",
    host: FakeTranslator,
  });
  const created = translator.create();
  assert.equal(translator.status.value, "creating");
  FakeTranslator.lastMonitor?.emit(0.25);
  assert.equal(translator.downloadProgress.value, 0.25);
  assert.equal(translator.status.value, "downloading");
  FakeTranslator.lastMonitor?.emit(3);
  assert.equal(translator.downloadProgress.value, 1, "progress is clamped");
  release();
  assert.equal(await created, true);
  assert.equal(translator.status.value, "ready");
});

void test("language changes destroy the session", async () => {
  FakeTranslator.reset();
  const target = ref("ja");
  const translator = useTranslator({
    sourceLanguage: "en",
    targetLanguage: target,
    host: FakeTranslator,
  });
  await translator.translate("a");
  target.value = "ko";
  await nextTick();
  assert.equal(FakeTranslator.created[0]?.destroyed, true);
  assert.equal(translator.status.value, "idle");
  assert.equal(await translator.translate("a"), "ko:a");
});

void test("creation failures land in error and status", async () => {
  FakeTranslator.reset();
  const failure = new Error("NotAllowedError");
  FakeTranslator.failure = failure;
  const translator = useTranslator({
    sourceLanguage: "en",
    targetLanguage: "ja",
    host: FakeTranslator,
  });
  assert.equal(await translator.create(), false);
  assert.equal(translator.status.value, "error");
  assert.equal(translator.error.value, failure);
  await assert.rejects(translator.translate("x"), (cause) => cause === failure);
  FakeTranslator.failure = undefined;
  assert.equal(await translator.translate("x"), "ja:x");
  assert.equal(translator.error.value, undefined);
});

void test("synchronous host creation failures keep the promise contract", async () => {
  const failure = new Error("create threw before returning a promise");
  const translator = useTranslator({
    sourceLanguage: "en",
    targetLanguage: "ja",
    host: {
      availability: () => Promise.resolve("available" as const),
      create: () => {
        throw failure;
      },
    },
  });

  assert.equal(await translator.create(), false);
  assert.equal(translator.status.value, "error");
  assert.equal(translator.error.value, failure);
  await assert.rejects(translator.translate("hi"), (cause) => cause === failure);
});

void test("scope disposal destroys the session and aborts creation", async () => {
  FakeTranslator.reset();
  const scope = effectScope();
  const translator = scope.run(() =>
    useTranslator({ sourceLanguage: "en", targetLanguage: "ja", host: FakeTranslator }),
  );
  assert.ok(translator);
  await translator.translate("x");
  scope.stop();
  assert.equal(FakeTranslator.created[0]?.destroyed, true);

  let release = (): void => {};
  FakeTranslator.gate = new Promise((resolve) => {
    release = resolve;
  });
  const pending = useTranslator({
    sourceLanguage: "en",
    targetLanguage: "ja",
    host: FakeTranslator,
  });
  const created = pending.create();
  pending.destroy();
  release();
  assert.equal(await created, false);
  assert.equal(pending.status.value, "idle");
});

void test("unsupported hosts report unavailable and reject operations", async () => {
  const translator = useTranslator({ sourceLanguage: "en", targetLanguage: "ja", host: null });
  assert.equal(translator.supported.value, false);
  assert.equal(translator.status.value, "unsupported");
  assert.equal(await translator.availability(), "unavailable");
  await assert.rejects(translator.translate("x"), /VIZE_COMPOSE_BUILTIN_AI_UNSUPPORTED/);
  await assert.rejects(collect(translator.translateStreaming("x")), /Translator is not available/);
});

void test("language detector returns typed candidates", async () => {
  const detector = useLanguageDetector({
    host: FakeLanguageDetector,
    expectedInputLanguages: ["fr", "en"],
  });
  assert.equal(await detector.availability(), "unavailable");
  assert.ok(detector.error.value instanceof Error);
  const [best] = await detector.detect("bonjour");
  assert.deepEqual(best, { detectedLanguage: "fr", confidence: 0.9 });
  assert.deepEqual(FakeLanguageDetector.lastOptions?.expectedInputLanguages, ["fr", "en"]);
  await assert.rejects(detector.detect("fail"), /detect failed/);
  assert.ok(detector.error.value instanceof Error);
});

void test("summarizer forwards options, context and streaming", async () => {
  const summarizer = useSummarizer({ type: "tldr", sharedContext: "news", host: FakeSummarizer });
  assert.equal(await summarizer.summarize("abcdef", { context: ">" }), ">abc");
  assert.equal(FakeSummarizer.lastOptions?.type, "tldr");
  assert.equal(FakeSummarizer.lastOptions?.format, "markdown");
  assert.equal(FakeSummarizer.lastOptions?.length, "short");
  assert.equal(FakeSummarizer.lastOptions?.sharedContext, "news");
  assert.deepEqual(await collect(summarizer.summarizeStreaming("xy")), ["x", "y"]);

  const long = useSummarizer({ length: "long", host: FakeSummarizer });
  assert.equal(await long.availability(), "downloadable");
  await long.summarize("abc");
  assert.equal("sharedContext" in (FakeSummarizer.lastOptions ?? {}), false);
});

void test("server rendering creates nothing", async () => {
  const state = await renderComposableOnServer(() => {
    const translator = useTranslator({ sourceLanguage: "en", targetLanguage: "ja" });
    const detector = useLanguageDetector();
    const summarizer = useSummarizer();
    return {
      supported: translator.supported,
      status: translator.status,
      downloadProgress: translator.downloadProgress,
      detector: detector.status,
      summarizer: summarizer.status,
    };
  });
  assert.equal(
    state,
    '{"supported":false,"status":"unsupported","downloadProgress":0,"detector":"unsupported","summarizer":"unsupported"}',
  );
});
