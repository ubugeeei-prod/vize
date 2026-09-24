import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope, nextTick, ref } from "vue";

import { useAnimate } from "./animate.ts";
import type { AnimateFrameHost, AnimationKeyframes } from "./animate.ts";
import { asElement, FakeDocument, FakeElement } from "./testing/fake-dom.ts";

class FakeAnimation extends EventTarget {
  playState: AnimationPlayState = "running";
  pending = false;
  currentTime: number | null = 0;
  playbackRate = 1;
  persisted = false;
  committed = false;
  readonly effect: { keyframes: unknown; setKeyframes: (frames: unknown) => void };
  readonly timing: unknown;

  constructor(keyframes: unknown, timing: unknown) {
    super();
    this.timing = timing;
    this.effect = {
      keyframes,
      setKeyframes: (frames) => {
        this.effect.keyframes = frames;
      },
    };
  }

  play(): void {
    this.playState = "running";
  }
  pause(): void {
    this.playState = "paused";
  }
  reverse(): void {
    this.playbackRate *= -1;
  }
  finish(): void {
    this.playState = "finished";
    this.dispatchEvent(new Event("finish"));
  }
  cancel(): void {
    this.playState = "idle";
    this.currentTime = null;
  }
  persist(): void {
    this.persisted = true;
  }
  commitStyles(): void {
    this.committed = true;
  }
}

class AnimatedElement extends FakeElement {
  readonly animations: FakeAnimation[] = [];
  animate(keyframes: unknown, timing: unknown): FakeAnimation {
    const animation = new FakeAnimation(keyframes, timing);
    this.animations.push(animation);
    return animation;
  }
}

const noFrames: AnimateFrameHost = {
  requestAnimationFrame: () => 0,
  cancelAnimationFrame: () => undefined,
};

void test("creates the animation with typed keyframes and controls playback", async () => {
  const element = new AnimatedElement(new FakeDocument());
  const keyframes = ref<AnimationKeyframes>([
    { opacity: 0 },
    { opacity: 1, transform: "scale(2)" },
  ]);
  const scope = effectScope();
  const animate = scope.run(() =>
    useAnimate(asElement(element), keyframes, {
      duration: 300,
      commitStyles: true,
      persist: true,
      frameHost: noFrames,
    }),
  );
  assert.ok(animate);
  const animation = element.animations[0];
  assert.ok(animation);
  assert.deepEqual(animation.timing, { duration: 300 });
  assert.equal(animation.persisted, true);
  assert.equal(animate.playState.value, "running");

  animate.pause();
  assert.equal(animate.playState.value, "paused");
  animate.playbackRate.value = 2;
  assert.equal(animation.playbackRate, 2);
  animate.reverse();
  assert.equal(animate.playbackRate.value, -2);
  animate.currentTime.value = 150;
  assert.equal(animation.currentTime, 150);

  keyframes.value = { opacity: [0, 0.5, 1] };
  await nextTick();
  assert.deepEqual(animation.effect.keyframes, { opacity: [0, 0.5, 1] });

  animate.finish();
  assert.equal(animation.committed, true);
  assert.equal(animate.playState.value, "finished");

  scope.stop();
  await nextTick();
  assert.equal(animation.playState, "idle");
});

void test("immediate false starts paused; unsupported elements stay idle", () => {
  const element = new AnimatedElement(new FakeDocument());
  const scope = effectScope();
  const paused = scope.run(() =>
    useAnimate(asElement(element), [{ opacity: 0 }], {
      duration: 10,
      immediate: false,
      frameHost: noFrames,
    }),
  );
  assert.equal(paused?.playState.value, "paused");

  const plain = new FakeDocument().createElement();
  const unsupported = scope.run(() => useAnimate(asElement(plain), [{ opacity: 0 }], 100));
  assert.deepEqual([unsupported?.isSupported.value, unsupported?.playState.value], [false, "idle"]);
  unsupported?.play();
  assert.equal(unsupported?.animation.value, null);
  scope.stop();
});
