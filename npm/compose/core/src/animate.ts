import { computed, readonly, ref, shallowRef, toValue, watch } from "vue";
import type { MaybeRefOrGetter, Ref, ShallowRef, WritableComputedRef } from "vue";

import { resolveElement } from "./element-target.ts";
import type { MaybeElementTarget } from "./element-target.ts";
import { tryOnScopeDispose } from "./scope.ts";

/** CSS property names (camelCase) that accept string values on `CSSStyleDeclaration`. */
export type AnimatableCssProperty = {
  [Key in keyof CSSStyleDeclaration]: Key extends string
    ? CSSStyleDeclaration[Key] extends string
      ? Key
      : never
    : never;
}[keyof CSSStyleDeclaration];

type KeyframeProperty = Exclude<AnimatableCssProperty, "offset" | "cssText">;

/** Keyframe whose property names are checked against `CSSStyleDeclaration`. */
export type TypedKeyframe = {
  readonly [Key in KeyframeProperty]?: string | number | null;
} & {
  /** Position of the keyframe in `[0, 1]`. */
  readonly offset?: number | null;
  /** Easing applied from this keyframe to the next. */
  readonly easing?: string;
  /** Composite operation for this keyframe. */
  readonly composite?: CompositeOperationOrAuto;
  /** CSS `offset` shorthand (motion path); WAAPI names it `cssOffset` to avoid the keyframe offset. */
  readonly cssOffset?: string;
};

/** Value list of one property in property-indexed keyframes. */
export type IndexedKeyframeValue =
  | string
  | number
  | null
  | readonly string[]
  | readonly (number | null)[];

/** Property-indexed keyframes with checked property names. */
export type TypedPropertyIndexedKeyframes = {
  readonly [Key in KeyframeProperty]?: IndexedKeyframeValue;
} & {
  /** Keyframe positions. */
  readonly offset?: number | null | readonly (number | null)[];
  /** Easings. */
  readonly easing?: string | readonly string[];
  /** Composite operations. */
  readonly composite?: CompositeOperationOrAuto | readonly CompositeOperationOrAuto[];
};

/** Keyframes accepted by {@link useAnimate}. */
export type AnimationKeyframes = readonly TypedKeyframe[] | TypedPropertyIndexedKeyframes;

/** Frame scheduler used to keep `currentTime` in sync while running. */
export interface AnimateFrameHost {
  /** Schedule a callback before the next repaint. */
  readonly requestAnimationFrame: (callback: FrameRequestCallback) => number;
  /** Cancel a scheduled frame. */
  readonly cancelAnimationFrame: (handle: number) => void;
}

/** Options for {@link useAnimate}. */
export interface UseAnimateOptions extends KeyframeAnimationOptions {
  /**
   * Start playing as soon as the element resolves.
   *
   * @default true
   */
  readonly immediate?: boolean;

  /**
   * Commit the final computed styles to the element's `style` on finish.
   *
   * @default false
   */
  readonly commitStyles?: boolean;

  /**
   * Keep the animation from being auto-removed by the browser when it is
   * replaced by another filling animation.
   *
   * @default false
   */
  readonly persist?: boolean;

  /**
   * Called once the animation object is created.
   *
   * @default undefined
   */
  readonly onReady?: (animation: Animation) => void;

  /**
   * Called when creating the animation throws (e.g. invalid keyframes).
   *
   * @default undefined
   */
  readonly onError?: (error: unknown) => void;

  /**
   * Frame scheduler used to sync `currentTime` while running.
   *
   * @default globalThis.window when available
   */
  readonly frameHost?: MaybeRefOrGetter<AnimateFrameHost | null | undefined>;
}

/** Reactive animation state returned by {@link useAnimate}. */
export interface AnimateControls {
  /** Whether the resolved element supports `animate()`. `false` during server rendering. */
  readonly isSupported: Readonly<Ref<boolean>>;
  /** Underlying `Animation`, once created. */
  readonly animation: Readonly<ShallowRef<Animation | null>>;
  /** Playback state. */
  readonly playState: Readonly<Ref<AnimationPlayState>>;
  /** Whether a play/pause is pending. */
  readonly pending: Readonly<Ref<boolean>>;
  /** Current time in milliseconds (writable to seek). */
  readonly currentTime: WritableComputedRef<number | null>;
  /** Playback rate (writable). */
  readonly playbackRate: WritableComputedRef<number>;
  /** Play (creating the animation if needed). */
  readonly play: () => void;
  /** Pause. */
  readonly pause: () => void;
  /** Reverse the playback direction. */
  readonly reverse: () => void;
  /** Jump to the end. */
  readonly finish: () => void;
  /** Cancel and clear effects. */
  readonly cancel: () => void;
}

/**
 * Drive the Web Animations API reactively with typed keyframes.
 *
 * Keyframe property names are checked against `CSSStyleDeclaration`, so typos
 * are compile errors. Changing keyframes updates the running effect in place.
 * Playback state is synced after every control call, on `finish`/`cancel`/
 * `remove`, and every animation frame while running. The animation is
 * cancelled when the owning reactive scope stops. Nothing runs during server
 * rendering (`playState` is `"idle"`).
 *
 * @param target Reactive element target.
 * @param keyframes Reactive typed keyframes (`null` to pause creation).
 * @param options Duration in milliseconds, or timing plus lifecycle options.
 * @returns Reactive playback state and controls.
 */
export function useAnimate(
  target: MaybeElementTarget,
  keyframes: MaybeRefOrGetter<AnimationKeyframes | null>,
  options: number | UseAnimateOptions,
): AnimateControls {
  const settings: UseAnimateOptions = typeof options === "number" ? { duration: options } : options;
  const {
    immediate = true,
    commitStyles = false,
    persist = false,
    onReady,
    onError,
    frameHost,
    ...timing
  } = settings;
  const animation = shallowRef<Animation | null>(null);
  const isSupported = ref(false);
  const playState = ref<AnimationPlayState>("idle");
  const pending = ref(false);
  const time = ref<number | null>(null);
  const rate = ref(timing.playbackRate ?? 1);
  let stopFrames = (): void => undefined;

  const readTime = (value: CSSNumberish | null): number | null =>
    typeof value === "number" ? value : null;
  const sync = (): void => {
    const current = animation.value;
    if (!current) return;
    playState.value = current.playState;
    pending.value = current.pending;
    time.value = readTime(current.currentTime);
    rate.value = current.playbackRate;
    if (current.playState === "running") startFrames();
    else stopFrames();
  };
  const startFrames = (): void => {
    const frames = frameHost === undefined ? browserFrameHost() : toValue(frameHost);
    if (!frames) return;
    stopFrames();
    let handle = frames.requestAnimationFrame(function tick() {
      sync();
      if (animation.value?.playState === "running") handle = frames.requestAnimationFrame(tick);
    });
    stopFrames = () => {
      frames.cancelAnimationFrame(handle);
      stopFrames = () => undefined;
    };
  };

  const create = (): Animation | null => {
    const element = resolveElement(target);
    const frames = toValue(keyframes);
    isSupported.value = element !== null && typeof element.animate === "function";
    if (!element || !frames || !isSupported.value) return null;
    try {
      const created = element.animate(toPlatformKeyframes(frames), timing);
      if (persist) created.persist();
      if (commitStyles) {
        created.addEventListener("finish", () => {
          try {
            created.commitStyles();
          } catch {
            // Committing fails for disconnected elements; the fill still applies.
          }
        });
      }
      for (const type of ["finish", "cancel", "remove"] as const) {
        created.addEventListener(type, sync);
      }
      animation.value = created;
      onReady?.(created);
      return created;
    } catch (error) {
      onError?.(error);
      return null;
    }
  };

  const stopTarget = watch(
    () => resolveElement(target),
    (element, _previous, onCleanup) => {
      if (!element) return;
      const created = create();
      if (created && !immediate) created.pause();
      sync();
      onCleanup(() => {
        stopFrames();
        created?.cancel();
        if (animation.value === created) animation.value = null;
        playState.value = "idle";
      });
    },
    { immediate: true, flush: "post" },
  );
  const stopKeyframes = watch(
    () => toValue(keyframes),
    (frames) => {
      const effect = animation.value?.effect;
      if (frames && effect && isKeyframeEffect(effect))
        effect.setKeyframes(toPlatformKeyframes(frames));
    },
    { deep: true },
  );
  tryOnScopeDispose(() => {
    stopTarget.stop();
    stopKeyframes.stop();
    stopFrames();
  });

  const control = (action: (current: Animation) => void) => (): void => {
    const current = animation.value ?? create();
    if (!current) return;
    action(current);
    sync();
  };

  return {
    isSupported: readonly(isSupported),
    animation,
    playState: readonly(playState),
    pending: readonly(pending),
    currentTime: computed({
      get: () => time.value,
      set: (value: number | null) => {
        if (!animation.value) return;
        animation.value.currentTime = value;
        sync();
      },
    }),
    playbackRate: computed({
      get: () => rate.value,
      set: (value: number) => {
        rate.value = value;
        if (!animation.value) return;
        animation.value.playbackRate = value;
        sync();
      },
    }),
    play: control((current) => current.play()),
    pause: control((current) => current.pause()),
    reverse: control((current) => current.reverse()),
    finish: control((current) => current.finish()),
    cancel: control((current) => current.cancel()),
  };
}

function toPlatformKeyframes(frames: AnimationKeyframes): Keyframe[] | PropertyIndexedKeyframes {
  if (isKeyframeList(frames)) return frames.map((frame) => ({ ...frame }));
  const indexed: PropertyIndexedKeyframes = {};
  for (const [key, value] of Object.entries(frames)) {
    if (value !== undefined) indexed[key] = copyIndexedValue(value);
  }
  return indexed;
}

function copyIndexedValue(
  value: IndexedKeyframeValue,
): string | number | null | string[] | (number | null)[] {
  return typeof value === "object" && value !== null ? value.slice() : value;
}

function isKeyframeList(frames: AnimationKeyframes): frames is readonly TypedKeyframe[] {
  return Array.isArray(frames);
}

function isKeyframeEffect(effect: AnimationEffect): effect is KeyframeEffect {
  return "setKeyframes" in effect && typeof effect.setKeyframes === "function";
}

function browserFrameHost(): AnimateFrameHost | undefined {
  return typeof window !== "undefined" ? window : undefined;
}
