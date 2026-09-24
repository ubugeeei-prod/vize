import { readonly, ref, toValue, watch } from "vue";
import type { ComponentPublicInstance, MaybeRefOrGetter, Ref, WatchHandle } from "vue";

import { resolveElement, resolveElements } from "./element-target.ts";
import type { MaybeElementTarget, MaybeElementTargets } from "./element-target.ts";
import { tryOnScopeDispose } from "./scope.ts";

/** Runtime capability that supplies the `IntersectionObserver` constructor. */
export interface IntersectionObserverHost {
  /** `IntersectionObserver` constructor, absent in unsupported runtimes. */
  readonly IntersectionObserver?: new (
    callback: IntersectionObserverCallback,
    options?: IntersectionObserverInit,
  ) => IntersectionObserver;
}

/**
 * Intersection root: an element target, a document, or `null`/`undefined`
 * for the top-level viewport.
 */
export type IntersectionRoot = MaybeRefOrGetter<
  Element | Document | ComponentPublicInstance | null | undefined
>;

/** Options for {@link useIntersectionObserver}. */
export interface UseIntersectionObserverOptions {
  /**
   * Ancestor (or document) whose box is used as the viewport.
   *
   * @default undefined (the top-level viewport)
   */
  readonly root?: IntersectionRoot;

  /**
   * Margin grown or shrunk around the root box, in CSS margin syntax.
   *
   * @default "0px"
   */
  readonly rootMargin?: MaybeRefOrGetter<string>;

  /**
   * Visible-ratio thresholds that trigger the callback.
   *
   * @default 0
   */
  readonly threshold?: MaybeRefOrGetter<number | readonly number[]>;

  /**
   * Start observing during composable creation. When `false`, call `resume`.
   *
   * @default true
   */
  readonly immediate?: boolean;

  /**
   * Reactive constructor capability for alternate runtimes and tests.
   *
   * @default globalThis when it provides `IntersectionObserver`
   */
  readonly host?: MaybeRefOrGetter<IntersectionObserverHost | null | undefined>;

  /**
   * Target resolution timing. `"post"` observes template refs after mount.
   *
   * @default "post"
   */
  readonly flush?: "pre" | "post" | "sync";
}

/** Controls returned by {@link useIntersectionObserver}. */
export interface IntersectionObserverControls {
  /** Whether the host provides `IntersectionObserver`. `false` during server rendering. */
  readonly isSupported: Readonly<Ref<boolean>>;

  /** Whether observation is currently requested (not paused or stopped). */
  readonly isActive: Readonly<Ref<boolean>>;

  /** Disconnect the observer while keeping the ability to `resume`. */
  readonly pause: () => void;

  /** Reconnect after `pause` (or start when created with `immediate: false`). */
  readonly resume: () => void;

  /** Disconnect permanently. Later `resume` calls are ignored. Idempotent. */
  readonly stop: () => void;
}

/**
 * Observe the intersection of reactive elements with a root.
 *
 * One observer is created per combination of resolved targets, root,
 * margin, and threshold; changing any of them disconnects the previous
 * observer first. During server rendering `isSupported` is `false` and the
 * callback never runs. The observer is disconnected when the owning scope
 * stops.
 *
 * @param targets Reactive element target or list of targets.
 * @param callback Native `IntersectionObserver` callback.
 * @param options Root, margin, thresholds, capability, and timing.
 * @default options {}
 * @returns Support/activity flags and pause/resume/stop controls.
 */
export function useIntersectionObserver(
  targets: MaybeElementTargets,
  callback: IntersectionObserverCallback,
  options: UseIntersectionObserverOptions = {},
): IntersectionObserverControls {
  const isSupported = ref(false);
  const isActive = ref(options.immediate ?? true);
  let stopped = false;
  let stopWatch: WatchHandle | undefined;

  const host = (): IntersectionObserverHost | null | undefined =>
    options.host === undefined ? browserIntersectionObserverHost() : toValue(options.host);
  isSupported.value = host()?.IntersectionObserver !== undefined;

  const connect = (): void => {
    stopWatch = watch(
      () =>
        [
          host(),
          resolveElements(targets),
          resolveRoot(options.root),
          toValue(options.rootMargin) ?? "0px",
          toValue(options.threshold) ?? 0,
        ] as const,
      ([currentHost, elements, root, rootMargin, threshold], _previous, onCleanup) => {
        const Observer = currentHost?.IntersectionObserver;
        isSupported.value = Observer !== undefined;
        if (!Observer || elements.length === 0) return;
        const observer = new Observer(callback, {
          root,
          rootMargin,
          threshold: typeof threshold === "number" ? threshold : [...threshold],
        });
        for (const element of elements) observer.observe(element);
        onCleanup(() => observer.disconnect());
      },
      { immediate: true, flush: options.flush ?? "post" },
    );
  };

  const pause = (): void => {
    stopWatch?.stop();
    stopWatch = undefined;
    isActive.value = false;
  };
  const resume = (): void => {
    if (stopped || stopWatch) return;
    isActive.value = true;
    connect();
  };
  const stop = (): void => {
    stopped = true;
    pause();
  };

  if (isActive.value) connect();
  tryOnScopeDispose(stop);

  return {
    isSupported: readonly(isSupported),
    isActive: readonly(isActive),
    pause,
    resume,
    stop,
  };
}

/** Options for {@link useElementVisibility}. */
export interface UseElementVisibilityOptions extends Omit<
  UseIntersectionObserverOptions,
  "immediate"
> {
  /**
   * Visibility exposed before the first observation and during server
   * rendering. Keep it identical on server and client.
   *
   * @default false
   */
  readonly initialValue?: boolean;

  /**
   * Stop observing after the element becomes visible for the first time.
   *
   * @default false
   */
  readonly once?: boolean;
}

/** Reactive state returned by {@link useElementVisibility}. */
export interface ElementVisibilityControls {
  /** Whether the element currently intersects the root. */
  readonly isVisible: Readonly<Ref<boolean>>;

  /** Whether the host provides `IntersectionObserver`. */
  readonly isSupported: Readonly<Ref<boolean>>;

  /** Stop observing. Idempotent. */
  readonly stop: () => void;
}

/**
 * Track whether an element is visible inside the viewport (or a root).
 *
 * Uses the latest intersection entry, so rapid enter/leave sequences settle
 * on the final state. With `once`, the observer disconnects after the first
 * visible entry and `isVisible` stays `true`.
 *
 * @param target Reactive element target.
 * @param options Initial value, one-shot mode, and observer options.
 * @default options {}
 * @returns Reactive visibility plus controls.
 */
export function useElementVisibility(
  target: MaybeElementTarget,
  options: UseElementVisibilityOptions = {},
): ElementVisibilityControls {
  const isVisible = ref(options.initialValue ?? false);
  const controls = useIntersectionObserver(
    target,
    (entries) => {
      let latest: IntersectionObserverEntry | undefined;
      for (const entry of entries) {
        if (!latest || entry.time >= latest.time) latest = entry;
      }
      if (!latest) return;
      isVisible.value = latest.isIntersecting;
      if (options.once && latest.isIntersecting) controls.stop();
    },
    options,
  );
  return { isVisible: readonly(isVisible), isSupported: controls.isSupported, stop: controls.stop };
}

function resolveRoot(root: IntersectionRoot | undefined): Element | Document | null {
  if (root === undefined) return null;
  const value = toValue(root);
  return isDocumentNode(value) ? value : resolveElement(value);
}

function isDocumentNode(value: unknown): value is Document {
  return typeof value === "object" && value !== null && "nodeType" in value && value.nodeType === 9;
}

function browserIntersectionObserverHost(): IntersectionObserverHost | undefined {
  return typeof IntersectionObserver === "function" ? globalThis : undefined;
}
