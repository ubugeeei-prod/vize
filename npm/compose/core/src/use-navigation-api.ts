import { computed, readonly, shallowRef, toValue, watch } from "vue";
import type { ComputedRef, MaybeRefOrGetter, ShallowRef } from "vue";

import { tryOnScopeDispose } from "./scope.ts";

/** Navigation types of the Navigation API. */
export type NavigationTypeName = "push" | "replace" | "reload" | "traverse";

/** History handling requested by `navigate`. */
export type NavigationHistoryMode = "auto" | "push" | "replace";

/** Events of the Navigation API observed by {@link useNavigationApi}. */
export type NavigationEventName =
  | "navigate"
  | "navigatesuccess"
  | "navigateerror"
  | "currententrychange";

/** Minimal `NavigationHistoryEntry`. */
export interface NavigationHistoryEntryLike {
  /** Entry URL, or null when hidden (cross-origin). */
  readonly url: string | null;
  /** Key identifying the history slot. */
  readonly key: string;
  /** Unique identifier of the entry. */
  readonly id: string;
  /** Index in the entry list, or -1. */
  readonly index: number;
  /** Whether the entry belongs to the current document. */
  readonly sameDocument: boolean;
  /** Developer-defined state. */
  getState(): unknown;
}

/** Minimal `NavigationDestination`. */
export interface NavigationDestinationLike {
  /** Destination URL. */
  readonly url: string;
  /** Index of the destination entry for traversals, otherwise -1. */
  readonly index: number;
  /** Whether the destination is the current document. */
  readonly sameDocument: boolean;
  /** State of the destination. */
  getState(): unknown;
}

/** Options accepted by {@link NavigateEventLike.intercept}. */
export interface NavigationInterceptInit {
  /** Performs the same-document navigation. */
  readonly handler?: () => Promise<void>;
  /** Focus behavior after the navigation. */
  readonly focusReset?: "after-transition" | "manual";
  /** Scroll behavior after the navigation. */
  readonly scroll?: "after-transition" | "manual";
}

/** Minimal `NavigateEvent`. */
export interface NavigateEventLike {
  /** Navigation type. */
  readonly navigationType: NavigationTypeName;
  /** Where the navigation goes. */
  readonly destination: NavigationDestinationLike;
  /** Whether `intercept` may be called. */
  readonly canIntercept: boolean;
  /** Whether the user initiated the navigation. */
  readonly userInitiated: boolean;
  /** Whether only the fragment changes. */
  readonly hashChange: boolean;
  /** Download file name for `<a download>` navigations, else null. */
  readonly downloadRequest: string | null;
  /** Aborted when the navigation is cancelled. */
  readonly signal: AbortSignal;
  /** Info passed by the initiator. */
  readonly info: unknown;
  /** Convert the navigation into a same-document navigation. */
  intercept(options?: NavigationInterceptInit): void;
  /** Cancel the navigation when cancelable. */
  preventDefault(): void;
}

/** Minimal result of a Navigation API action. */
export interface NavigationResultLike {
  /** Resolves when the URL and entry changed. */
  readonly committed?: Promise<NavigationHistoryEntryLike>;
  /** Resolves when the navigation finished (handlers settled). */
  readonly finished?: Promise<NavigationHistoryEntryLike>;
}

/** Minimal `window.navigation`. */
export interface NavigationHost {
  /** Current entry. */
  readonly currentEntry: NavigationHistoryEntryLike | null;
  /** Ongoing intercepted navigation. */
  readonly transition: {
    /** Navigation type. */
    readonly navigationType: NavigationTypeName;
    /** Entry navigated away from. */
    readonly from: NavigationHistoryEntryLike;
  } | null;
  /** Whether `back` is possible. */
  readonly canGoBack: boolean;
  /** Whether `forward` is possible. */
  readonly canGoForward: boolean;
  /** Same-origin entries of the session history. */
  entries(): readonly NavigationHistoryEntryLike[];
  /** Navigate to `url`. */
  navigate(
    url: string,
    options?: { state?: unknown; history?: NavigationHistoryMode; info?: unknown },
  ): NavigationResultLike;
  /** Traverse one entry back. */
  back(options?: { info?: unknown }): NavigationResultLike;
  /** Traverse one entry forward. */
  forward(options?: { info?: unknown }): NavigationResultLike;
  /** Traverse to the entry with `key`. */
  traverseTo(key: string, options?: { info?: unknown }): NavigationResultLike;
  /** Reload the current entry. */
  reload(options?: { state?: unknown; info?: unknown }): NavigationResultLike;
  /** Replace the current entry's state without navigating. */
  updateCurrentEntry(options: { state: unknown }): void;
  /** Subscribe to a navigation event. */
  addEventListener(type: NavigationEventName, listener: (event: Event) => void): void;
  /** Unsubscribe from a navigation event. */
  removeEventListener(type: NavigationEventName, listener: (event: Event) => void): void;
}

/** Plain snapshot of a history entry. */
export interface NavigationEntrySnapshot {
  /** Entry URL, or null when hidden. */
  readonly url: string | null;
  /** Key identifying the history slot. */
  readonly key: string;
  /** Unique identifier of the entry. */
  readonly id: string;
  /** Index in the entry list. */
  readonly index: number;
  /** Whether the entry belongs to the current document. */
  readonly sameDocument: boolean;
}

/** Plain snapshot of the ongoing navigation. */
export interface NavigationTransitionSnapshot {
  /** Navigation type. */
  readonly navigationType: NavigationTypeName;
  /** Entry navigated away from. */
  readonly from: NavigationEntrySnapshot;
}

/** Details passed to `onCurrentEntryChange` listeners. */
export interface NavigationEntryChange {
  /** Navigation type, or null for `updateCurrentEntry`. */
  readonly navigationType: NavigationTypeName | null;
  /** Previous entry. */
  readonly from: NavigationEntrySnapshot;
}

/** Discriminated outcome of a navigation action. */
export type NavigationOutcome =
  | {
      /** The navigation finished. */
      readonly status: "success";
      /** Entry navigated to. */
      readonly entry: NavigationEntrySnapshot;
    }
  | {
      /** The Navigation API is missing. */
      readonly status: "unsupported";
    }
  | {
      /** The navigation failed or was aborted. */
      readonly status: "error";
      /** Error thrown or rejected by the host. */
      readonly error: unknown;
    };

/** Options of {@link NavigationApiControls.navigate}. */
export interface NavigationNavigateOptions<State> {
  /**
   * State stored on the new entry.
   *
   * @default undefined
   */
  readonly state?: State;
  /**
   * History handling.
   *
   * @default "auto"
   */
  readonly history?: NavigationHistoryMode;
  /**
   * Info handed to `navigate` listeners.
   *
   * @default undefined
   */
  readonly info?: unknown;
}

/** Options of {@link NavigationApiControls.intercept}. */
export interface NavigationInterceptOptions {
  /**
   * Runs the same-document navigation.
   *
   * @default undefined (commit the URL only)
   */
  readonly handler?: (event: NavigateEventLike) => Promise<void> | void;
  /**
   * Focus behavior after the navigation.
   *
   * @default "after-transition"
   */
  readonly focusReset?: "after-transition" | "manual";
  /**
   * Scroll behavior after the navigation.
   *
   * @default "after-transition"
   */
  readonly scroll?: "after-transition" | "manual";
}

/** Options for {@link useNavigationApi}. */
export interface UseNavigationApiOptions<State = unknown> {
  /**
   * Navigation API capability for alternate runtimes and tests.
   *
   * @default window.navigation when it exists
   */
  readonly navigation?: MaybeRefOrGetter<NavigationHost | null | undefined>;

  /**
   * Validate or convert raw entry state into `State`; return undefined for
   * foreign state.
   *
   * @default the raw state, trusted as `State`
   */
  readonly parseState?: (raw: unknown) => State | undefined;
}

/** Reactive state and actions returned by {@link useNavigationApi}. */
export interface NavigationApiControls<State> {
  /** Whether the Navigation API is available. */
  readonly supported: ComputedRef<boolean>;
  /** Snapshot of the current entry. */
  readonly currentEntry: Readonly<ShallowRef<NavigationEntrySnapshot | null>>;
  /** Snapshots of the same-origin session entries. */
  readonly entries: Readonly<ShallowRef<readonly NavigationEntrySnapshot[]>>;
  /** Whether `back` is possible. */
  readonly canGoBack: Readonly<ShallowRef<boolean>>;
  /** Whether `forward` is possible. */
  readonly canGoForward: Readonly<ShallowRef<boolean>>;
  /** Ongoing intercepted navigation. */
  readonly transition: Readonly<ShallowRef<NavigationTransitionSnapshot | null>>;
  /** State of the current entry (not proxied), refreshed with the entry. */
  readonly state: ComputedRef<State | undefined>;
  /** Most recent failed action or `navigateerror`, cleared on success. */
  readonly error: Readonly<ShallowRef<unknown>>;

  /**
   * Read the current entry's state now.
   *
   * @returns The parsed state.
   */
  readonly getState: () => State | undefined;

  /**
   * Listen to `navigate`.
   *
   * @param listener Receives the navigate event.
   * @returns Removes the listener.
   */
  readonly onNavigate: (listener: (event: NavigateEventLike) => void) => () => void;

  /**
   * Listen to `navigatesuccess`.
   *
   * @param listener Called after a navigation finished.
   * @returns Removes the listener.
   */
  readonly onNavigateSuccess: (listener: () => void) => () => void;

  /**
   * Listen to `navigateerror`.
   *
   * @param listener Receives the navigation error.
   * @returns Removes the listener.
   */
  readonly onNavigateError: (listener: (error: unknown) => void) => () => void;

  /**
   * Listen to `currententrychange`.
   *
   * @param listener Receives the navigation type and previous entry.
   * @returns Removes the listener.
   */
  readonly onCurrentEntryChange: (listener: (change: NavigationEntryChange) => void) => () => void;

  /**
   * Intercept interceptable, same-origin, non-download, non-hash navigations
   * whose destination matches `predicate`.
   *
   * @param predicate Receives the destination URL and the event.
   * @param options Handler, focus and scroll behavior.
   * @default options {}
   * @returns Removes the interceptor.
   */
  readonly intercept: (
    predicate: (url: URL, event: NavigateEventLike) => boolean,
    options?: NavigationInterceptOptions,
  ) => () => void;

  /**
   * Navigate to `url`.
   *
   * @param url Destination.
   * @param options State, history mode and info.
   * @default options {}
   * @returns The outcome once finished.
   */
  readonly navigate: (
    url: string,
    options?: NavigationNavigateOptions<State>,
  ) => Promise<NavigationOutcome>;

  /**
   * Go back one entry.
   *
   * @param info Info handed to `navigate` listeners.
   * @returns The outcome once finished.
   */
  readonly back: (info?: unknown) => Promise<NavigationOutcome>;

  /**
   * Go forward one entry.
   *
   * @param info Info handed to `navigate` listeners.
   * @returns The outcome once finished.
   */
  readonly forward: (info?: unknown) => Promise<NavigationOutcome>;

  /**
   * Traverse to the entry with `key`.
   *
   * @param key Entry key.
   * @param info Info handed to `navigate` listeners.
   * @returns The outcome once finished.
   */
  readonly traverseTo: (key: string, info?: unknown) => Promise<NavigationOutcome>;

  /**
   * Reload the current entry, optionally replacing its state.
   *
   * @param options New state and info.
   * @default options {}
   * @returns The outcome once finished.
   */
  readonly reload: (options?: {
    readonly state?: State;
    readonly info?: unknown;
  }) => Promise<NavigationOutcome>;

  /**
   * Replace the current entry's state without navigating.
   *
   * @param options New state.
   * @returns Whether the state was updated.
   */
  readonly updateCurrentEntry: (options: { readonly state: State }) => boolean;
}

function isNavigationHost(candidate: unknown): candidate is NavigationHost {
  return (
    typeof candidate === "object" &&
    candidate !== null &&
    typeof Reflect.get(candidate, "navigate") === "function" &&
    typeof Reflect.get(candidate, "addEventListener") === "function"
  );
}

function browserNavigation(): NavigationHost | undefined {
  if (typeof window === "undefined") return undefined;
  const candidate: unknown = Reflect.get(window, "navigation");
  return isNavigationHost(candidate) ? candidate : undefined;
}

function isNavigateEvent(event: Event): event is Event & NavigateEventLike {
  return "destination" in event && "canIntercept" in event && "intercept" in event;
}

function isEntryChangeEvent(event: Event): event is Event & {
  readonly navigationType: NavigationTypeName | null;
  readonly from: NavigationHistoryEntryLike;
} {
  return "from" in event && "navigationType" in event;
}

function snapshot(entry: NavigationHistoryEntryLike): NavigationEntrySnapshot {
  return {
    url: entry.url,
    key: entry.key,
    id: entry.id,
    index: entry.index,
    sameDocument: entry.sameDocument,
  };
}

function ignore(): void {
  // `finished` carries the outcome.
}

/**
 * Reactive access to the Navigation API (`window.navigation`).
 *
 * Entry snapshots refresh on `currententrychange`, `navigatesuccess` and
 * `navigateerror`. Actions resolve to a {@link NavigationOutcome} and never
 * reject; failures also land in `error`. Listeners (including interceptors)
 * are removed when the owning reactive scope stops; outside a scope the
 * caller removes them with the returned functions.
 *
 * Server rendering: nothing is read or subscribed; `supported` is false,
 * `currentEntry` null, `entries` empty, and actions resolve to
 * `{ status: "unsupported" }`.
 *
 * @example
 * ```ts
 * const nav = useNavigationApi<{ tab: string }>();
 * nav.intercept((url) => url.pathname.startsWith("/app/"), {
 *   handler: async () => renderRoute(location.pathname),
 * });
 * ```
 *
 * @param options Navigation host and state parser.
 * @default options {}
 * @returns Navigation state, hooks and actions.
 */
export function useNavigationApi<State = unknown>(
  options: UseNavigationApiOptions<State> = {},
): NavigationApiControls<State> {
  const currentEntry = shallowRef<NavigationEntrySnapshot | null>(null);
  const entries = shallowRef<readonly NavigationEntrySnapshot[]>([]);
  const canGoBack = shallowRef(false);
  const canGoForward = shallowRef(false);
  const transition = shallowRef<NavigationTransitionSnapshot | null>(null);
  const state = shallowRef<State | undefined>(undefined);
  const error = shallowRef<unknown>(undefined);

  const navigateListeners = new Set<(event: NavigateEventLike) => void>();
  const successListeners = new Set<() => void>();
  const errorListeners = new Set<(error: unknown) => void>();
  const changeListeners = new Set<(change: NavigationEntryChange) => void>();

  const resolveHost = (): NavigationHost | undefined => {
    if (options.navigation === undefined) return browserNavigation();
    return toValue(options.navigation) ?? undefined;
  };

  const parse = (raw: unknown): State | undefined => {
    if (options.parseState) return options.parseState(raw);
    // Without a parser the caller vouches for the stored state's type.
    return raw as State | undefined;
  };

  const getState = (): State | undefined => {
    const entry = resolveHost()?.currentEntry;
    return entry ? parse(entry.getState()) : undefined;
  };

  const refresh = (host: NavigationHost | undefined): void => {
    const entry = host?.currentEntry ?? null;
    currentEntry.value = entry ? snapshot(entry) : null;
    entries.value = host ? host.entries().map(snapshot) : [];
    canGoBack.value = host?.canGoBack ?? false;
    canGoForward.value = host?.canGoForward ?? false;
    const active = host?.transition ?? null;
    transition.value = active
      ? { navigationType: active.navigationType, from: snapshot(active.from) }
      : null;
    state.value = entry ? parse(entry.getState()) : undefined;
  };

  const add = <Listener>(set: Set<Listener>, listener: Listener): (() => void) => {
    set.add(listener);
    return () => {
      set.delete(listener);
    };
  };

  const stopWatch = watch(
    resolveHost,
    (host, _previous, onCleanup) => {
      refresh(host);
      if (!host) return;
      const onNavigate = (event: Event): void => {
        if (!isNavigateEvent(event)) return;
        for (const listener of Array.from(navigateListeners)) listener(event);
      };
      const onSuccess = (): void => {
        refresh(host);
        for (const listener of Array.from(successListeners)) listener();
      };
      const onError = (event: Event): void => {
        refresh(host);
        const cause: unknown = Reflect.get(event, "error");
        error.value = cause;
        for (const listener of Array.from(errorListeners)) listener(cause);
      };
      const onChange = (event: Event): void => {
        refresh(host);
        if (!isEntryChangeEvent(event)) return;
        const change = { navigationType: event.navigationType, from: snapshot(event.from) };
        for (const listener of Array.from(changeListeners)) listener(change);
      };
      host.addEventListener("navigate", onNavigate);
      host.addEventListener("navigatesuccess", onSuccess);
      host.addEventListener("navigateerror", onError);
      host.addEventListener("currententrychange", onChange);
      onCleanup(() => {
        host.removeEventListener("navigate", onNavigate);
        host.removeEventListener("navigatesuccess", onSuccess);
        host.removeEventListener("navigateerror", onError);
        host.removeEventListener("currententrychange", onChange);
      });
    },
    { immediate: true, flush: "sync" },
  );

  const perform = async (
    action: (host: NavigationHost) => NavigationResultLike,
  ): Promise<NavigationOutcome> => {
    const host = resolveHost();
    if (!host) return { status: "unsupported" };
    try {
      const result = action(host);
      result.committed?.catch(ignore);
      const entry = (await result.finished) ?? host.currentEntry;
      error.value = undefined;
      refresh(host);
      return entry
        ? { status: "success", entry: snapshot(entry) }
        : { status: "error", error: undefined };
    } catch (cause) {
      error.value = cause;
      return { status: "error", error: cause };
    }
  };

  const intercept = (
    predicate: (url: URL, event: NavigateEventLike) => boolean,
    interceptOptions: NavigationInterceptOptions = {},
  ): (() => void) =>
    add(navigateListeners, (event: NavigateEventLike) => {
      if (!event.canIntercept || event.hashChange || event.downloadRequest !== null) return;
      const url = new URL(event.destination.url);
      const current = resolveHost()?.currentEntry?.url;
      if (current && new URL(current).origin !== url.origin) return;
      if (!predicate(url, event)) return;
      const { handler, focusReset, scroll } = interceptOptions;
      event.intercept({
        ...(handler ? { handler: async () => handler(event) } : {}),
        ...(focusReset ? { focusReset } : {}),
        ...(scroll ? { scroll } : {}),
      });
    });

  const infoOption = (info: unknown): { info?: unknown } => (info === undefined ? {} : { info });

  tryOnScopeDispose(() => {
    stopWatch();
    navigateListeners.clear();
    successListeners.clear();
    errorListeners.clear();
    changeListeners.clear();
  });

  return {
    supported: computed(() => resolveHost() !== undefined),
    currentEntry: readonly(currentEntry),
    entries: readonly(entries),
    canGoBack: readonly(canGoBack),
    canGoForward: readonly(canGoForward),
    transition: readonly(transition),
    state: computed(() => state.value),
    error: readonly(error),
    getState,
    onNavigate: (listener) => add(navigateListeners, listener),
    onNavigateSuccess: (listener) => add(successListeners, listener),
    onNavigateError: (listener) => add(errorListeners, listener),
    onCurrentEntryChange: (listener) => add(changeListeners, listener),
    intercept,
    navigate: (url, navigateOptions = {}) =>
      perform((host) =>
        host.navigate(url, {
          ...("state" in navigateOptions ? { state: navigateOptions.state } : {}),
          ...(navigateOptions.history ? { history: navigateOptions.history } : {}),
          ...infoOption(navigateOptions.info),
        }),
      ),
    back: (info) => perform((host) => host.back(infoOption(info))),
    forward: (info) => perform((host) => host.forward(infoOption(info))),
    traverseTo: (key, info) => perform((host) => host.traverseTo(key, infoOption(info))),
    reload: (reloadOptions = {}) =>
      perform((host) =>
        host.reload({
          ...("state" in reloadOptions ? { state: reloadOptions.state } : {}),
          ...infoOption(reloadOptions.info),
        }),
      ),
    updateCurrentEntry: ({ state: next }) => {
      const host = resolveHost();
      if (!host) return false;
      try {
        host.updateCurrentEntry({ state: next });
        error.value = undefined;
        refresh(host);
        return true;
      } catch (cause) {
        error.value = cause;
        return false;
      }
    },
  };
}
