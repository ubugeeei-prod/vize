import { computed, readonly, ref, shallowRef, toValue, unref } from "vue";
import type { ComputedRef, MaybeRef, MaybeRefOrGetter, Ref, ShallowRef } from "vue";

import { tryOnScopeDispose } from "./scope.ts";

/** Notification permission, plus `"unsupported"` when the API is missing. */
export type NotificationPermissionState = "default" | "granted" | "denied" | "unsupported";

/** Options accepted by the `Notification` constructor. */
export interface NotificationOptionsLike<Data> {
  /** Body text. */
  readonly body?: string;

  /** Icon URL. */
  readonly icon?: string;

  /** Monochrome badge URL for constrained surfaces. */
  readonly badge?: string;

  /** Grouping tag; a new notification with the same tag replaces the old one. */
  readonly tag?: string;

  /** BCP 47 language tag. */
  readonly lang?: string;

  /** Text direction. */
  readonly dir?: "auto" | "ltr" | "rtl";

  /** Keep the notification visible until the user interacts with it. */
  readonly requireInteraction?: boolean;

  /** Suppress sounds and vibration. */
  readonly silent?: boolean | null;

  /** Application data attached to the notification. */
  readonly data?: Data;
}

/** Minimal `Notification` instance consumed by {@link useWebNotification}. */
export interface NotificationLike extends EventTarget {
  /** Dismiss the notification. */
  close(): void;
}

/** Minimal `Notification` constructor (with its static members). */
export interface NotificationHost {
  /** Current permission. */
  readonly permission: string;

  /** Ask the user for permission. */
  requestPermission(): Promise<string>;

  /** Show a notification. */
  new (title: string, options?: NotificationOptionsLike<unknown>): NotificationLike;
}

/** Default notification content for {@link useWebNotification}. */
export interface WebNotificationDefaults<Data> extends NotificationOptionsLike<Data> {
  /** Default title. */
  readonly title?: string;
}

/** Options for {@link useWebNotification}. */
export interface UseWebNotificationOptions<Data> {
  /**
   * Notification constructor for alternate runtimes and tests. A ref (not a
   * getter) because the host itself is a constructor function.
   *
   * @default window.Notification when it exists
   */
  readonly host?: MaybeRef<NotificationHost | null | undefined>;

  /**
   * Reactive default title and options merged into every `show` call.
   *
   * @default {}
   */
  readonly defaults?: MaybeRefOrGetter<WebNotificationDefaults<Data>>;

  /**
   * Request permission from `show` when it has not been decided yet.
   *
   * @default true
   */
  readonly requestPermissionOnShow?: boolean;
}

/** Discriminated outcome of {@link WebNotificationControls.show}. */
export type WebNotificationResult =
  | {
      /** The notification is showing. */
      readonly status: "shown";
      /** The shown notification. */
      readonly notification: NotificationLike;
    }
  | {
      /** Permission is not granted, the API is missing, or construction failed. */
      readonly status: "denied" | "unsupported" | "failed";
      /** Exact error thrown by the host, when one was thrown. */
      readonly error: unknown;
    };

/** Notification lifecycle events exposed by {@link useWebNotification}. */
export type WebNotificationEvent = "click" | "show" | "error" | "close";

/** Reactive state and actions returned by {@link useWebNotification}. */
export interface WebNotificationControls<Data> {
  /** Whether the Notifications API is available. */
  readonly supported: ComputedRef<boolean>;

  /** Current permission state. */
  readonly permission: Readonly<Ref<NotificationPermissionState>>;

  /** Most recently shown notification, cleared when it closes. */
  readonly notification: Readonly<ShallowRef<NotificationLike | undefined>>;

  /** Data attached to the current notification. */
  readonly data: Readonly<ShallowRef<Data | undefined>>;

  /**
   * Ask the user for permission.
   *
   * @returns The resulting permission state.
   */
  readonly requestPermission: () => Promise<NotificationPermissionState>;

  /**
   * Show a notification, replacing the current one.
   *
   * @param title Title overriding the default.
   * @param options Options merged over the defaults.
   * @returns The discriminated outcome; never rejects.
   */
  readonly show: (
    title?: string,
    options?: NotificationOptionsLike<Data>,
  ) => Promise<WebNotificationResult>;

  /** Close the current notification. */
  readonly close: () => void;

  /**
   * Subscribe to an event of every notification shown by this composable.
   *
   * @param event Lifecycle event.
   * @param handler Called with the native event and its notification.
   * @returns Unsubscribe function.
   */
  readonly on: (
    event: WebNotificationEvent,
    handler: (event: Event, notification: NotificationLike) => void,
  ) => () => void;
}

function browserNotificationHost(): NotificationHost | undefined {
  if (typeof window === "undefined") return undefined;
  return "Notification" in window ? window.Notification : undefined;
}

function normalizePermission(permission: string): NotificationPermissionState {
  return permission === "granted" || permission === "denied" ? permission : "default";
}

const notificationEvents: readonly WebNotificationEvent[] = ["click", "show", "error", "close"];

/**
 * Show typed Web Notifications with permission handling.
 *
 * `show` merges reactive defaults with per-call options, asks for
 * permission first when it is still undecided (configurable), and resolves
 * to a discriminated {@link WebNotificationResult}. Handlers registered with
 * `on` apply to every notification this composable shows. The current
 * notification is closed when the owning reactive scope stops.
 *
 * Server rendering: the API is never touched; `permission` is
 * `"unsupported"` and `supported` is false.
 *
 * @example
 * ```ts
 * const notify = useWebNotification<{ id: string }>({ defaults: { icon: "/icon.png" } });
 * notify.on("click", () => window.focus());
 * await notify.show("Build finished", { data: { id: "42" } });
 * ```
 *
 * @typeParam Data Application data attached to notifications.
 * @param options Capability, defaults, and permission policy.
 * @default options {}
 * @returns Notification state and actions.
 */
export function useWebNotification<Data = unknown>(
  options: UseWebNotificationOptions<Data> = {},
): WebNotificationControls<Data> {
  const resolveHost = (): NotificationHost | undefined =>
    options.host === undefined ? browserNotificationHost() : (unref(options.host) ?? undefined);
  const readPermission = (): NotificationPermissionState => {
    const host = resolveHost();
    return host ? normalizePermission(host.permission) : "unsupported";
  };
  const permission = ref<NotificationPermissionState>(readPermission());
  const notification = shallowRef<NotificationLike | undefined>(undefined);
  const data = shallowRef<Data | undefined>(undefined);
  const handlers = new Map<
    WebNotificationEvent,
    Set<(event: Event, notification: NotificationLike) => void>
  >();

  const requestPermission = async (): Promise<NotificationPermissionState> => {
    const host = resolveHost();
    if (!host) return (permission.value = "unsupported");
    try {
      permission.value = normalizePermission(await host.requestPermission());
    } catch {
      permission.value = readPermission();
    }
    return permission.value;
  };

  const close = (): void => {
    notification.value?.close();
    notification.value = undefined;
    data.value = undefined;
  };

  const show = async (
    title?: string,
    overrides?: NotificationOptionsLike<Data>,
  ): Promise<WebNotificationResult> => {
    const host = resolveHost();
    if (!host) return { status: "unsupported", error: undefined };
    permission.value = readPermission();
    if (permission.value === "default" && (options.requestPermissionOnShow ?? true)) {
      await requestPermission();
    }
    if (permission.value !== "granted") return { status: "denied", error: undefined };
    const { title: defaultTitle = "", ...defaults } = toValue(options.defaults) ?? {};
    const merged: NotificationOptionsLike<Data> = { ...defaults, ...overrides };
    let created: NotificationLike;
    try {
      created = new host(title ?? defaultTitle, merged);
    } catch (error) {
      return { status: "failed", error };
    }
    close();
    notification.value = created;
    data.value = merged.data;
    for (const event of notificationEvents) {
      created.addEventListener(event, (nativeEvent) => {
        for (const handler of handlers.get(event) ?? []) handler(nativeEvent, created);
        if (event === "close" && notification.value === created) {
          notification.value = undefined;
          data.value = undefined;
        }
      });
    }
    return { status: "shown", notification: created };
  };

  const on = (
    event: WebNotificationEvent,
    handler: (event: Event, notification: NotificationLike) => void,
  ): (() => void) => {
    let set = handlers.get(event);
    if (!set) handlers.set(event, (set = new Set()));
    set.add(handler);
    return () => {
      set.delete(handler);
    };
  };

  tryOnScopeDispose(() => {
    close();
    handlers.clear();
  });

  return {
    supported: computed(() => resolveHost() !== undefined),
    permission: readonly(permission),
    notification,
    data,
    requestPermission,
    show,
    close,
    on,
  };
}
