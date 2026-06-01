/**
 * useAnimation - Ink-compatible shared animation state.
 */

import { computed, onMounted, onUnmounted, reactive, watch } from "@vue/runtime-core";

export interface UseAnimationOptions {
  /** Time between ticks in milliseconds */
  interval?: number;
  /** Whether the animation is running */
  isActive?: boolean;
}

export interface AnimationResult {
  frame: number;
  time: number;
  delta: number;
  reset: () => void;
}

const defaultAnimationInterval = 100;
const maximumTimerInterval = 2_147_483_647;

type AnimationCallback = (currentTime: number) => void;

interface AnimationSubscriber {
  callback: AnimationCallback;
  interval: number;
  startTime: number;
  nextDueTime: number;
}

const animationSubscribers = new Map<AnimationCallback, AnimationSubscriber>();
let animationTimer: ReturnType<typeof setTimeout> | null = null;

function currentTime(): number {
  return performance.now();
}

function normalizeAnimationInterval(interval: number): number {
  if (!Number.isFinite(interval)) {
    return defaultAnimationInterval;
  }

  return Math.min(maximumTimerInterval, Math.max(1, interval));
}

function clearAnimationTimer() {
  if (!animationTimer) return;
  clearTimeout(animationTimer);
  animationTimer = null;
}

function scheduleAnimationTick() {
  clearAnimationTimer();
  if (animationSubscribers.size === 0) return;

  let nextDueTime = Number.POSITIVE_INFINITY;
  for (const subscriber of animationSubscribers.values()) {
    nextDueTime = Math.min(nextDueTime, subscriber.nextDueTime);
  }

  animationTimer = setTimeout(
    () => {
      animationTimer = null;
      const now = currentTime();

      for (const subscriber of animationSubscribers.values()) {
        if (now < subscriber.nextDueTime) continue;

        subscriber.callback(now);

        const elapsedTime = now - subscriber.startTime;
        const elapsedFrames = Math.floor(elapsedTime / subscriber.interval) + 1;
        subscriber.nextDueTime = subscriber.startTime + elapsedFrames * subscriber.interval;
      }

      scheduleAnimationTick();
    },
    Math.max(0, nextDueTime - currentTime()),
  );
}

function subscribeToAnimation(callback: AnimationCallback, interval: number) {
  const startTime = currentTime();
  animationSubscribers.set(callback, {
    callback,
    interval,
    startTime,
    nextDueTime: startTime + interval,
  });
  scheduleAnimationTick();

  return {
    startTime,
    unsubscribe() {
      animationSubscribers.delete(callback);
      scheduleAnimationTick();
    },
  };
}

export function useAnimation(options: UseAnimationOptions = {}): AnimationResult {
  const safeInterval = computed(() =>
    normalizeAnimationInterval(options.interval ?? defaultAnimationInterval),
  );
  const active = computed(() => options.isActive ?? true);
  let unsubscribe: (() => void) | null = null;
  let startedAt = currentTime();
  let previousTick = startedAt;

  const state = reactive<AnimationResult>({
    frame: 0,
    time: 0,
    delta: 0,
    reset: () => {
      if (active.value) {
        start();
      } else {
        resetState(currentTime());
      }
    },
  });

  const resetState = (now: number) => {
    startedAt = now;
    previousTick = now;
    state.frame = 0;
    state.time = 0;
    state.delta = 0;
  };

  const tick = (now: number) => {
    const elapsed = now - startedAt;
    state.frame = Math.floor(elapsed / safeInterval.value);
    state.time = elapsed;
    state.delta = now - previousTick;
    previousTick = now;
  };

  const stop = () => {
    unsubscribe?.();
    unsubscribe = null;
  };

  const start = () => {
    stop();
    resetState(currentTime());
    const subscription = subscribeToAnimation(tick, safeInterval.value);
    startedAt = subscription.startTime;
    previousTick = subscription.startTime;
    unsubscribe = () => subscription.unsubscribe();
  };

  onMounted(() => {
    if (active.value) {
      start();
    }
  });

  watch([active, safeInterval], ([isActive]) => {
    if (isActive) {
      start();
    } else {
      stop();
    }
  });

  onUnmounted(stop);

  return state;
}
