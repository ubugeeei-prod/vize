import { readonly, ref, toValue, watch } from "vue";
import type { MaybeRefOrGetter, Ref } from "vue";

import { tryOnScopeDispose } from "./scope.ts";

/**
 * Permission names understood by at least one major browser.
 *
 * The `(string & {})` member keeps autocompletion for the well-known names
 * while still accepting names that browsers add later.
 */
export type WellKnownPermissionName =
  | "accelerometer"
  | "background-fetch"
  | "background-sync"
  | "bluetooth"
  | "camera"
  | "clipboard-read"
  | "clipboard-write"
  | "display-capture"
  | "geolocation"
  | "gyroscope"
  | "idle-detection"
  | "local-fonts"
  | "magnetometer"
  | "microphone"
  | "midi"
  | "nfc"
  | "notifications"
  | "payment-handler"
  | "periodic-background-sync"
  | "persistent-storage"
  | "push"
  | "screen-wake-lock"
  | "speaker-selection"
  | "storage-access"
  | "window-management"
  | (string & {});

/** Descriptor passed to `navigator.permissions.query`. */
export interface PermissionDescriptorLike {
  /** Permission name. */
  readonly name: WellKnownPermissionName;

  /** Push: only user-visible notifications (required `true` by most browsers). */
  readonly userVisibleOnly?: boolean;

  /** MIDI: request system-exclusive message access. */
  readonly sysex?: boolean;

  /** Camera: request pan-tilt-zoom control. */
  readonly panTiltZoom?: boolean;

  /** Clipboard (Chromium): allow access without a user gesture. */
  readonly allowWithoutGesture?: boolean;
}

/** Browser permission state, plus the capability states added by this package. */
export type PermissionQueryState = "granted" | "denied" | "prompt" | "unsupported" | "unknown";

/** Minimal `PermissionStatus` shape consumed by {@link usePermission}. */
export interface PermissionStatusLike extends EventTarget {
  /** Current state reported by the browser. */
  readonly state: string;
}

/** Minimal `Permissions` shape consumed by {@link usePermission}. */
export interface PermissionsHost {
  /**
   * Query one permission. Rejects (TypeError) for names the browser does not
   * know. Declared with method syntax so the DOM `Permissions` object, whose
   * descriptor names are a narrower union, satisfies this interface.
   */
  query(descriptor: PermissionDescriptorLike): Promise<PermissionStatusLike>;
}

/** Options for {@link usePermission}. */
export interface UsePermissionOptions {
  /**
   * Permissions capability for alternate runtimes and tests.
   *
   * @default window.navigator.permissions when a browser window exists
   */
  readonly host?: MaybeRefOrGetter<PermissionsHost | null | undefined>;

  /**
   * State exposed before the first query settles and during server rendering.
   *
   * @default "unknown"
   */
  readonly initialState?: PermissionQueryState;
}

/** Reactive state returned by {@link usePermission}. */
export interface PermissionControls {
  /**
   * Current permission state. `"unsupported"` means the Permissions API (or
   * this permission name) is unavailable; `"unknown"` means no answer yet.
   */
  readonly state: Readonly<Ref<PermissionQueryState>>;

  /** Whether a Permissions capability is attached. */
  readonly supported: Readonly<Ref<boolean>>;

  /**
   * Query the permission again and resubscribe to its changes.
   *
   * @returns The settled state.
   */
  readonly query: () => Promise<PermissionQueryState>;
}

function normalizeState(state: string): PermissionQueryState {
  return state === "granted" || state === "denied" || state === "prompt" ? state : "unknown";
}

function browserPermissions(): PermissionsHost | undefined {
  if (typeof window === "undefined") return undefined;
  return window.navigator.permissions ?? undefined;
}

function toDescriptor(
  name: WellKnownPermissionName | PermissionDescriptorLike,
): PermissionDescriptorLike {
  return typeof name === "string" ? { name } : name;
}

/**
 * Observe a browser permission reactively.
 *
 * The permission is queried when the composable is created and whenever
 * the reactive name/descriptor or host changes; the `change` event of the
 * returned `PermissionStatus` keeps `state` current (for example after the
 * user revokes camera access in the site settings). The subscription is
 * removed when the name changes and when the owning reactive scope stops.
 *
 * Server rendering: no browser window exists, so nothing is queried and
 * `state` stays at `initialState` (`"unknown"`) with `supported` false.
 * Unknown names (the browser rejects the query) report `"unsupported"`.
 *
 * @example
 * ```ts
 * const { state } = usePermission("camera");
 * const canUseCamera = computed(() => state.value === "granted");
 * ```
 *
 * @param name Reactive permission name or full descriptor.
 * @param options Capability host and initial state.
 * @default options {}
 * @returns The reactive permission state and a re-query control.
 */
export function usePermission(
  name: MaybeRefOrGetter<WellKnownPermissionName | PermissionDescriptorLike>,
  options: UsePermissionOptions = {},
): PermissionControls {
  const initialState = options.initialState ?? "unknown";
  const state = ref<PermissionQueryState>(initialState);
  const supported = ref(false);
  let generation = 0;
  let unsubscribe: (() => void) | undefined;
  let active = true;

  const resolveHost = (): PermissionsHost | undefined =>
    options.host === undefined ? browserPermissions() : (toValue(options.host) ?? undefined);

  const release = (): void => {
    unsubscribe?.();
    unsubscribe = undefined;
  };

  const query = async (): Promise<PermissionQueryState> => {
    const current = ++generation;
    release();
    const host = resolveHost();
    supported.value = host !== undefined;
    if (!host) {
      // Without a browser window (server rendering) the answer is unknown,
      // not "unsupported": keep the initial state so hydration matches.
      state.value = typeof window === "undefined" ? initialState : "unsupported";
      return state.value;
    }
    let status: PermissionStatusLike;
    try {
      status = await host.query(toDescriptor(toValue(name)));
    } catch {
      if (current === generation) state.value = "unsupported";
      return "unsupported";
    }
    if (current !== generation || !active) return normalizeState(status.state);
    const update = (): void => {
      state.value = normalizeState(status.state);
    };
    update();
    status.addEventListener("change", update);
    unsubscribe = () => status.removeEventListener("change", update);
    return state.value;
  };

  if (typeof window !== "undefined" || options.host !== undefined) {
    watch(
      [() => toValue(name), resolveHost],
      () => {
        void query();
      },
      { immediate: true, flush: "sync" },
    );
  }

  tryOnScopeDispose(() => {
    active = false;
    generation += 1;
    release();
  });

  return { state: readonly(state), supported: readonly(supported), query };
}
