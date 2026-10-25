import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope, nextTick, ref } from "vue";
import { Window } from "happy-dom";
import { isElementNode } from "./element-target.ts";

import { useAnimate } from "./animate.ts";
import type { AnimateFrameHost, AnimationKeyframes } from "./animate.ts";
import { asElement, FakeDocument, FakeElement } from "./testing/fake-dom.ts";

function deferredAnimation() {
  let resolve = (_value: FakeAnimation): void => undefined;
  let reject = (_error: unknown): void => undefined;
  const promise = new Promise<FakeAnimation>((onResolve, onReject) => {
    resolve = onResolve;
    reject = onReject;
  });
  return { promise, resolve, reject };
}

class FakeAnimation extends EventTarget {
  playState: AnimationPlayState = "running";
  pending = false;
  currentTime: number | null = 0;
  playbackRate = 1;
  persisted = false;
  committed = false;
  #completion = deferredAnimation();
  ready: Promise<FakeAnimation> = Promise.resolve(this);
  #readiness = deferredAnimation();

  queueReady(): Promise<FakeAnimation> {
    this.#readiness = deferredAnimation();
    this.pending = true;
    this.ready = this.#readiness.promise;
    return this.ready;
  }

  get finished(): Promise<FakeAnimation> {
    return this.#completion.promise;
  }
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
    if (this.playState === "finished") this.#completion = deferredAnimation();
    this.playState = "running";
  }
  pause(): void {
    this.playState = "paused";
  }
  reverse(): void {
    this.playbackRate *= -1;
  }
  finish(): void {
    this.#completion.resolve(this);
    this.playState = "finished";
    this.dispatchEvent(new Event("finish"));
  }
  cancel(): void {
    if (this.pending) {
      this.#readiness.reject(new DOMException("Playback was canceled.", "AbortError"));
      this.ready = Promise.resolve(this);
      this.pending = false;
    }
    if (this.playState !== "idle") {
      this.#completion.reject(new DOMException("The animation was canceled.", "AbortError"));
      this.#completion = deferredAnimation();
    }
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

void test("owned cancellation observes each completion promise across replay and disposal", async () => {
  const first = new AnimatedElement(new FakeDocument());
  const second = new AnimatedElement(new FakeDocument());
  const target = ref(asElement(first));
  const scope = effectScope();
  const controls = scope.run(() =>
    useAnimate(target, [{ opacity: 0 }], { duration: 10, frameHost: noFrames }),
  );
  assert.ok(controls);
  const animation = first.animations[0];
  assert.ok(animation);
  for (let round = 0; round < 3; round += 1) {
    const finished = animation.finished;
    controls.cancel();
    await new Promise<void>((resolve) => setImmediate(resolve));
    assert.deepEqual(
      [controls.playState.value, animation.playState, animation.currentTime],
      ["idle", "idle", null],
    );
    await assert.rejects(finished, { name: "AbortError" });
    controls.play();
    controls.finish();
    assert.equal(await animation.finished, animation);
    controls.play();
  }
  const pendingReady = animation.queueReady();
  const pendingFinished = animation.finished;
  controls.cancel();
  await new Promise<void>((resolve) => setImmediate(resolve));
  await assert.rejects(pendingReady, { name: "AbortError" });
  await assert.rejects(pendingFinished, { name: "AbortError" });
  assert.equal(controls.pending.value, false);
  controls.play();
  const replaced = animation.finished;
  target.value = asElement(second);
  await nextTick();
  await new Promise<void>((resolve) => setImmediate(resolve));
  await assert.rejects(replaced, { name: "AbortError" });
  assert.equal(first.animations.length, 1);
  assert.equal(second.animations.length, 1);
  const active = second.animations[0];
  assert.ok(active);
  assert.equal(controls.animation.value, active);
  const disposed = active.finished;
  scope.stop();
  await new Promise<void>((resolve) => setImmediate(resolve));
  await assert.rejects(disposed, { name: "AbortError" });
  assert.deepEqual(
    [controls.animation.value, controls.playState.value, active.playState],
    [null, "idle", "idle"],
  );
});

void test("cancellation keeps synchronous platform errors visible", () => {
  const element = new AnimatedElement(new FakeDocument());
  const scope = effectScope();
  const controls = scope.run(() =>
    useAnimate(asElement(element), [{ opacity: 0 }], {
      duration: 10,
      immediate: false,
      frameHost: noFrames,
    }),
  );
  assert.ok(controls);
  const animation = element.animations[0];
  assert.ok(animation);
  const originalCancel = animation.cancel.bind(animation);
  const failure = new Error("platform cancellation failed");
  animation.cancel = () => {
    throw failure;
  };
  assert.throws(controls.cancel, (error) => error === failure);
  animation.cancel = originalCancel;
  scope.stop();
});

void test("platform animations unmount without an unhandled completion rejection", async () => {
  const window = new Window();
  const element = window.document.createElement("div");
  window.document.body.append(element);
  assert.ok(isElementNode(element));
  const scope = effectScope();
  try {
    const controls = scope.run(() =>
      useAnimate(element, [{ opacity: 0 }, { opacity: 1 }], {
        duration: 1000,
        immediate: false,
        frameHost: noFrames,
      }),
    );
    assert.ok(controls?.isSupported.value);
    const animation = controls.animation.value;
    assert.ok(animation);
    const completion = animation.finished;
    scope.stop();
    await new Promise<void>((resolve) => setImmediate(resolve));
    await assert.rejects(completion, { name: "AbortError" });
    assert.deepEqual(
      [controls.animation.value, controls.playState.value, animation.playState],
      [null, "idle", "idle"],
    );
  } finally {
    scope.stop();
    await window.happyDOM.close();
  }
});
