import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope, ref, shallowRef } from "vue";

import { renderComposableOnServer } from "./testing/ssr-harness.ts";
import { useSpeechRecognition } from "./use-speech-recognition.ts";
import type {
  SpeechRecognitionHost,
  SpeechRecognitionLike,
  SpeechRecognitionResultLike,
} from "./use-speech-recognition.ts";

const instances: FakeRecognition[] = [];

class FakeRecognition extends EventTarget implements SpeechRecognitionLike {
  lang = "";
  continuous = false;
  interimResults = false;
  maxAlternatives = 0;
  running = false;
  aborted = 0;
  listeners = 0;

  constructor() {
    super();
    instances.push(this);
  }

  override addEventListener(type: string, listener: EventListener): void {
    this.listeners += 1;
    super.addEventListener(type, listener);
  }

  override removeEventListener(type: string, listener: EventListener): void {
    this.listeners -= 1;
    super.removeEventListener(type, listener);
  }

  start(): void {
    if (this.running) throw new Error("InvalidStateError");
    this.running = true;
    this.dispatchEvent(new Event("start"));
  }

  stop(): void {
    this.running = false;
    this.dispatchEvent(new Event("end"));
  }

  abort(): void {
    this.aborted += 1;
    this.running = false;
    this.dispatchEvent(new Event("end"));
  }

  emitResults(results: readonly SpeechRecognitionResultLike[]): void {
    this.dispatchEvent(Object.assign(new Event("result"), { results }));
  }

  emitError(code: string): void {
    this.dispatchEvent(Object.assign(new Event("error"), { error: code, message: "m" }));
  }
}

function result(isFinal: boolean, ...transcripts: string[]): SpeechRecognitionResultLike {
  return Object.assign(
    transcripts.map((transcript, index) => ({ transcript, confidence: 1 - index / 10 })),
    { isFinal },
  );
}

function latest(): FakeRecognition {
  const instance = instances.at(-1);
  assert.ok(instance);
  return instance;
}

void test("applies options on start and accumulates transcripts", () => {
  const lang = ref("ja-JP");
  const speech = useSpeechRecognition({ host: FakeRecognition, lang, maxAlternatives: 2 });
  const recognizer = latest();

  assert.equal(speech.supported.value, true);
  assert.equal(speech.start(), true);
  assert.equal(recognizer.lang, "ja-JP");
  assert.equal(recognizer.continuous, true);
  assert.equal(recognizer.interimResults, true);
  assert.equal(recognizer.maxAlternatives, 2);
  assert.equal(speech.listening.value, true);

  recognizer.emitResults([result(true, "hello "), result(false, "wor", "war")]);
  assert.equal(speech.result.value, "hello ");
  assert.equal(speech.interimResult.value, "wor");
  assert.deepEqual(
    speech.alternatives.value.map((alternative) => alternative.transcript),
    ["wor", "war"],
  );

  speech.stop();
  assert.equal(speech.listening.value, false);
  assert.equal(speech.interimResult.value, "");
  assert.equal(speech.result.value, "hello ");
});

void test("clears results when a new session starts", () => {
  const speech = useSpeechRecognition({ host: FakeRecognition });
  const recognizer = latest();
  speech.start();
  recognizer.emitResults([result(true, "one")]);
  speech.stop();
  speech.start();
  assert.equal(speech.result.value, "");
});

void test("normalizes error codes", () => {
  const speech = useSpeechRecognition({ host: FakeRecognition });
  const recognizer = latest();
  recognizer.emitError("not-allowed");
  assert.deepEqual(speech.error.value, { code: "not-allowed", message: "m" });
  recognizer.emitError("brand-new-code");
  assert.equal(speech.error.value?.code, "unknown");
  speech.start();
  assert.equal(speech.error.value, undefined);
});

void test("toggles, aborts, and reports an already-running recognizer", () => {
  const speech = useSpeechRecognition({ host: FakeRecognition });
  const recognizer = latest();
  speech.toggle();
  assert.equal(speech.listening.value, true);
  assert.equal(speech.start(), true, "starting while listening is a no-op");
  speech.toggle();
  assert.equal(speech.listening.value, false);

  speech.toggle(true);
  speech.abort();
  assert.equal(recognizer.aborted, 1);
  assert.equal(speech.listening.value, false);

  recognizer.running = true;
  assert.equal(speech.start(), false);
});

void test("recreates the recognizer when the host ref changes", () => {
  const host = shallowRef<SpeechRecognitionHost | null>(FakeRecognition);
  const speech = useSpeechRecognition({ host });
  const first = latest();
  speech.start();

  host.value = null;
  assert.equal(first.aborted, 1);
  assert.equal(first.listeners, 0);
  assert.equal(speech.supported.value, false);
  assert.equal(speech.start(), false);

  host.value = FakeRecognition;
  assert.notEqual(latest(), first);
  assert.equal(speech.start(), true);
});

void test("aborts and detaches with the scope", () => {
  const scope = effectScope();
  const speech = scope.run(() => useSpeechRecognition({ host: FakeRecognition }));
  assert.ok(speech);
  const recognizer = latest();
  speech.start();
  scope.stop();
  assert.equal(recognizer.aborted, 1);
  assert.equal(recognizer.listeners, 0);
});

void test("server rendering creates no recognizer", async () => {
  const state = await renderComposableOnServer(() => {
    const speech = useSpeechRecognition();
    return {
      supported: speech.supported,
      listening: speech.listening,
      result: speech.result,
    };
  });
  assert.equal(state, '{"supported":false,"listening":false,"result":""}');
});
