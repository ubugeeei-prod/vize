import { readonly, ref, toValue, watch } from "vue";
import type { MaybeRefOrGetter, Ref, WatchHandle } from "vue";

import { resolveElements } from "./element-target.ts";
import type { MaybeElementTargets } from "./element-target.ts";
import { tryOnScopeDispose } from "./scope.ts";

/** Runtime capability that supplies the `MutationObserver` constructor. */
export interface MutationObserverHost {
  /** `MutationObserver` constructor, absent in unsupported runtimes. */
  readonly MutationObserver?: new (callback: MutationCallback) => MutationObserver;
}

/** Options for {@link useMutationObserver}. */
export interface UseMutationObserverOptions {
  /**
   * Observe attribute changes.
   *
   * @default false (implicitly `true` when `attributeFilter` or `attributeOldValue` is set)
   */
  readonly attributes?: boolean;

  /**
   * Observe child-list changes.
   *
   * @default false
   */
  readonly childList?: boolean;

  /**
   * Extend observation to the whole subtree.
   *
   * @default false
   */
  readonly subtree?: boolean;

  /**
   * Observe text-content changes.
   *
   * @default false (implicitly `true` when `characterDataOldValue` is set)
   */
  readonly characterData?: boolean;

  /**
   * Record the previous attribute value.
   *
   * @default false
   */
  readonly attributeOldValue?: boolean;

  /**
   * Record the previous text value.
   *
   * @default false
   */
  readonly characterDataOldValue?: boolean;

  /**
   * Restrict attribute observation to these local names.
   *
   * @default undefined (all attributes)
   */
  readonly attributeFilter?: readonly string[];

  /**
   * Reactive constructor capability for alternate runtimes and tests.
   *
   * @default globalThis when it provides `MutationObserver`
   */
  readonly host?: MaybeRefOrGetter<MutationObserverHost | null | undefined>;

  /**
   * Target resolution timing. `"post"` observes template refs after mount.
   *
   * @default "post"
   */
  readonly flush?: "pre" | "post" | "sync";
}

/** Controls returned by {@link useMutationObserver}. */
export interface MutationObserverControls {
  /** Whether the host provides `MutationObserver`. `false` during server rendering. */
  readonly isSupported: Readonly<Ref<boolean>>;

  /**
   * Drain the records queued by the active observer without invoking the
   * callback.
   *
   * @returns Pending records, or an empty list without an active observer.
   */
  readonly takeRecords: () => MutationRecord[];

  /** Disconnect permanently. Idempotent. */
  readonly stop: () => void;
}

/**
 * Observe DOM mutations of reactive elements.
 *
 * The observer is recreated when the resolved targets or host change and is
 * disconnected when the owning reactive scope stops. At least one of
 * `attributes`, `childList`, or `characterData` must be enabled (directly or
 * implicitly), mirroring the platform contract. No work happens during
 * server rendering.
 *
 * @param targets Reactive element target or list of targets.
 * @param callback Native `MutationObserver` callback.
 * @param options Observation filter, capability, and timing.
 * @default options {}
 * @returns Support flag, record draining, and stop control.
 */
export function useMutationObserver(
  targets: MaybeElementTargets,
  callback: MutationCallback,
  options: UseMutationObserverOptions = {},
): MutationObserverControls {
  const isSupported = ref(false);
  const init = observerInit(options);
  let active: MutationObserver | undefined;
  let stopWatch: WatchHandle | undefined = watch(
    () =>
      [
        options.host === undefined ? browserMutationObserverHost() : toValue(options.host),
        resolveElements(targets),
      ] as const,
    ([host, elements], _previous, onCleanup) => {
      const Observer = host?.MutationObserver;
      isSupported.value = Observer !== undefined;
      if (!Observer || elements.length === 0) return;
      const observer = new Observer(callback);
      for (const element of elements) observer.observe(element, init);
      active = observer;
      onCleanup(() => {
        observer.disconnect();
        if (active === observer) active = undefined;
      });
    },
    { immediate: true, flush: options.flush ?? "post" },
  );

  const stop = (): void => {
    stopWatch?.stop();
    stopWatch = undefined;
  };
  tryOnScopeDispose(stop);

  return {
    isSupported: readonly(isSupported),
    takeRecords: () => active?.takeRecords() ?? [],
    stop,
  };
}

function observerInit(options: UseMutationObserverOptions): MutationObserverInit {
  const init: MutationObserverInit = {};
  if (options.attributes !== undefined) init.attributes = options.attributes;
  if (options.childList !== undefined) init.childList = options.childList;
  if (options.subtree !== undefined) init.subtree = options.subtree;
  if (options.characterData !== undefined) init.characterData = options.characterData;
  if (options.attributeOldValue !== undefined) init.attributeOldValue = options.attributeOldValue;
  if (options.characterDataOldValue !== undefined) {
    init.characterDataOldValue = options.characterDataOldValue;
  }
  if (options.attributeFilter !== undefined) init.attributeFilter = [...options.attributeFilter];
  return init;
}

function browserMutationObserverHost(): MutationObserverHost | undefined {
  return typeof MutationObserver === "function" ? globalThis : undefined;
}
