import { computed, readonly, shallowRef, toValue } from "vue";
import type { ComputedRef, MaybeRefOrGetter, ShallowRef } from "vue";

import { useReducedMotion } from "./media-query.ts";
import { tryOnScopeDispose } from "./scope.ts";

/** Minimal `ViewTransition`. */
export interface ViewTransitionLike {
  /** Settles when the transition animation finished (or was skipped). */
  readonly finished: Promise<void>;
  /** Resolves when the animation is about to start; rejects when skipped. */
  readonly ready: Promise<void>;
  /** Settles when the update callback settled. */
  readonly updateCallbackDone: Promise<void>;
  /** Skip the animation; the update still runs. */
  skipTransition(): void;
}

/** Argument accepted by `document.startViewTransition`. */
export type StartViewTransitionArgument =
  | (() => unknown)
  | {
      /** DOM update callback. */
      readonly update?: () => unknown;
      /** Active transition types (`:active-view-transition-type()`). */
      readonly types?: string[];
    };

/** Minimal document consumed by {@link useViewTransition}. */
export interface ViewTransitionDocumentHost {
  /** Start a same-document view transition. */
  startViewTransition(update?: StartViewTransitionArgument): ViewTransitionLike;
}

/** Options for {@link useViewTransition}. */
export interface UseViewTransitionOptions {
  /**
   * Document for alternate runtimes and tests.
   *
   * @default window.document when it implements startViewTransition
   */
  readonly document?: MaybeRefOrGetter<ViewTransitionDocumentHost | null | undefined>;

  /**
   * Run updates without a transition while reduced motion is preferred.
   *
   * @default true
   */
  readonly respectReducedMotion?: boolean;

  /**
   * Whether reduced motion is preferred.
   *
   * @default useReducedMotion() === "reduce"
   */
  readonly reducedMotion?: MaybeRefOrGetter<boolean>;
}

/** Per-call options of {@link ViewTransitionControls.start}. */
export interface ViewTransitionStartOptions {
  /**
   * Transition types. Runtimes without typed transitions fall back to an
   * untyped transition.
   *
   * @default []
   */
  readonly types?: readonly string[];
}

/** Reactive state and actions returned by {@link useViewTransition}. */
export interface ViewTransitionControls {
  /** Whether `document.startViewTransition` is available. */
  readonly supported: ComputedRef<boolean>;

  /**
   * The running transition (the host object itself, not a readonly proxy, so
   * native methods keep working), cleared once it finished.
   */
  readonly transition: ComputedRef<ViewTransitionLike | null>;

  /** Whether a transition started by this composable is running. */
  readonly isTransitioning: Readonly<ShallowRef<boolean>>;

  /**
   * Run `update` inside a view transition, or directly when unsupported or
   * reduced motion is preferred.
   *
   * @param update DOM update; may return a value or a promise.
   * @param options Transition types.
   * @default options {}
   * @returns Resolves with the update result once the DOM was updated (the
   * animation may still run; await `transition.value?.finished` for it);
   * rejects with the update error.
   */
  readonly start: <Result>(
    update: () => Result | PromiseLike<Result>,
    options?: ViewTransitionStartOptions,
  ) => Promise<Result>;

  /** Skip the running transition's animation. */
  readonly skip: () => void;
}

function browserDocument(): ViewTransitionDocumentHost | undefined {
  return typeof window !== "undefined" && typeof window.document.startViewTransition === "function"
    ? window.document
    : undefined;
}

function ignore(): void {
  // Settlement is observed elsewhere.
}

/**
 * Animate DOM updates with the View Transition API.
 *
 * Starting a new transition skips the running one (browser behavior). The
 * running transition is skipped when the owning reactive scope stops.
 *
 * Server rendering: `start` runs the update directly; `supported` and
 * `isTransitioning` are false.
 *
 * @example
 * ```ts
 * const { start } = useViewTransition();
 * await start(async () => {
 *   page.value = next;
 *   await nextTick();
 * }, { types: ["forward"] });
 * ```
 *
 * @param options Document host and reduced-motion policy.
 * @default options {}
 * @returns Transition state and actions.
 */
export function useViewTransition(options: UseViewTransitionOptions = {}): ViewTransitionControls {
  const transition = shallowRef<ViewTransitionLike | null>(null);
  const isTransitioning = shallowRef(false);
  const respect = options.respectReducedMotion ?? true;
  const preference =
    respect && options.reducedMotion === undefined ? useReducedMotion() : undefined;

  const resolveDocument = (): ViewTransitionDocumentHost | undefined =>
    options.document === undefined ? browserDocument() : (toValue(options.document) ?? undefined);

  const prefersReducedMotion = (): boolean => {
    if (!respect) return false;
    return preference ? preference.value === "reduce" : toValue(options.reducedMotion) === true;
  };

  const begin = (
    host: ViewTransitionDocumentHost,
    run: () => Promise<void>,
    types: readonly string[],
  ): ViewTransitionLike => {
    if (types.length === 0) return host.startViewTransition(run);
    try {
      return host.startViewTransition({ update: run, types: [...types] });
    } catch {
      // Runtimes without typed transitions reject the options object.
      return host.startViewTransition(run);
    }
  };

  const start = async <Result>(
    update: () => Result | PromiseLike<Result>,
    startOptions: ViewTransitionStartOptions = {},
  ): Promise<Result> => {
    const host = resolveDocument();
    if (!host || prefersReducedMotion()) return update();

    let run: () => Promise<void> = async () => undefined;
    const outcome = new Promise<Result>((resolve, reject) => {
      run = async () => {
        try {
          resolve(await update());
        } catch (cause) {
          reject(cause);
          throw cause;
        }
      };
    });

    let current: ViewTransitionLike;
    try {
      current = begin(host, run, startOptions.types ?? []);
    } catch {
      return update();
    }
    transition.value = current;
    isTransitioning.value = true;
    const clear = (): void => {
      if (transition.value !== current) return;
      transition.value = null;
      isTransitioning.value = false;
    };
    current.ready.catch(ignore);
    current.updateCallbackDone.catch(ignore);
    current.finished.then(clear, clear);
    return outcome;
  };

  const skip = (): void => {
    transition.value?.skipTransition();
  };

  tryOnScopeDispose(skip);

  return {
    supported: computed(() => resolveDocument() !== undefined),
    transition: computed(() => transition.value),
    isTransitioning: readonly(isTransitioning),
    start,
    skip,
  };
}
