import {
  computed,
  hasInjectionContext,
  readonly,
  ref,
  shallowRef,
  toValue,
  watchPostEffect,
} from "vue";
import type { ComputedRef, MaybeRefOrGetter, Ref, ShallowRef } from "vue";

import { tryOnScopeDispose } from "./scope.ts";

/** Lock modes defined by the Web Locks API. */
export type WebLockMode = "exclusive" | "shared";

/** Granted lock handed to a request callback. */
export interface WebLockLike {
  /** Lock name. */
  readonly name: string;

  /** Granted mode. */
  readonly mode: WebLockMode;
}

/** One entry of a lock manager snapshot. */
export interface WebLockInfo {
  /** Lock name. */
  readonly name?: string | undefined;

  /** Requested or granted mode. */
  readonly mode?: WebLockMode | undefined;

  /** Identifier of the client holding or requesting the lock. */
  readonly clientId?: string | undefined;
}

/** Snapshot returned by {@link WebLocksControls.query}. */
export interface WebLocksSnapshot {
  /** Locks currently held by any client of the origin. */
  readonly held: readonly WebLockInfo[];

  /** Requests currently waiting for a lock. */
  readonly pending: readonly WebLockInfo[];
}

/** Options forwarded to `LockManager.request`. */
export interface WebLockManagerRequestOptions {
  /** Requested mode. */
  readonly mode: WebLockMode;

  /**
   * Only grant the lock when it is immediately available.
   *
   * @default false
   */
  readonly ifAvailable?: boolean;

  /**
   * Release any held lock of the same name and grant this request.
   *
   * @default false
   */
  readonly steal?: boolean;

  /**
   * Abort the request while it is pending.
   *
   * @default undefined
   */
  readonly signal?: AbortSignal;
}

/** Minimal `LockManager` (`navigator.locks`) consumed by {@link useWebLocks}. */
export interface LockManagerLike {
  /** Request a lock and run `callback` while it is held. */
  request(
    name: string,
    options: WebLockManagerRequestOptions,
    callback: (lock: WebLockLike | null) => unknown,
  ): Promise<unknown>;

  /** Snapshot the origin's held and pending locks. */
  query(): Promise<{
    /** Held locks. */
    readonly held?: readonly WebLockInfo[] | undefined;
    /** Pending requests. */
    readonly pending?: readonly WebLockInfo[] | undefined;
  }>;
}

/** Options for {@link useWebLocks}. */
export interface UseWebLocksOptions {
  /**
   * Lock manager for alternate runtimes and tests.
   *
   * @default window.navigator.locks when it exists
   */
  readonly locks?: MaybeRefOrGetter<LockManagerLike | null | undefined>;
}

/** Per-request options for {@link WebLocksControls.request}. */
export interface WebLockRequestOptions {
  /**
   * Requested mode.
   *
   * @default "exclusive"
   */
  readonly mode?: WebLockMode;

  /**
   * Resolve with `{ acquired: false }` instead of waiting when the lock is busy.
   *
   * @default false
   */
  readonly ifAvailable?: boolean;

  /**
   * Preempt any holder of the lock (exclusive mode only).
   *
   * @default false
   */
  readonly steal?: boolean;

  /**
   * Abort the request while it is still pending.
   *
   * @default undefined
   */
  readonly signal?: AbortSignal;
}

/** Outcome of an `ifAvailable` request. */
export type WebLockAttempt<Value> =
  | {
      /** The lock was granted and the callback ran. */
      readonly acquired: true;
      /** Callback result. */
      readonly value: Value;
    }
  | {
      /** The lock was busy; the callback did not run. */
      readonly acquired: false;
    };

/** Request function exposed by {@link WebLocksControls}. */
export interface WebLockRequest {
  /**
   * Request a lock that is only granted when immediately available.
   *
   * @param name Lock name.
   * @param callback Work to run while the lock is held.
   * @param options Request options with `ifAvailable: true`.
   * @returns The callback result, or `{ acquired: false }` when busy.
   */
  <Value>(
    name: string,
    callback: (lock: WebLockLike) => Value | PromiseLike<Value>,
    options: WebLockRequestOptions & { readonly ifAvailable: true },
  ): Promise<WebLockAttempt<Value>>;

  /**
   * Request a lock and run `callback` while it is held.
   *
   * @param name Lock name.
   * @param callback Work to run while the lock is held.
   * @param options Mode, steal and abort options.
   * @returns The callback result.
   */
  <Value>(
    name: string,
    callback: (lock: WebLockLike) => Value | PromiseLike<Value>,
    options?: WebLockRequestOptions & { readonly ifAvailable?: false },
  ): Promise<Value>;
}

/** Reactive state and actions returned by {@link useWebLocks}. */
export interface WebLocksControls {
  /** Whether the Web Locks API is available. */
  readonly supported: ComputedRef<boolean>;

  /** Names of locks requested through this composable that are currently held. */
  readonly held: Readonly<Ref<readonly string[]>>;

  /** Names of locks requested through this composable that are still waiting. */
  readonly pending: Readonly<Ref<readonly string[]>>;

  /** Most recent request failure (including aborts), cleared by the next grant. */
  readonly error: Readonly<ShallowRef<unknown>>;

  /**
   * Request a lock. Rejects with a tagged `TypeError` for invalid option
   * combinations, a tagged `Error` when unsupported, the host's `AbortError`
   * when aborted, or the callback's rejection.
   */
  readonly request: WebLockRequest;

  /**
   * Snapshot the origin's locks.
   *
   * @returns Held and pending locks; empty lists when unsupported or on failure.
   */
  readonly query: () => Promise<WebLocksSnapshot>;
}

function browserLocks(): LockManagerLike | undefined {
  if (typeof window === "undefined") return undefined;
  const { navigator } = window;
  return "locks" in navigator && navigator.locks ? navigator.locks : undefined;
}

function without(names: readonly string[], name: string): readonly string[] {
  const index = names.indexOf(name);
  return index === -1 ? names : [...names.slice(0, index), ...names.slice(index + 1)];
}

/**
 * Coordinate work across tabs and workers with the Web Locks API.
 *
 * `request` runs the callback while the named lock is held and resolves with
 * its result; with `ifAvailable` it resolves with a discriminated
 * `{ acquired }` outcome instead of waiting. `held` and `pending` track the
 * lock names requested through this composable. Requests still waiting when
 * the owning reactive scope stops are aborted; locks already held are
 * released when their callbacks settle. Outside a scope, pass a `signal` to
 * own cancellation.
 *
 * Server rendering: nothing is requested, `supported` is false and the name
 * lists are empty.
 *
 * @example
 * ```ts
 * const locks = useWebLocks();
 * await locks.request("sync", async () => syncOutbox());
 * const attempt = await locks.request("leader", runLeader, { ifAvailable: true });
 * ```
 *
 * @param options Lock manager override.
 * @default options {}
 * @returns Lock state and actions.
 */
export function useWebLocks(options: UseWebLocksOptions = {}): WebLocksControls {
  const held = ref<readonly string[]>([]);
  const pending = ref<readonly string[]>([]);
  const error = shallowRef<unknown>(undefined);
  const controllers = new Set<AbortController>();

  // Inside a component the host resolves only after mount (a post-flush
  // job), so a hydrating client first renders the same unsupported state as
  // the server. Outside components it resolves immediately.
  const hydrated = shallowRef(!hasInjectionContext());
  if (!hydrated.value) {
    watchPostEffect(() => {
      hydrated.value = true;
    });
  }
  const resolveLocks = (): LockManagerLike | undefined =>
    !hydrated.value
      ? undefined
      : options.locks === undefined
        ? browserLocks()
        : (toValue(options.locks) ?? undefined);

  const run = async (
    name: string,
    callback: (lock: WebLockLike) => unknown,
    requestOptions: WebLockRequestOptions = {},
  ): Promise<WebLockAttempt<unknown>> => {
    const { mode = "exclusive", ifAvailable = false, steal = false, signal } = requestOptions;
    if (
      (ifAvailable && steal) ||
      (steal && mode === "shared") ||
      (signal && (ifAvailable || steal))
    ) {
      throw new TypeError(
        "[VIZE_COMPOSE_WEB_LOCKS_INVALID_OPTIONS] steal cannot be combined with ifAvailable, shared mode or signal, and signal cannot be combined with ifAvailable.",
      );
    }
    const locks = resolveLocks();
    if (!locks) {
      throw new Error("[VIZE_COMPOSE_WEB_LOCKS_UNSUPPORTED] The Web Locks API is not available.");
    }

    const controller = ifAvailable || steal ? undefined : new AbortController();
    const forwardAbort = (): void => controller?.abort(signal?.reason);
    if (controller) {
      if (signal?.aborted) forwardAbort();
      else signal?.addEventListener("abort", forwardAbort, { once: true });
      controllers.add(controller);
    }
    pending.value = [...pending.value, name];
    let granted = false;
    const managerOptions: WebLockManagerRequestOptions = controller
      ? { mode, signal: controller.signal }
      : { mode, ifAvailable, steal };

    let outcome: WebLockAttempt<unknown> = { acquired: false };
    try {
      await locks.request(name, managerOptions, async (lock) => {
        pending.value = without(pending.value, name);
        if (controller) controllers.delete(controller);
        if (!lock) return;
        granted = true;
        error.value = undefined;
        held.value = [...held.value, name];
        try {
          outcome = { acquired: true, value: await callback(lock) };
        } finally {
          held.value = without(held.value, name);
        }
      });
      return outcome;
    } catch (cause) {
      error.value = cause;
      throw cause;
    } finally {
      if (!granted) pending.value = without(pending.value, name);
      if (controller) controllers.delete(controller);
      signal?.removeEventListener("abort", forwardAbort);
    }
  };

  // The overloads share one implementation: `run` reports the attempt and
  // this adapter unwraps it for waiting requests. The cast is confined here.
  const request = ((
    name: string,
    callback: (lock: WebLockLike) => unknown,
    requestOptions?: WebLockRequestOptions,
  ): Promise<unknown> => {
    const attempt = run(name, callback, requestOptions);
    return requestOptions?.ifAvailable
      ? attempt
      : attempt.then((outcome) => (outcome.acquired ? outcome.value : undefined));
  }) as WebLockRequest;

  const query = async (): Promise<WebLocksSnapshot> => {
    const locks = resolveLocks();
    if (!locks) return { held: [], pending: [] };
    try {
      const snapshot = await locks.query();
      return { held: snapshot.held ?? [], pending: snapshot.pending ?? [] };
    } catch (cause) {
      error.value = cause;
      return { held: [], pending: [] };
    }
  };

  tryOnScopeDispose(() => {
    for (const controller of controllers) {
      controller.abort(new DOMException("The owning scope was disposed.", "AbortError"));
    }
    controllers.clear();
  });

  return {
    supported: computed(() => resolveLocks() !== undefined),
    held: readonly(held),
    pending: readonly(pending),
    error: readonly(error),
    request,
    query,
  };
}
