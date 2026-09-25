import assert from "node:assert/strict";

import { afterEach, beforeEach, test } from "vite-plus/test";
import { effectScope, h, nextTick, shallowRef } from "vue";

import type { AudioAnalyserSource, AudioVisualizerExpose } from "./audio-visualizer.ts";
import AudioVisualizer from "./audio-visualizer.vue";
import AudioVisualizerBars from "./audio-visualizer-bars.vue";
import {
  barEdges,
  computeLevel,
  computePeak,
  toBars,
  toWaveformPath,
} from "./audio-visualizer-math.ts";
import { resetSharedAudioContextForTesting, useAudioAnalyser } from "./audio-visualizer-runtime.ts";
import {
  FakeAudioContext,
  FakeAudioNode,
  FakeMediaStream,
  installFakeAudio,
} from "./audio-visualizer-test-utils.ts";
import type { FakeAudioEnvironment } from "./audio-visualizer-test-utils.ts";
import { mountInteraction } from "../../../testing/mount.ts";

let audio: FakeAudioEnvironment;

beforeEach(() => {
  resetSharedAudioContextForTesting();
  audio = installFakeAudio();
});

afterEach(() => {
  audio.restore();
  resetSharedAudioContextForTesting();
});

function assertClose(actual: readonly number[], expected: readonly number[]): void {
  assert.equal(actual.length, expected.length);
  actual.forEach((value, index) => {
    assert.ok(
      Math.abs(value - (expected[index] ?? Number.NaN)) < 1e-9,
      `${value} ≈ ${expected[index]}`,
    );
  });
}

async function flush(): Promise<void> {
  await Promise.resolve();
  await Promise.resolve();
  await nextTick();
}

function context(): FakeAudioContext {
  const instance = FakeAudioContext.instances[0];
  assert.ok(instance, "an audio context must be created");
  return instance;
}

test("stays idle until connection is deferred past setup, then routes a media element", async () => {
  const scope = effectScope();
  const element = document.createElement("audio");
  const analyser = scope.run(() => useAudioAnalyser({ source: element }));
  assert.ok(analyser);
  assert.equal(analyser.state.value, "idle", "setup and hydration render the server state");
  assert.equal(FakeAudioContext.instances.length, 0);

  await flush();
  const ctx = context();
  assert.equal(analyser.state.value, "suspended");
  assert.equal(analyser.sampleRate.value, 48_000);
  assert.equal(ctx.elementSources, 1);
  const analyserNode = ctx.analysers[0];
  assert.ok(analyserNode);
  assert.equal(analyserNode.reads, 1, "the first frame is read on connect");
  assert.ok(analyser.frequency.value instanceof Uint8Array);
  assert.equal(analyser.frequency.value.length, 1024);
  assert.equal(analyser.waveform.value.length, 2048);
  assert.equal(analyser.frequency.value[0], 10);
  assert.equal(analyser.level.value, 0.5);
  assert.equal(analyser.peak.value, 0.5);
  assert.equal(audio.queued(), 0, "no frames are read while suspended");

  assert.equal(await analyser.resume(), true);
  assert.equal(analyser.state.value, "running");
  audio.frame(16);
  assert.equal(analyserNode.reads, 2);
  assert.equal(analyser.frequency.value[0], 20);
  audio.frame(32);
  assert.equal(analyserNode.reads, 3);
  scope.stop();
  assert.equal(analyser.state.value, "idle");
  assert.equal(audio.queued(), 0);
});

test("media-element sources are created once per element and stay audible", async () => {
  const element = document.createElement("video");
  const scope = effectScope();
  const [first, second] =
    scope.run(() => [
      useAudioAnalyser({ source: element }),
      useAudioAnalyser({ source: element, connectToDestination: false }),
    ]) ?? [];
  await flush();
  const ctx = context();
  assert.equal(ctx.elementSources, 1, "the platform allows one source node per element");
  assert.equal(first?.state.value, "suspended");
  assert.equal(second?.state.value, "suspended");
  element.dispatchEvent(new Event("play"));
  await flush();
  assert.equal(ctx.state, "running", "playing the element resumes the context");
  assert.equal(first?.state.value, "running");
  scope.stop();
});

test("streams and nodes connect without destination routing; unknown sources are unsupported", async () => {
  const scope = effectScope();
  const source = shallowRef<AudioAnalyserSource | null>(new FakeMediaStream());
  const analyser = scope.run(() => useAudioAnalyser({ source }));
  await flush();
  const ctx = context();
  assert.equal(ctx.streamSources, 1);
  assert.equal(analyser?.state.value, "suspended");
  assert.deepEqual(ctx.destination.connections, []);

  const node = new FakeAudioNode();
  source.value = node as unknown as AudioNode;
  await flush();
  assert.equal(node.connections.length, 1, "nodes feed the analyser only");

  source.value = { getTracks: () => [] };
  await flush();
  assert.equal(analyser?.state.value, "unsupported", "stream-likes must be real MediaStreams");

  source.value = null;
  await flush();
  assert.equal(analyser?.state.value, "idle");
  scope.stop();
});

test("float precision reads decibel and sample buffers", async () => {
  const scope = effectScope();
  const analyser = scope.run(() =>
    useAudioAnalyser({ source: document.createElement("audio"), precision: "float", fftSize: 64 }),
  );
  await flush();
  assert.ok(analyser?.frequency.value instanceof Float32Array);
  assert.equal(analyser.frequency.value.length, 32);
  assert.equal(analyser.frequency.value[0], -65);
  assert.equal(analyser.level.value, 0.5);
  assert.deepEqual(analyser.bands(2, { scale: "linear" }), [0.5, 0.5]);
  assert.equal(context().analysers[0]?.floatReads, 1);
  scope.stop();
});

test("pausing, frame-rate throttling, and reactive settings control reads", async () => {
  const scope = effectScope();
  const paused = shallowRef(false);
  const fftSize = shallowRef(2048);
  const analyser = scope.run(() =>
    useAudioAnalyser({
      source: document.createElement("audio"),
      paused,
      frameRate: 10,
      fftSize,
      smoothingTimeConstant: 0.5,
      minDecibels: -90,
      maxDecibels: -10,
    }),
  );
  await flush();
  const node = context().analysers[0];
  assert.ok(node);
  assert.equal(node.smoothingTimeConstant, 0.5);
  assert.equal(node.minDecibels, -90);
  assert.equal(node.maxDecibels, -10);
  await analyser?.resume();
  for (const time of [100, 116, 132, 150, 199, 200]) audio.frame(time);
  assert.equal(node.reads, 3, "10 fps reads at 100ms and 199ms (1ms tolerance) plus connect");

  paused.value = true;
  await nextTick();
  assert.equal(analyser?.state.value, "paused");
  assert.equal(audio.queued(), 0);
  paused.value = false;
  await nextTick();
  assert.equal(analyser?.state.value, "running");

  fftSize.value = 256;
  await nextTick();
  assert.equal(node.fftSize, 256);
  audio.frame(400);
  assert.equal(analyser?.frequency.value.length, 128, "buffers follow the new fft size");
  scope.stop();
});

test("reduced motion throttles or pauses frame reads", async () => {
  audio.restore();
  audio = installFakeAudio({ reducedMotion: true });
  const scope = effectScope();
  const throttled = scope.run(() =>
    useAudioAnalyser({ source: document.createElement("audio"), reducedMotionFrameRate: 4 }),
  );
  await flush();
  await throttled?.resume();
  const node = context().analysers[0];
  assert.ok(node);
  for (const time of [100, 200, 300, 350]) audio.frame(time);
  assert.equal(node.reads, 3, "4 fps reads at 100ms and 350ms plus the connect read");
  scope.stop();

  const pausing = effectScope();
  const paused = pausing.run(() =>
    useAudioAnalyser({ source: document.createElement("audio"), reducedMotion: "pause" }),
  );
  await flush();
  await paused?.resume();
  assert.equal(paused?.state.value, "paused");
  pausing.stop();
});

test("invalid fft sizes, missing Web Audio, and connection failures are reported", async () => {
  const scope = effectScope();
  assert.throws(
    () => scope.run(() => useAudioAnalyser({ source: null, fftSize: 1000 })),
    /VIZE_UI_AUDIO_ANALYSER_FFT_SIZE/,
  );
  assert.throws(() => useAudioAnalyser({ source: null }), /VIZE_UI_AUDIO_ANALYSER_SETUP/);

  const failing = new FakeAudioContext();
  failing.failAnalyser = true;
  const broken = scope.run(() =>
    useAudioAnalyser({
      source: document.createElement("audio"),
      context: failing as unknown as BaseAudioContext,
    }),
  );
  await flush();
  assert.equal(broken?.state.value, "error");
  assert.ok(broken?.error.value instanceof Error);
  assert.equal(await broken?.resume(), false);
  scope.stop();

  audio.restore();
  audio = installFakeAudio({ audio: false });
  const bare = effectScope();
  const unsupported = bare.run(() => useAudioAnalyser({ source: document.createElement("audio") }));
  await flush();
  assert.equal(unsupported?.state.value, "unsupported");
  bare.stop();
});

test("disposing disconnects nodes and ignores later source changes", async () => {
  const scope = effectScope();
  const source = shallowRef<AudioAnalyserSource | null>(document.createElement("audio"));
  const analyser = scope.run(() => useAudioAnalyser({ source }));
  await flush();
  const node = context().analysers[0];
  assert.ok(node);
  analyser?.dispose();
  assert.equal(analyser?.state.value, "idle");
  source.value = document.createElement("audio");
  await flush();
  assert.equal(context().analysers.length, 1);
  scope.stop();
});

test("the visualizer is decorative by default and exposes frame data to slots", async () => {
  const element = document.createElement("audio");
  const handle = mountInteraction(AudioVisualizer, {
    props: { source: element, fftSize: 64 },
    slots: {
      default: (state: {
        state: string;
        level: number;
        frequency: Uint8Array;
        waveformPath: (width: number, height: number) => string;
        bands: (count: number) => readonly number[];
      }) =>
        h("output", {
          "data-slot-state": state.state,
          "data-bins": String(state.frequency.length),
          "data-level": String(state.level),
          "data-bands": state.bands(4).join(","),
          "data-path": state.waveformPath(4, 2).slice(0, 12),
        }),
    },
  });
  const root = handle.root();
  assert.equal(root.getAttribute("data-vize-ui"), "audio-visualizer-root");
  assert.equal(root.getAttribute("aria-hidden"), "true");
  assert.equal(root.getAttribute("role"), null);
  assert.equal(root.getAttribute("data-state"), "idle");
  await flush();
  const output = root.querySelector("output");
  assert.equal(root.getAttribute("data-state"), "suspended");
  assert.equal(output?.getAttribute("data-bins"), "32");
  assert.equal(output?.getAttribute("data-level"), "0.5");
  assertClose(output?.getAttribute("data-bands")?.split(",").map(Number) ?? [], [
    10 / 255,
    10 / 255,
    10 / 255,
    10 / 255,
  ]);
  assert.equal(output?.getAttribute("data-path"), "M0,0.5 L0.06");
  assert.equal(root.style.getPropertyValue("--vize-ui-audio-visualizer-level"), "0.5");
  const exposed = handle.exposes<AudioVisualizerExpose>();
  assert.equal(await exposed.resume(), true);
  assert.equal(exposed.state, "running");
  audio.frame(16);
  await nextTick();
  assertClose([Number(output?.getAttribute("data-bands")?.split(",")[0])], [20 / 255]);
  handle.unmount();

  const labelled = mountInteraction(AudioVisualizer, { props: { ariaLabel: "Microphone level" } });
  assert.equal(labelled.root().getAttribute("role"), "img");
  assert.equal(labelled.root().getAttribute("aria-hidden"), null);
  labelled.unmount();
});

test("bars render one span per bar with height custom properties or a custom slot", async () => {
  const handle = mountInteraction(AudioVisualizer, {
    props: { source: document.createElement("audio"), fftSize: 64, precision: "float" },
    slots: {
      default: () => [
        h(AudioVisualizerBars, { count: 4, scale: "linear" }),
        h(
          AudioVisualizerBars,
          { count: 2 },
          {
            default: ({ bars }: { bars: readonly number[] }) =>
              h("i", { "data-custom": bars.length }),
          },
        ),
      ],
    },
  });
  await flush();
  const containers = handle.root().querySelectorAll('[data-vize-ui="audio-visualizer-bars"]');
  const bars = containers[0]?.querySelectorAll('[data-vize-ui="audio-visualizer-bar"]');
  assert.equal(containers[0]?.getAttribute("data-count"), "4");
  assert.equal(bars?.length, 4);
  assert.equal(
    (bars?.[0] as HTMLElement | undefined)?.style.getPropertyValue(
      "--vize-ui-audio-visualizer-bar",
    ),
    String(Math.round((35 / 70) * 1000) / 1000),
  );
  assert.equal(containers[1]?.querySelector("[data-custom]")?.getAttribute("data-custom"), "2");
  handle.unmount();

  assert.throws(
    () => mountInteraction(AudioVisualizerBars),
    /VIZE_UI_CONTEXT_MISSING: AudioVisualizer requires a matching provider/,
  );
});

test("pure helpers compute levels, bars, edges, and waveform paths deterministically", () => {
  const silence = new Uint8Array(8).fill(128);
  assert.equal(computeLevel(silence), 0);
  assert.equal(computePeak(silence), 0);
  assert.equal(computeLevel(new Uint8Array(0)), 0);
  assert.equal(computeLevel(new Float32Array([1, -1, 1, -1])), 1);
  assert.equal(computePeak(new Float32Array([0.25, -0.75])), 0.75);
  assert.equal(computePeak(new Float32Array([2])), 1, "levels clamp to 1");

  assert.deepEqual(barEdges(8, 4, "linear"), [0, 2, 4, 6, 8]);
  assert.deepEqual(barEdges(1024, 4, "log"), [1, 6, 32, 181, 1024]);
  assert.deepEqual(barEdges(2, 4, "linear"), [0, 1, 2, 2, 2]);
  assert.deepEqual(barEdges(0, 4, "log"), []);

  const data = new Uint8Array([0, 51, 102, 153, 204, 255, 255, 255]);
  assertClose(toBars(data, 2, { scale: "linear" }), [0.3, 0.95]);
  assert.deepEqual(toBars(new Uint8Array([255, 0]), 4, { scale: "linear" }), [1, 0, 0, 0]);
  assert.deepEqual(
    toBars(new Float32Array([-100, -30, -65, -200]), 2, { scale: "linear", decibels: [-100, -30] }),
    [0.5, 0.25],
  );
  assert.ok(Object.isFrozen(toBars(data, 2)));

  assert.equal(toWaveformPath(new Uint8Array([128, 255, 0]), 10, 4), "M0,2 L5,0.02 L10,4");
  assert.equal(toWaveformPath(new Float32Array([0, 1]), 3, 3), "M0,1.5 L3,0");
  assert.equal(toWaveformPath(new Uint8Array(0), 10, 4), "");
  assert.equal(toWaveformPath(new Uint8Array([1]), 0, 4), "");
});
