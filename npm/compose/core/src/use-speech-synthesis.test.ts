import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope, ref } from "vue";

import { renderComposableOnServer } from "./testing/ssr-harness.ts";
import { useSpeechSynthesis } from "./use-speech-synthesis.ts";
import type {
  SpeechSynthesisHost,
  SpeechSynthesisLike,
  SpeechSynthesisUtteranceLike,
  SpeechSynthesisVoiceLike,
} from "./use-speech-synthesis.ts";

class FakeUtterance extends EventTarget implements SpeechSynthesisUtteranceLike {
  text: string;
  lang = "";
  pitch = 1;
  rate = 1;
  volume = 1;
  voice: SpeechSynthesisVoiceLike | null = null;

  constructor(text = "") {
    super();
    this.text = text;
  }

  emit(type: string, error?: string): void {
    this.dispatchEvent(Object.assign(new Event(type), error === undefined ? {} : { error }));
  }
}

class FakeSynthesis extends EventTarget implements SpeechSynthesisLike {
  readonly queue: FakeUtterance[] = [];
  voiceList: SpeechSynthesisVoiceLike[] = [];
  cancelled = 0;
  paused = 0;
  resumed = 0;

  speak(utterance: SpeechSynthesisUtteranceLike): void {
    if (utterance instanceof FakeUtterance) this.queue.push(utterance);
  }

  cancel(): void {
    this.cancelled += 1;
    const active = this.queue.splice(0);
    for (const utterance of active) utterance.emit("error", "interrupted");
  }

  pause(): void {
    this.paused += 1;
  }

  resume(): void {
    this.resumed += 1;
  }

  getVoices(): readonly SpeechSynthesisVoiceLike[] {
    return this.voiceList;
  }
}

function createHost(): { host: SpeechSynthesisHost; synthesis: FakeSynthesis } {
  const synthesis = new FakeSynthesis();
  return { host: { synthesis, Utterance: FakeUtterance }, synthesis };
}

const voice: SpeechSynthesisVoiceLike = {
  name: "Kyoko",
  lang: "ja-JP",
  voiceURI: "kyoko",
  localService: true,
  default: false,
};

void test("speaks the current text with the configured parameters", () => {
  const { host, synthesis } = createHost();
  const text = ref("first");
  const speech = useSpeechSynthesis(text, { host, lang: "ja-JP", pitch: 2, rate: 0.5, voice });

  text.value = "second";
  assert.equal(speech.speak(), true);
  const utterance = synthesis.queue[0];
  assert.ok(utterance);
  assert.equal(utterance.text, "second");
  assert.equal(utterance.lang, "ja-JP");
  assert.equal(utterance.pitch, 2);
  assert.equal(utterance.rate, 0.5);
  assert.equal(utterance.voice, voice);

  utterance.emit("start");
  assert.equal(speech.status.value, "speaking");
  speech.pause();
  utterance.emit("pause");
  assert.equal(speech.status.value, "paused");
  speech.resume();
  utterance.emit("resume");
  assert.equal(speech.status.value, "speaking");
  assert.equal(synthesis.paused, 1);
  assert.equal(synthesis.resumed, 1);
  utterance.emit("end");
  assert.equal(speech.status.value, "ended");
});

void test("treats cancellation as idle and real failures as errors", () => {
  const { host, synthesis } = createHost();
  const speech = useSpeechSynthesis("x", { host });

  speech.speak();
  synthesis.queue[0]?.emit("start");
  speech.cancel();
  assert.equal(speech.status.value, "idle");
  assert.equal(speech.error.value, undefined);

  speech.speak();
  synthesis.queue.at(-1)?.emit("error", "network");
  assert.equal(speech.status.value, "error");
  assert.equal(speech.error.value, "network");

  speech.speak();
  assert.equal(speech.error.value, undefined);
  synthesis.queue.at(-1)?.emit("error", "future-code");
  assert.equal(speech.error.value, "unknown");
});

void test("ignores events from a replaced utterance", () => {
  const { host, synthesis } = createHost();
  const speech = useSpeechSynthesis("x", { host });
  speech.speak();
  const first = synthesis.queue[0];
  speech.speak();
  first?.emit("end");
  assert.notEqual(speech.status.value, "ended");
});

void test("tracks voices through voiceschanged", () => {
  const { host, synthesis } = createHost();
  const speech = useSpeechSynthesis("x", { host });
  assert.deepEqual(speech.voices.value, []);
  synthesis.voiceList = [voice];
  synthesis.dispatchEvent(new Event("voiceschanged"));
  assert.deepEqual(speech.voices.value, [voice]);
});

void test("validates pitch, rate, and volume", () => {
  const { host } = createHost();
  assert.throws(
    () => useSpeechSynthesis("x", { host, pitch: 3 }),
    /VIZE_COMPOSE_SPEECH_SYNTHESIS_INVALID_OPTION/,
  );
  assert.throws(() => useSpeechSynthesis("x", { host, rate: 0 }), /rate/);
  const volume = ref(1);
  const speech = useSpeechSynthesis("x", { host, volume });
  volume.value = 2;
  assert.throws(() => speech.speak(), /volume/);
});

void test("cancels own speech and detaches with the scope", () => {
  const { host, synthesis } = createHost();
  const scope = effectScope();
  const speech = scope.run(() => useSpeechSynthesis("x", { host }));
  assert.ok(speech);
  speech.speak();
  scope.stop();
  assert.equal(synthesis.cancelled, 1);
  synthesis.voiceList = [voice];
  synthesis.dispatchEvent(new Event("voiceschanged"));
  assert.deepEqual(speech.voices.value, []);
});

void test("reports unsupported without a host", () => {
  const speech = useSpeechSynthesis("x", { host: null });
  assert.equal(speech.supported.value, false);
  assert.equal(speech.speak(), false);
});

void test("server rendering speaks nothing", async () => {
  const state = await renderComposableOnServer(() => {
    const speech = useSpeechSynthesis("hello");
    return { supported: speech.supported, status: speech.status, voices: speech.voices };
  });
  assert.equal(state, '{"supported":false,"status":"idle","voices":[]}');
});
