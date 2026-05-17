/**
 * useAnimation - shared timer-friendly animation state.
 */

import { onMounted, onUnmounted, reactive, watch } from "@vue/runtime-core";

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

export function useAnimation(options: UseAnimationOptions = {}): AnimationResult {
  const interval = options.interval ?? 100;
  let timer: ReturnType<typeof setInterval> | null = null;
  let startedAt = Date.now();
  let previousTick = startedAt;

  const state = reactive<AnimationResult>({
    frame: 0,
    time: 0,
    delta: 0,
    reset: () => {
      startedAt = Date.now();
      previousTick = startedAt;
      state.frame = 0;
      state.time = 0;
      state.delta = 0;
    },
  });

  const tick = () => {
    const now = Date.now();
    state.frame += 1;
    state.time = now - startedAt;
    state.delta = now - previousTick;
    previousTick = now;
  };

  const stop = () => {
    if (timer) {
      clearInterval(timer);
      timer = null;
    }
  };

  const start = () => {
    stop();
    state.reset();
    timer = setInterval(tick, interval);
  };

  onMounted(() => {
    if (options.isActive ?? true) {
      start();
    }
  });

  watch(
    () => options.isActive,
    (isActive) => {
      if (isActive ?? true) {
        start();
      } else {
        stop();
      }
    },
  );

  onUnmounted(stop);

  return state;
}
