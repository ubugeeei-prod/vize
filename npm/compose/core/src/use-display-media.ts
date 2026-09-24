import { toValue } from "vue";
import type { MaybeRefOrGetter } from "vue";

import { useMediaStream } from "./use-user-media.ts";
import type { MediaStreamControls, MediaStreamLike } from "./use-user-media.ts";

/** Minimal `MediaDevices` consumed by {@link useDisplayMedia}. */
export interface DisplayMediaHost {
  /** Ask the user to pick a screen, window, or tab to capture. */
  getDisplayMedia(options?: DisplayMediaStreamOptions): Promise<MediaStreamLike>;
}

/** Options for {@link useDisplayMedia}. */
export interface UseDisplayMediaOptions {
  /**
   * Video capture constraints.
   *
   * @default true
   */
  readonly video?: MaybeRefOrGetter<boolean | MediaTrackConstraints>;

  /**
   * Audio capture constraints (tab or system audio where supported).
   *
   * @default false
   */
  readonly audio?: MaybeRefOrGetter<boolean | MediaTrackConstraints>;

  /**
   * Start while `true`, stop while `false`. Browsers require a user gesture
   * for screen capture, so prefer calling `start` from an event handler.
   *
   * @default false
   */
  readonly enabled?: MaybeRefOrGetter<boolean>;

  /**
   * `MediaDevices` capability for alternate runtimes and tests.
   *
   * @default window.navigator.mediaDevices when it implements `getDisplayMedia`
   */
  readonly host?: MaybeRefOrGetter<DisplayMediaHost | null | undefined>;
}

function browserDisplayMediaHost(): DisplayMediaHost | undefined {
  if (typeof window === "undefined") return undefined;
  // `mediaDevices` is undefined in insecure contexts despite its DOM typing.
  const devices: MediaDevices | undefined = window.navigator.mediaDevices;
  return devices && typeof devices.getDisplayMedia === "function" ? devices : undefined;
}

/**
 * Capture a screen, window, or tab with `getDisplayMedia`.
 *
 * Shares the stream lifecycle of `useMediaStream` (from `use-user-media`):
 * classified failures (`"permission-denied"` when the user dismisses the
 * picker), latest-wins requests, and automatic return to `"idle"` when the
 * user ends sharing from the browser UI (every track `ended`). Tracks are
 * stopped when the owning reactive scope stops.
 *
 * Server rendering: nothing is captured; `status` is `"idle"` and
 * `supported` is false.
 *
 * @example
 * ```ts
 * const screen = useDisplayMedia({ audio: true });
 * const share = () => screen.start();
 * ```
 *
 * @param options Capture constraints, activation, and capability.
 * @default options {}
 * @returns Stream state and actions.
 */
export function useDisplayMedia(options: UseDisplayMediaOptions = {}): MediaStreamControls {
  const resolveHost = (): DisplayMediaHost | undefined =>
    options.host === undefined ? browserDisplayMediaHost() : (toValue(options.host) ?? undefined);

  return useMediaStream<DisplayMediaStreamOptions>({
    source: () => {
      const host = resolveHost();
      return host ? { request: (constraints) => host.getDisplayMedia(constraints) } : undefined;
    },
    constraints: () => ({
      video: toValue(options.video) ?? true,
      audio: toValue(options.audio) ?? false,
    }),
    enabled: () => toValue(options.enabled) ?? false,
  });
}
