import { computed, readonly, ref, toValue, watchEffect } from "vue";
import type { ComputedRef, MaybeRefOrGetter, Ref } from "vue";

/** Page visibility state exposed by {@link useDocumentVisibility}. */
export type PageVisibilityState = "hidden" | "visible";

/** Capability required to observe document visibility. */
export interface DocumentVisibilityHost {
  /** Current browser visibility state. Unknown states are treated as hidden. */
  readonly visibilityState?: string;

  /** Legacy visibility flag used when `visibilityState` is absent. */
  readonly hidden?: boolean;

  /** Subscribe to `visibilitychange`. */
  readonly addEventListener: (
    event: "visibilitychange",
    listener: EventListener,
    options?: boolean | AddEventListenerOptions,
  ) => void;

  /** Remove a previous `visibilitychange` listener. */
  readonly removeEventListener: (
    event: "visibilitychange",
    listener: EventListener,
    options?: boolean | EventListenerOptions,
  ) => void;
}

/** Options for {@link useDocumentVisibility}. */
export interface UseDocumentVisibilityOptions {
  /**
   * State exposed while no document capability is available.
   *
   * @default "visible"
   */
  readonly ssrState?: PageVisibilityState;

  /**
   * Reactive document capability for alternate runtimes and tests.
   *
   * @default globalThis.document when available
   */
  readonly host?: MaybeRefOrGetter<DocumentVisibilityHost | null | undefined>;

  /**
   * Reactive subscription update timing.
   *
   * @default "pre"
   */
  readonly flush?: "pre" | "post" | "sync";
}

/** Reactive state returned by {@link useDocumentVisibility}. */
export interface DocumentVisibilityControls {
  /** Current visibility state, using the configured SSR fallback while unsupported. */
  readonly state: Readonly<Ref<PageVisibilityState>>;

  /** Whether a concrete document capability is currently attached. */
  readonly supported: Readonly<Ref<boolean>>;

  /** Whether `state` is `"visible"`. */
  readonly visible: ComputedRef<boolean>;

  /** Whether `state` is `"hidden"`. */
  readonly hidden: ComputedRef<boolean>;
}

/**
 * Observe document visibility without touching browser globals during module evaluation.
 *
 * The composable is safe during server rendering: when no host resolves, it exposes
 * `ssrState` and marks `supported` false. During hydration or tests, passing a
 * host attaches exactly one `visibilitychange` listener and removes it when the
 * reactive host changes or the owning scope stops. Unsupported or future
 * visibility states normalize to `"hidden"` so consumers fail closed instead of
 * accidentally treating a prerendered or unloaded page as visible.
 *
 * @param options Runtime capability, server fallback, and watcher timing.
 * @default options {}
 * @returns Reactive visibility state plus derived booleans.
 */
export function useDocumentVisibility(
  options: UseDocumentVisibilityOptions = {},
): DocumentVisibilityControls {
  const fallback = options.ssrState ?? "visible";
  const state = ref<PageVisibilityState>(fallback);
  const supported = ref(false);

  watchEffect(
    (onCleanup) => {
      const host =
        options.host === undefined ? browserDocumentVisibilityHost() : toValue(options.host);
      if (!host) {
        supported.value = false;
        state.value = fallback;
        return;
      }

      const update = (): void => {
        state.value = readVisibilityState(host);
      };
      supported.value = true;
      update();
      host.addEventListener("visibilitychange", update);
      onCleanup(() => {
        host.removeEventListener("visibilitychange", update);
        supported.value = false;
      });
    },
    { flush: options.flush ?? "pre" },
  );

  return {
    state: readonly(state),
    supported: readonly(supported),
    visible: computed(() => state.value === "visible"),
    hidden: computed(() => state.value === "hidden"),
  };
}

function readVisibilityState(host: DocumentVisibilityHost): PageVisibilityState {
  if (host.visibilityState === "visible") return "visible";
  if (host.visibilityState === "hidden") return "hidden";
  if (host.visibilityState !== undefined) return "hidden";
  return host.hidden === false ? "visible" : "hidden";
}

function browserDocumentVisibilityHost(): DocumentVisibilityHost | undefined {
  return typeof document !== "undefined" ? document : undefined;
}
