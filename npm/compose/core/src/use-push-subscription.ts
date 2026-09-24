import { computed, readonly, shallowReadonly, shallowRef, toValue, watch } from "vue";
import type { ComputedRef, MaybeRefOrGetter, ShallowRef } from "vue";

import { tryOnScopeDispose } from "./scope.ts";

/** Push permission states. */
export type PushPermissionState = "granted" | "denied" | "prompt";

/** JSON form of a push subscription (`PushSubscription.toJSON()`). */
export interface PushSubscriptionJsonLike {
  /** Push service endpoint. */
  readonly endpoint?: string | undefined;

  /** Expiration time in epoch milliseconds, if any. */
  readonly expirationTime?: number | null | undefined;

  /** Base64url-encoded keys (`p256dh`, `auth`). */
  readonly keys?: Readonly<Record<string, string>> | undefined;
}

/** Minimal `PushSubscription` consumed by {@link usePushSubscription}. */
export interface PushSubscriptionLike {
  /** Push service endpoint. */
  readonly endpoint: string;

  /** Expiration time in epoch milliseconds, if any. */
  readonly expirationTime: number | null;

  /** Serialize for a server. */
  toJSON(): PushSubscriptionJsonLike;

  /** Cancel the subscription. */
  unsubscribe(): Promise<boolean>;
}

/** Options forwarded to `PushManager.subscribe`. */
export interface PushManagerSubscribeOptions {
  /** VAPID public key bytes. */
  readonly applicationServerKey: BufferSource;

  /** Only user-visible notifications. */
  readonly userVisibleOnly: boolean;
}

/** Minimal `PushManager` consumed by {@link usePushSubscription}. */
export interface PushManagerLike {
  /** Create (or return the existing) subscription. */
  subscribe(options: PushManagerSubscribeOptions): Promise<PushSubscriptionLike>;

  /** Current subscription, if any. */
  getSubscription(): Promise<PushSubscriptionLike | null>;

  /** Permission state for push. */
  permissionState(options?: { readonly userVisibleOnly?: boolean }): Promise<PushPermissionState>;
}

/** Registration-like owner of a push manager. */
export interface PushRegistrationLike {
  /** The registration's push manager. */
  readonly pushManager: PushManagerLike;
}

/** Options for {@link usePushSubscription}. */
export interface UsePushSubscriptionOptions {
  /**
   * Service worker registration owning the push manager. `null` while it is
   * not known yet.
   *
   * @default window.navigator.serviceWorker.ready when push is available
   */
  readonly registration?: MaybeRefOrGetter<PushRegistrationLike | null | undefined>;

  /**
   * Load the existing subscription as soon as a registration is known.
   *
   * @default true
   */
  readonly immediate?: boolean;
}

/** Arguments of {@link PushSubscriptionControls.subscribe}. */
export interface PushSubscribeOptions {
  /** VAPID public key as a base64url string or raw bytes. */
  readonly applicationServerKey: string | BufferSource;

  /**
   * Only user-visible notifications (required by most browsers).
   *
   * @default true
   */
  readonly userVisibleOnly?: boolean;
}

/** Serializable snapshot of the current subscription. */
export interface PushSubscriptionSnapshot {
  /** Push service endpoint. */
  readonly endpoint: string;

  /** Expiration time in epoch milliseconds, or `null`. */
  readonly expirationTime: number | null;

  /** Base64url-encoded keys (`p256dh`, `auth`). */
  readonly keys: Readonly<Record<string, string>>;
}

/** Reactive state and actions returned by {@link usePushSubscription}. */
export interface PushSubscriptionControls {
  /** Whether the Push API is available. */
  readonly supported: ComputedRef<boolean>;

  /** Current subscription, or `null`. */
  readonly subscription: Readonly<ShallowRef<PushSubscriptionLike | null>>;

  /** JSON snapshot of the current subscription, or `null`. */
  readonly json: ComputedRef<PushSubscriptionSnapshot | null>;

  /** Most recent failure, cleared by the next successful action. */
  readonly error: Readonly<ShallowRef<unknown>>;

  /**
   * Subscribe to push messages. Rejects with a tagged `TypeError` when the
   * key string is not valid base64url.
   *
   * @param options VAPID key and visibility flag.
   * @returns The subscription, or `null` when unsupported or on failure.
   */
  readonly subscribe: (options: PushSubscribeOptions) => Promise<PushSubscriptionLike | null>;

  /**
   * Cancel the current subscription.
   *
   * @returns Whether a subscription was cancelled.
   */
  readonly unsubscribe: () => Promise<boolean>;

  /**
   * Re-read the existing subscription.
   *
   * @returns The subscription, or `null`.
   */
  readonly refresh: () => Promise<PushSubscriptionLike | null>;

  /**
   * Query the push permission state.
   *
   * @param userVisibleOnly Query for user-visible pushes.
   * @default userVisibleOnly true
   * @returns The permission state, or `"unsupported"`.
   */
  readonly permissionState: (
    userVisibleOnly?: boolean,
  ) => Promise<PushPermissionState | "unsupported">;
}

const base64UrlAlphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

function decodeApplicationServerKey(text: string): Uint8Array<ArrayBuffer> {
  const clean = text.trim().replace(/=+$/u, "").replaceAll("+", "-").replaceAll("/", "_");
  const invalid = (): TypeError =>
    new TypeError(
      "[VIZE_COMPOSE_PUSH_INVALID_KEY] applicationServerKey must be a base64url string.",
    );
  if (clean.length % 4 === 1) throw invalid();
  const bytes = new Uint8Array(Math.floor((clean.length * 3) / 4));
  let buffer = 0;
  let bits = 0;
  let offset = 0;
  for (const character of clean) {
    const value = base64UrlAlphabet.indexOf(character);
    if (value === -1) throw invalid();
    buffer = ((buffer << 6) | value) & 0xffffff;
    bits += 6;
    if (bits >= 8) {
      bits -= 8;
      bytes[offset] = (buffer >> bits) & 0xff;
      offset += 1;
    }
  }
  return bytes;
}

interface ReadyContainer {
  readonly ready: Promise<PushRegistrationLike>;
}

function browserContainer(): ReadyContainer | undefined {
  if (typeof window === "undefined" || !("PushManager" in window)) return undefined;
  const { navigator } = window;
  return "serviceWorker" in navigator && navigator.serviceWorker
    ? navigator.serviceWorker
    : undefined;
}

/**
 * Manage a Web Push subscription.
 *
 * Uses the push manager of `options.registration`, or of
 * `navigator.serviceWorker.ready` by default. The existing subscription is
 * loaded when the registration becomes known; `subscribe` accepts the VAPID
 * key as a base64url string. `json` is the snapshot to send to a push server.
 * Nothing needs cleanup; pending results are ignored after the owning
 * reactive scope stops.
 *
 * Server rendering: nothing is read, `supported` is false and the
 * subscription is `null`.
 *
 * @example
 * ```ts
 * const push = usePushSubscription();
 * const enable = async () => {
 *   await push.subscribe({ applicationServerKey: VAPID_PUBLIC_KEY });
 *   await fetch("/api/push", { method: "POST", body: JSON.stringify(push.json.value) });
 * };
 * ```
 *
 * @param options Registration override and initial read policy.
 * @default options {}
 * @returns Subscription state and actions.
 */
export function usePushSubscription(
  options: UsePushSubscriptionOptions = {},
): PushSubscriptionControls {
  const subscription = shallowRef<PushSubscriptionLike | null>(null);
  const error = shallowRef<unknown>(undefined);
  let active = true;

  const source = (): PushRegistrationLike | ReadyContainer | undefined =>
    options.registration === undefined
      ? browserContainer()
      : (toValue(options.registration) ?? undefined);

  const resolveManager = async (): Promise<PushManagerLike | undefined> => {
    const current = source();
    if (!current) return undefined;
    return "pushManager" in current ? current.pushManager : (await current.ready).pushManager;
  };

  const attempt = async <Value>(
    fallback: Value,
    action: (manager: PushManagerLike) => Promise<Value>,
  ): Promise<Value> => {
    try {
      const manager = await resolveManager();
      if (!manager) return fallback;
      const value = await action(manager);
      if (active) error.value = undefined;
      return value;
    } catch (cause) {
      if (active) error.value = cause;
      return fallback;
    }
  };

  const refresh = (): Promise<PushSubscriptionLike | null> =>
    attempt(null, async (manager) => {
      const current = await manager.getSubscription();
      if (active) subscription.value = current;
      return current;
    });

  const subscribe = async ({
    applicationServerKey,
    userVisibleOnly = true,
  }: PushSubscribeOptions): Promise<PushSubscriptionLike | null> => {
    const key =
      typeof applicationServerKey === "string"
        ? decodeApplicationServerKey(applicationServerKey)
        : applicationServerKey;
    return attempt(null, async (manager) => {
      const next = await manager.subscribe({ applicationServerKey: key, userVisibleOnly });
      if (active) subscription.value = next;
      return next;
    });
  };

  const unsubscribe = async (): Promise<boolean> => {
    const current = subscription.value;
    if (!current) return false;
    try {
      const removed = await current.unsubscribe();
      if (active && removed && subscription.value === current) subscription.value = null;
      if (active) error.value = undefined;
      return removed;
    } catch (cause) {
      if (active) error.value = cause;
      return false;
    }
  };

  const permissionState = (userVisibleOnly = true): Promise<PushPermissionState | "unsupported"> =>
    attempt<PushPermissionState | "unsupported">("unsupported", (manager) =>
      manager.permissionState({ userVisibleOnly }),
    );

  if (options.immediate ?? true) {
    watch(
      source,
      (current) => {
        if (current) void refresh();
        else subscription.value = null;
      },
      { immediate: true },
    );
  }

  tryOnScopeDispose(() => {
    active = false;
  });

  return {
    supported: computed(() => source() !== undefined),
    subscription: shallowReadonly(subscription),
    json: computed(() => {
      const current = subscription.value;
      if (!current) return null;
      const serialized = current.toJSON();
      return {
        endpoint: serialized.endpoint ?? current.endpoint,
        expirationTime: serialized.expirationTime ?? current.expirationTime,
        keys: serialized.keys ?? {},
      };
    }),
    error: readonly(error),
    subscribe,
    unsubscribe,
    refresh,
    permissionState,
  };
}
