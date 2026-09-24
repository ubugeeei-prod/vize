import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope, nextTick, ref } from "vue";

import { cubicBezier, TransitionPresets, useTransition } from "./transition.ts";
import type { TransitionFrameHost } from "./transition.ts";

class Frames implements TransitionFrameHost {
  private queue = new Map<number, FrameRequestCallback>();
  private next = 1;
  requestAnimationFrame(callback: FrameRequestCallback): number {
    const id = this.next++;
    this.queue.set(id, callback);
    return id;
  }
  cancelAnimationFrame(handle: number): void {
    this.queue.delete(handle);
  }
  get pending(): number {
    return this.queue.size;
  }
  frame(time: number): void {
    const callbacks = [...this.queue.values()];
    this.queue.clear();
    for (const callback of callbacks) callback(time);
  }
}

void test("cubic Bézier presets hit their endpoints and are monotonic for ease-in-out", () => {
  const ease = cubicBezier(TransitionPresets.easeInOutCubic);
  assert.equal(ease(0), 0);
  assert.equal(ease(1), 1);
  assert.ok(Math.abs(ease(0.5) - 0.5) < 1e-3);
  assert.ok(ease(0.25) < 0.25 && ease(0.75) > 0.75);
  assert.equal(cubicBezier([0, 0, 1, 1])(0.3), 0.3);
});

void test("tweens numbers with delay, callbacks, and interruption", async () => {
  const frames = new Frames();
  const source = ref(0);
  const phases: string[] = [];
  const scope = effectScope();
  const output = scope.run(() =>
    useTransition(source, {
      host: frames,
      duration: 100,
      delay: 50,
      onStarted: () => phases.push("start"),
      onFinished: () => phases.push("finish"),
    }),
  );
  assert.equal(output?.value, 0);

  source.value = 100;
  await nextTick();
  frames.frame(0);
  frames.frame(40);
  assert.equal(output?.value, 0);
  frames.frame(100);
  assert.equal(output?.value, 50);
  source.value = 0;
  await nextTick();
  frames.frame(200);
  frames.frame(300);
  assert.equal(output?.value, 25);
  frames.frame(400);
  assert.equal(output?.value, 0);
  assert.deepEqual(phases, ["start", "start", "finish"]);
  assert.equal(frames.pending, 0);
  scope.stop();
});

void test("tweens tuples element-wise and mirrors the source without frames", async () => {
  const frames = new Frames();
  const source = ref<readonly [number, number]>([0, 10]);
  const scope = effectScope();
  const tuple = scope.run(() =>
    useTransition(source, { host: frames, duration: 100, easing: "linear" }),
  );
  source.value = [100, 20];
  await nextTick();
  frames.frame(0);
  frames.frame(50);
  assert.deepEqual(tuple?.value, [50, 15]);

  const server = ref(1);
  const mirrored = scope.run(() => useTransition(server, { host: () => undefined }));
  server.value = 9;
  await nextTick();
  assert.equal(mirrored?.value, 9);

  const disabled = ref(1);
  const skipped = scope.run(() => useTransition(disabled, { host: frames, disabled: true }));
  disabled.value = 5;
  await nextTick();
  assert.equal(skipped?.value, 5);
  scope.stop();
  assert.equal(frames.pending, 0);
});
