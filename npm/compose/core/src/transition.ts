import { shallowRef, toValue, watch } from "vue";
import type { MaybeRefOrGetter, Ref } from "vue";

import { tryOnScopeDispose } from "./scope.ts";

/** Cubic Bézier control points `[x1, y1, x2, y2]`. */
export type CubicBezierPoints = readonly [number, number, number, number];

/** Easing: a preset name, cubic Bézier control points, or a function on `[0, 1]`. */
export type TransitionEasing =
  | TransitionPresetName
  | CubicBezierPoints
  | ((progress: number) => number);

/** Named cubic Bézier easing presets (the classic easings.net set). */
export const TransitionPresets = {
  linear: [0, 0, 1, 1],
  easeInSine: [0.12, 0, 0.39, 0],
  easeOutSine: [0.61, 1, 0.88, 1],
  easeInOutSine: [0.37, 0, 0.63, 1],
  easeInQuad: [0.11, 0, 0.5, 0],
  easeOutQuad: [0.5, 1, 0.89, 1],
  easeInOutQuad: [0.45, 0, 0.55, 1],
  easeInCubic: [0.32, 0, 0.67, 0],
  easeOutCubic: [0.33, 1, 0.68, 1],
  easeInOutCubic: [0.65, 0, 0.35, 1],
  easeInQuart: [0.5, 0, 0.75, 0],
  easeOutQuart: [0.25, 1, 0.5, 1],
  easeInOutQuart: [0.76, 0, 0.24, 1],
  easeInQuint: [0.64, 0, 0.78, 0],
  easeOutQuint: [0.22, 1, 0.36, 1],
  easeInOutQuint: [0.83, 0, 0.17, 1],
  easeInExpo: [0.7, 0, 0.84, 0],
  easeOutExpo: [0.16, 1, 0.3, 1],
  easeInOutExpo: [0.87, 0, 0.13, 1],
  easeInCirc: [0.55, 0, 1, 0.45],
  easeOutCirc: [0, 0.55, 0.45, 1],
  easeInOutCirc: [0.85, 0, 0.15, 1],
  easeInBack: [0.36, 0, 0.66, -0.56],
  easeOutBack: [0.34, 1.56, 0.64, 1],
  easeInOutBack: [0.68, -0.6, 0.32, 1.6],
} as const satisfies Readonly<Record<string, CubicBezierPoints>>;

/** Names of {@link TransitionPresets}. */
export type TransitionPresetName = keyof typeof TransitionPresets;

/** Numeric value tweened by {@link useTransition}: a number or a fixed-length tuple. */
export type TransitionValue = number | readonly number[];

/** Mutable tuple output matching a tuple source. */
export type TransitionTupleOutput<Source extends readonly number[]> = {
  -readonly [Index in keyof Source]: number;
};

/** Frame scheduler used by {@link useTransition}. */
export interface TransitionFrameHost {
  /** Schedule a callback before the next repaint. */
  readonly requestAnimationFrame: (callback: FrameRequestCallback) => number;
  /** Cancel a scheduled frame. */
  readonly cancelAnimationFrame: (handle: number) => void;
}

/** Options for {@link useTransition}. */
export interface UseTransitionOptions {
  /**
   * Duration in milliseconds.
   *
   * @default 1000
   */
  readonly duration?: MaybeRefOrGetter<number>;

  /**
   * Delay before each transition starts, in milliseconds.
   *
   * @default 0
   */
  readonly delay?: MaybeRefOrGetter<number>;

  /**
   * Easing function, preset name, or cubic Bézier points.
   *
   * @default "linear"
   */
  readonly easing?: MaybeRefOrGetter<TransitionEasing>;

  /**
   * Jump to the source value without animating.
   *
   * @default false
   */
  readonly disabled?: MaybeRefOrGetter<boolean>;

  /**
   * Called when a transition starts.
   *
   * @default undefined
   */
  readonly onStarted?: () => void;

  /**
   * Called when a transition reaches its target.
   *
   * @default undefined
   */
  readonly onFinished?: () => void;

  /**
   * Frame scheduler. Without one (server rendering) the output mirrors the
   * source immediately.
   *
   * @default globalThis.window when available
   */
  readonly host?: MaybeRefOrGetter<TransitionFrameHost | null | undefined>;
}

/**
 * Tween a number toward its reactive source.
 *
 * Each source change starts a new transition from the current output, so
 * interrupted transitions stay continuous. Easing presets and Bézier points
 * are solved with Newton–Raphson plus bisection. Without a frame scheduler
 * (server rendering, tests) the output equals the source, which keeps SSR
 * deterministic. Pending frames are cancelled with the owning scope.
 *
 * @param source Reactive number.
 * @param options Duration, delay, easing, callbacks, and frame capability.
 * @returns Readonly tweened number.
 */
export function useTransition(
  source: MaybeRefOrGetter<number>,
  options?: UseTransitionOptions,
): Readonly<Ref<number>>;
/**
 * Tween each element of a fixed-length numeric tuple.
 *
 * @param source Reactive tuple; its length must stay constant.
 * @param options Duration, delay, easing, callbacks, and frame capability.
 * @returns Readonly tweened tuple with the same length.
 */
export function useTransition<const Source extends readonly number[]>(
  source: MaybeRefOrGetter<Source>,
  options?: UseTransitionOptions,
): Readonly<Ref<TransitionTupleOutput<Source>>>;
export function useTransition(
  source: MaybeRefOrGetter<TransitionValue>,
  options: UseTransitionOptions = {},
): Readonly<Ref<number | number[]>> {
  const initial = toValue(source);
  const output = shallowRef<number | number[]>(copy(initial));
  let cancel = (): void => undefined;

  const stop = watch(
    () => copy(toValue(source)),
    (target) => {
      cancel();
      const host = options.host === undefined ? browserFrameHost() : toValue(options.host);
      const duration = Math.max(0, toValue(options.duration) ?? 1000);
      if (!host || toValue(options.disabled) || duration === 0) {
        output.value = target;
        return;
      }
      const from = output.value;
      const ease = toEasingFunction(toValue(options.easing) ?? "linear");
      const delay = Math.max(0, toValue(options.delay) ?? 0);
      let startedAt: number | undefined;
      let started = false;
      let handle = 0;
      const frame = (time: number): void => {
        startedAt ??= time + delay;
        const elapsed = time - startedAt;
        if (elapsed < 0) {
          handle = host.requestAnimationFrame(frame);
          return;
        }
        if (!started) {
          started = true;
          options.onStarted?.();
        }
        const progress = Math.min(1, elapsed / duration);
        output.value = interpolate(from, target, ease(progress));
        if (progress < 1) {
          handle = host.requestAnimationFrame(frame);
          return;
        }
        cancel = () => undefined;
        options.onFinished?.();
      };
      handle = host.requestAnimationFrame(frame);
      cancel = () => {
        host.cancelAnimationFrame(handle);
        cancel = () => undefined;
      };
    },
  );
  tryOnScopeDispose(() => {
    stop.stop();
    cancel();
  });

  return output;
}

/**
 * Build an easing function from cubic Bézier control points.
 *
 * @param points `[x1, y1, x2, y2]` with `x1`/`x2` in `[0, 1]`.
 * @returns Easing function mapping progress to eased progress.
 */
export function cubicBezier(points: CubicBezierPoints): (progress: number) => number {
  const [x1, y1, x2, y2] = points;
  const a = (p1: number, p2: number): number => 1 - 3 * p2 + 3 * p1;
  const b = (p1: number, p2: number): number => 3 * p2 - 6 * p1;
  const c = (p1: number): number => 3 * p1;
  const at = (t: number, p1: number, p2: number): number =>
    ((a(p1, p2) * t + b(p1, p2)) * t + c(p1)) * t;
  const slope = (t: number, p1: number, p2: number): number =>
    3 * a(p1, p2) * t * t + 2 * b(p1, p2) * t + c(p1);
  const solve = (x: number): number => {
    let t = x;
    for (let iteration = 0; iteration < 8; iteration += 1) {
      const error = at(t, x1, x2) - x;
      if (Math.abs(error) < 1e-7) return t;
      const derivative = slope(t, x1, x2);
      if (Math.abs(derivative) < 1e-6) break;
      t -= error / derivative;
    }
    let low = 0;
    let high = 1;
    t = x;
    while (high - low > 1e-7) {
      if (at(t, x1, x2) < x) low = t;
      else high = t;
      t = (low + high) / 2;
    }
    return t;
  };
  return (progress) => {
    if (progress <= 0) return 0;
    if (progress >= 1) return 1;
    if (x1 === y1 && x2 === y2) return progress;
    return at(solve(progress), y1, y2);
  };
}

function toEasingFunction(easing: TransitionEasing): (progress: number) => number {
  if (typeof easing === "function") return easing;
  return cubicBezier(typeof easing === "string" ? TransitionPresets[easing] : easing);
}

function copy(value: TransitionValue): number | number[] {
  return typeof value === "number" ? value : [...value];
}

function interpolate(
  from: number | number[],
  to: number | number[],
  progress: number,
): number | number[] {
  if (typeof to === "number") {
    const start = typeof from === "number" ? from : to;
    return start + (to - start) * progress;
  }
  return to.map((target, index) => {
    const start = typeof from === "number" ? target : (from[index] ?? target);
    return start + (target - start) * progress;
  });
}

function browserFrameHost(): TransitionFrameHost | undefined {
  return typeof window !== "undefined" ? window : undefined;
}
