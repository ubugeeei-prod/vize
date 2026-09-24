import { computed, readonly, ref, shallowRef, toValue, watch } from "vue";
import type { ComputedRef, MaybeRefOrGetter, Ref, ShallowRef } from "vue";

import { tryOnScopeDispose } from "./scope.ts";

/** Minimal `MediaStreamTrack`. */
export interface MediaStreamTrackLike extends EventTarget {
  /** `"audio"` or `"video"`. */
  readonly kind: string;

  /** `"live"` or `"ended"`. */
  readonly readyState: string;

  /** Stop the track and release its device. */
  stop(): void;
}

/** Minimal `MediaStream`. */
export interface MediaStreamLike {
  /** Every track of the stream. */
  getTracks(): readonly MediaStreamTrackLike[];
}

/** Anything that can asynchronously produce a media stream. */
export interface MediaStreamSource<Constraints> {
  /** Acquire a stream; rejects with a DOMException on failure. */
  request(constraints: Constraints): Promise<MediaStreamLike>;
}

/** Lifecycle state of a managed media stream. */
export type MediaStreamStatus = "idle" | "requesting" | "active" | "error";

/** Normalized reason why a stream could not be acquired. */
export type MediaStreamErrorCode =
  | "aborted"
  | "failed"
  | "invalid-constraints"
  | "not-found"
  | "not-readable"
  | "overconstrained"
  | "permission-denied"
  | "unsupported";

/** Stream acquisition failure. */
export interface MediaStreamFailure {
  /** Normalized reason. */
  readonly code: MediaStreamErrorCode;

  /** Exact error thrown by the source, when one was thrown. */
  readonly cause: unknown;
}

/** Options for {@link useMediaStream}. */
export interface UseMediaStreamOptions<Constraints> {
  /** Reactive stream source; `null`/`undefined` means unsupported. */
  readonly source: MaybeRefOrGetter<MediaStreamSource<Constraints> | null | undefined>;

  /** Reactive constraints passed to the source. */
  readonly constraints: MaybeRefOrGetter<Constraints>;

  /**
   * Start while `true`, stop while `false`.
   *
   * @default false
   */
  readonly enabled?: MaybeRefOrGetter<boolean>;

  /**
   * Restart an active stream when the constraints change.
   *
   * @default true
   */
  readonly autoSwitch?: boolean;
}

/** Reactive state and actions of a managed media stream. */
export interface MediaStreamControls {
  /** Whether a stream source is available. */
  readonly supported: ComputedRef<boolean>;

  /** The active stream, if any. */
  readonly stream: Readonly<ShallowRef<MediaStreamLike | undefined>>;

  /** Lifecycle state. */
  readonly status: Readonly<Ref<MediaStreamStatus>>;

  /** Most recent acquisition failure, cleared by the next request. */
  readonly error: Readonly<ShallowRef<MediaStreamFailure | undefined>>;

  /**
   * Acquire a stream unless one is active. Overlapping calls resolve to the
   * newest stream; superseded streams are stopped immediately.
   *
   * @returns The active stream, or `undefined` on failure or supersession.
   */
  readonly start: () => Promise<MediaStreamLike | undefined>;

  /** Stop every track and release the stream. Repeated calls are safe. */
  readonly stop: () => void;

  /**
   * Stop and acquire a fresh stream with the current constraints.
   *
   * @returns The new stream, or `undefined` on failure or supersession.
   */
  readonly restart: () => Promise<MediaStreamLike | undefined>;
}

function errorName(error: unknown): string | undefined {
  return typeof error === "object" && error !== null && "name" in error
    ? String(error.name)
    : undefined;
}

function classify(error: unknown): MediaStreamErrorCode {
  switch (errorName(error)) {
    case "NotAllowedError":
    case "SecurityError":
      return "permission-denied";
    case "NotFoundError":
      return "not-found";
    case "NotReadableError":
      return "not-readable";
    case "OverconstrainedError":
      return "overconstrained";
    case "AbortError":
      return "aborted";
    case "TypeError":
      return "invalid-constraints";
    default:
      return "failed";
  }
}

function stopTracks(stream: MediaStreamLike): void {
  for (const track of stream.getTracks()) track.stop();
}

/**
 * Manage the lifecycle of a media stream from any asynchronous source.
 *
 * This is the engine behind {@link useUserMedia} and `useDisplayMedia`, and
 * can wrap any other stream producer (canvas capture, WebRTC). Acquisition
 * failures are classified into {@link MediaStreamErrorCode}s; the stream
 * returns to `"idle"` once all of its tracks ended (device unplugged, the
 * user stopped screen sharing). Latest-wins: a stream that resolves after a
 * newer request or a `stop` is stopped immediately. All tracks are stopped
 * when the owning reactive scope stops.
 *
 * Server rendering: without a browser window a missing source keeps the
 * status `"idle"` (no error), so server markup matches the first client
 * render.
 *
 * @example
 * ```ts
 * const canvasStream = useMediaStream({
 *   source: { request: async (fps: number) => canvas.captureStream(fps) },
 *   constraints: 30,
 * });
 * ```
 *
 * @typeParam Constraints Constraint type accepted by the source.
 * @param options Source, constraints, and activation.
 * @returns Stream state and actions.
 */
export function useMediaStream<Constraints>(
  options: UseMediaStreamOptions<Constraints>,
): MediaStreamControls {
  const stream = shallowRef<MediaStreamLike | undefined>(undefined);
  const status = ref<MediaStreamStatus>("idle");
  const error = shallowRef<MediaStreamFailure | undefined>(undefined);
  let generation = 0;
  let detach: (() => void) | undefined;

  const resolveSource = (): MediaStreamSource<Constraints> | undefined =>
    toValue(options.source) ?? undefined;

  const stop = (): void => {
    generation += 1;
    detach?.();
    detach = undefined;
    const current = stream.value;
    stream.value = undefined;
    if (current) stopTracks(current);
    status.value = "idle";
  };

  const attach = (next: MediaStreamLike): void => {
    const tracks = next.getTracks();
    const onEnded = (): void => {
      if (stream.value === next && tracks.every((track) => track.readyState === "ended")) stop();
    };
    for (const track of tracks) track.addEventListener("ended", onEnded);
    detach = () => {
      for (const track of tracks) track.removeEventListener("ended", onEnded);
    };
  };

  const start = async (): Promise<MediaStreamLike | undefined> => {
    if (status.value === "active" && stream.value) return stream.value;
    const current = ++generation;
    const source = resolveSource();
    if (!source) {
      if (typeof window !== "undefined") {
        error.value = { code: "unsupported", cause: undefined };
        status.value = "error";
      }
      return undefined;
    }
    error.value = undefined;
    status.value = "requesting";
    let next: MediaStreamLike;
    try {
      next = await source.request(toValue(options.constraints));
    } catch (cause) {
      if (current === generation) {
        error.value = { code: classify(cause), cause };
        status.value = "error";
      }
      return undefined;
    }
    if (current !== generation) {
      stopTracks(next);
      return undefined;
    }
    detach?.();
    const previous = stream.value;
    if (previous && previous !== next) stopTracks(previous);
    stream.value = next;
    attach(next);
    status.value = "active";
    return next;
  };

  const restart = async (): Promise<MediaStreamLike | undefined> => {
    stop();
    return start();
  };

  watch(
    () => toValue(options.enabled) ?? false,
    (enabled) => {
      if (enabled) void start();
      else if (status.value !== "idle") stop();
    },
    { immediate: true },
  );

  watch(
    () => toValue(options.constraints),
    () => {
      if ((options.autoSwitch ?? true) && status.value === "active") void restart();
    },
    { deep: true },
  );

  tryOnScopeDispose(stop);

  return {
    supported: computed(() => resolveSource() !== undefined),
    stream,
    status: readonly(status),
    error,
    start,
    stop,
    restart,
  };
}

/** Minimal `MediaDeviceInfo`. */
export interface MediaDeviceInfoLike {
  /** Device identifier. */
  readonly deviceId: string;

  /** `"audioinput"`, `"audiooutput"`, or `"videoinput"`. */
  readonly kind: string;

  /** Human-readable label (empty until permission is granted). */
  readonly label: string;

  /** Identifier shared by devices of one physical unit. */
  readonly groupId: string;
}

/** Minimal `MediaDevices` consumed by {@link useUserMedia}. */
export interface UserMediaHost extends EventTarget {
  /** Request camera/microphone access. */
  getUserMedia(constraints?: MediaStreamConstraints): Promise<MediaStreamLike>;

  /** List media devices. */
  enumerateDevices?(): Promise<readonly MediaDeviceInfoLike[]>;
}

/** Options for {@link useUserMedia}. */
export interface UseUserMediaOptions {
  /**
   * Reactive constraints for `getUserMedia`.
   *
   * @default { audio: true, video: true }
   */
  readonly constraints?: MaybeRefOrGetter<MediaStreamConstraints>;

  /**
   * Start while `true`, stop while `false`.
   *
   * @default false
   */
  readonly enabled?: MaybeRefOrGetter<boolean>;

  /**
   * Restart an active stream when the constraints change.
   *
   * @default true
   */
  readonly autoSwitch?: boolean;

  /**
   * Keep `devices` current via `enumerateDevices` and `devicechange`.
   *
   * @default false
   */
  readonly listDevices?: boolean;

  /**
   * `MediaDevices` capability for alternate runtimes and tests.
   *
   * @default window.navigator.mediaDevices
   */
  readonly host?: MaybeRefOrGetter<UserMediaHost | null | undefined>;
}

/** Reactive state and actions returned by {@link useUserMedia}. */
export interface UserMediaControls extends MediaStreamControls {
  /** Media devices, populated when `listDevices` is enabled. */
  readonly devices: Readonly<ShallowRef<readonly MediaDeviceInfoLike[]>>;

  /**
   * Re-enumerate media devices.
   *
   * @returns The device list, empty when unsupported or on failure.
   */
  readonly refreshDevices: () => Promise<readonly MediaDeviceInfoLike[]>;
}

function browserUserMediaHost(): UserMediaHost | undefined {
  if (typeof window === "undefined") return undefined;
  // `mediaDevices` is undefined in insecure contexts despite its DOM typing.
  const devices: MediaDevices | undefined = window.navigator.mediaDevices;
  return devices ?? undefined;
}

/**
 * Access the camera and microphone with `getUserMedia`.
 *
 * Built on {@link useMediaStream}: reactive constraints (restarting an
 * active stream when they change), `enabled` activation, classified
 * failures such as `"permission-denied"`, and latest-wins requests. With
 * `listDevices`, `devices` follows `devicechange`. Tracks and listeners are
 * released when the owning reactive scope stops.
 *
 * Server rendering: no device is touched; `status` is `"idle"`, `stream` is
 * undefined, and `supported` is false.
 *
 * @example
 * ```ts
 * const camera = useUserMedia({ constraints: { video: true, audio: false } });
 * await camera.start();
 * video.srcObject = camera.stream.value;
 * ```
 *
 * @param options Constraints, activation, device listing, and capability.
 * @default options {}
 * @returns Stream state, device list, and actions.
 */
export function useUserMedia(options: UseUserMediaOptions = {}): UserMediaControls {
  const resolveHost = (): UserMediaHost | undefined =>
    options.host === undefined ? browserUserMediaHost() : (toValue(options.host) ?? undefined);
  const devices = shallowRef<readonly MediaDeviceInfoLike[]>([]);

  const controls = useMediaStream<MediaStreamConstraints>({
    source: () => {
      const host = resolveHost();
      return host ? { request: (constraints) => host.getUserMedia(constraints) } : undefined;
    },
    constraints: () => toValue(options.constraints) ?? { audio: true, video: true },
    enabled: () => toValue(options.enabled) ?? false,
    autoSwitch: options.autoSwitch ?? true,
  });

  const refreshDevices = async (): Promise<readonly MediaDeviceInfoLike[]> => {
    const host = resolveHost();
    if (!host?.enumerateDevices) return (devices.value = []);
    try {
      devices.value = [...(await host.enumerateDevices())];
    } catch {
      devices.value = [];
    }
    return devices.value;
  };

  if (options.listDevices ?? false) {
    watch(
      resolveHost,
      (host, _previous, onCleanup) => {
        if (!host) return;
        const update = (): void => {
          void refreshDevices();
        };
        update();
        host.addEventListener("devicechange", update);
        onCleanup(() => host.removeEventListener("devicechange", update));
      },
      { immediate: true, flush: "sync" },
    );
    // Labels become available once a stream was granted.
    watch(controls.status, (status) => {
      if (status === "active") void refreshDevices();
    });
  }

  return { ...controls, devices, refreshDevices };
}
