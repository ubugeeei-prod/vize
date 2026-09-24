<script setup lang="ts">
import {
  computed,
  onBeforeUnmount,
  onMounted,
  shallowRef,
  toRaw,
  useTemplateRef,
  watch,
} from "vue";
import type { ComputedRef } from "vue";

import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import { useDeterministicId } from "../../foundations/id/deterministic-id.ts";
import { webcamCaptureContext } from "./webcam-capture-context.ts";
import type { WebcamCaptureContextValue } from "./webcam-capture-context.ts";
import { captureFrame } from "./webcam-capture-frame.ts";
import {
  createWebcamConstraints,
  normalizeUserMediaError,
  resolveWebcamCaptureMessages,
  stopWebcamStream,
  toWebcamDevices,
} from "./webcam-capture-media.ts";
import type {
  WebcamCaptureDevice,
  WebcamCaptureFailure,
  WebcamCaptureImageType,
  WebcamCaptureMessages,
  WebcamCapturePhotoResult,
  WebcamCaptureRootExpose,
  WebcamCaptureSlotState,
  WebcamCaptureStatus,
  WebcamFacingMode,
  WebcamMediaHost,
  WebcamMediaStreamLike,
} from "./webcam-capture-types.ts";

const {
  id = undefined,
  stream = undefined,
  constraints = undefined,
  audio = false,
  facingMode = undefined,
  defaultFacingMode = "user",
  deviceId = undefined,
  defaultDeviceId = undefined,
  autoStart = false,
  mirrored = undefined,
  host = undefined,
  captureType = "image/png",
  captureQuality = undefined,
  aspectRatio = undefined,
  maxWidth = undefined,
  countdownInterval = 1000,
  messages = undefined,
} = defineProps<{
  /**
   * Consumer-owned root id. `null` and `undefined` select a deterministic fallback.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /**
   * Consumer-owned stream, e.g. `useUserMedia().stream.value`. Any value other than
   * `undefined` (including `null`) selects external mode: the root never acquires
   * or stops tracks itself.
   *
   * @default undefined
   */
  readonly stream?: WebcamMediaStreamLike | null;

  /**
   * Extra video track constraints for owned acquisition (resolution, frame rate…).
   *
   * @default undefined
   */
  readonly constraints?: MediaTrackConstraints;

  /**
   * Also request the microphone for owned acquisition.
   *
   * @default false
   */
  readonly audio?: boolean;

  /**
   * Controlled facing direction (`v-model:facingMode`). `undefined` selects uncontrolled use.
   *
   * @default undefined
   */
  readonly facingMode?: WebcamFacingMode;

  /**
   * Initial facing direction for uncontrolled use.
   *
   * @default "user"
   */
  readonly defaultFacingMode?: WebcamFacingMode;

  /**
   * Controlled camera (`v-model:deviceId`). `null` clears the selection; `undefined`
   * selects uncontrolled use.
   *
   * @default undefined
   */
  readonly deviceId?: string | null;

  /**
   * Initial camera for uncontrolled use.
   *
   * @default undefined
   */
  readonly defaultDeviceId?: string;

  /**
   * Request the camera on mount. Otherwise permission is only requested by `start()`.
   *
   * @default false
   */
  readonly autoStart?: boolean;

  /**
   * Mirror the preview and captures. `undefined` mirrors the `user` camera only.
   *
   * @default undefined
   */
  readonly mirrored?: boolean;

  /**
   * `MediaDevices` capability for alternate runtimes and tests.
   *
   * @default navigator.mediaDevices
   */
  readonly host?: WebcamMediaHost | null;

  /**
   * Encoded capture type.
   *
   * @default "image/png"
   */
  readonly captureType?: WebcamCaptureImageType;

  /**
   * Encoder quality for lossy capture types, from `0` to `1`.
   *
   * @default undefined
   */
  readonly captureQuality?: number;

  /**
   * Center-crop captures to this width / height ratio.
   *
   * @default undefined
   */
  readonly aspectRatio?: number;

  /**
   * Downscale captures to at most this width in pixels.
   *
   * @default undefined
   */
  readonly maxWidth?: number;

  /**
   * Milliseconds per shutter countdown step.
   *
   * @default 1000
   */
  readonly countdownInterval?: number;

  /**
   * Localized labels and announcements. Omitted entries use English defaults.
   *
   * @default undefined
   */
  readonly messages?: Partial<WebcamCaptureMessages>;
}>();

const emit = defineEmits<{
  /** Fired when the facing direction requests a new controlled value. */
  "update:facingMode": [facingMode: WebcamFacingMode];

  /** Fired when the selected camera requests a new controlled value. */
  "update:deviceId": [deviceId: string | null];

  /** Fired after every distinct status transition. */
  statusChange: [status: WebcamCaptureStatus, previous: WebcamCaptureStatus];

  /** Fired when owned acquisition fails. */
  error: [failure: WebcamCaptureFailure];

  /** Fired when an owned stream starts or stops. */
  streamChange: [stream: WebcamMediaStreamLike | undefined];

  /** Fired after a photo is captured. */
  capture: [photo: WebcamCapturePhotoResult];

  /** Fired when a capture fails to draw or encode. */
  captureError: [error: unknown];
}>();

defineSlots<{
  /** Video, controls, photo, and status parts. Receives the camera state. */
  default(props: WebcamCaptureSlotState): unknown;
}>();

const element = useTemplateRef<HTMLDivElement>("element");
const baseId = useDeterministicId({ id: () => id, hint: "webcam-capture" });
const resolvedMessages = computed(() => resolveWebcamCaptureMessages(messages));
const facingState = useControllableState<WebcamFacingMode>({
  value: () => facingMode,
  defaultValue: () => defaultFacingMode,
});
const deviceState = useControllableState<string | null>({
  value: () => deviceId,
  defaultValue: () => defaultDeviceId ?? null,
});
const ownStream = shallowRef<WebcamMediaStreamLike | undefined>(undefined);
const ownStatus = shallowRef<WebcamCaptureStatus>("idle");
const failure = shallowRef<WebcamCaptureFailure | undefined>(undefined);
const devices = shallowRef<readonly WebcamCaptureDevice[]>(Object.freeze([]));
const countdown = shallowRef(0);
const capturing = shallowRef(false);
const photo = shallowRef<WebcamCapturePhotoResult | null>(null);
const photoUrl = shallowRef<string | undefined>(undefined);
const announcement = shallowRef("");
const videoElement = shallowRef<HTMLVideoElement | null>(null);
let generation = 0;
let countdownToken = 0;
let detachTracks: (() => void) | undefined;
let listenedHost: WebcamMediaHost | undefined;

const external = computed(() => stream !== undefined);
const currentStream = computed<WebcamMediaStreamLike | undefined>(() =>
  external.value ? (stream ?? undefined) : ownStream.value,
);
const status = computed<WebcamCaptureStatus>(() => {
  if (external.value) return stream ? "active" : "idle";
  return ownStatus.value;
});
const selectedDevice = computed(() => deviceState.value.value ?? undefined);
const facingModeValue = computed(() => facingState.value.value);
const mirroredState = computed(() => mirrored ?? facingState.value.value === "user");
const slotState = computed<WebcamCaptureSlotState>(() => ({
  capturing: capturing.value,
  countdown: countdown.value,
  deviceId: selectedDevice.value,
  devices: devices.value,
  error: external.value ? undefined : failure.value,
  external: external.value,
  facingMode: facingState.value.value,
  mirrored: mirroredState.value,
  status: status.value,
}));

function resolveHost(): WebcamMediaHost | undefined {
  if (host !== undefined) return host ?? undefined;
  if (typeof navigator === "undefined" || navigator.mediaDevices === undefined) return undefined;
  return navigator.mediaDevices;
}

function fail(next: WebcamCaptureFailure): void {
  failure.value = next;
  ownStatus.value = "error";
  emit("error", next);
}

function releaseOwnStream(): void {
  detachTracks?.();
  detachTracks = undefined;
  if (ownStream.value === undefined) return;
  stopWebcamStream(ownStream.value);
  ownStream.value = undefined;
  emit("streamChange", undefined);
}

function attach(next: WebcamMediaStreamLike): void {
  const tracks = next.getTracks();
  const onEnded = (): void => {
    if (ownStream.value === next && tracks.every((track) => track.readyState === "ended")) stop();
  };
  for (const track of tracks) track.addEventListener("ended", onEnded);
  detachTracks = () => {
    for (const track of tracks) track.removeEventListener("ended", onEnded);
  };
}

async function refreshDevices(): Promise<void> {
  const enumerate = resolveHost()?.enumerateDevices;
  if (enumerate === undefined) return;
  try {
    const listed = await enumerate.call(resolveHost());
    devices.value = toWebcamDevices(listed, resolvedMessages.value.deviceFallback);
  } catch {
    // Enumeration is best-effort; the current list stays in place.
  }
}

async function start(): Promise<WebcamMediaStreamLike | undefined> {
  if (external.value) return undefined;
  if (ownStatus.value === "active" && ownStream.value !== undefined) return ownStream.value;
  const run = ++generation;
  const mediaHost = resolveHost();
  if (mediaHost === undefined) {
    fail({ code: "unsupported", cause: undefined });
    return undefined;
  }
  failure.value = undefined;
  ownStatus.value = "requesting";
  let next: WebcamMediaStreamLike;
  try {
    next = await mediaHost.getUserMedia(
      createWebcamConstraints({
        audio,
        deviceId: selectedDevice.value,
        facingMode: facingState.value.value,
        video: constraints,
      }),
    );
  } catch (cause) {
    if (run === generation) fail({ code: normalizeUserMediaError(cause), cause });
    return undefined;
  }
  if (run !== generation) {
    stopWebcamStream(next);
    return undefined;
  }
  attach(next);
  ownStream.value = next;
  ownStatus.value = "active";
  emit("streamChange", next);
  void refreshDevices();
  return next;
}

function stop(): void {
  generation += 1;
  countdownToken += 1;
  countdown.value = 0;
  releaseOwnStream();
  if (!external.value) ownStatus.value = "idle";
}

async function restartIfActive(): Promise<void> {
  if (external.value || ownStatus.value !== "active") return;
  stop();
  await start();
}

async function switchCamera(): Promise<void> {
  const next: WebcamFacingMode = facingState.value.value === "user" ? "environment" : "user";
  if (facingState.set(next)) emit("update:facingMode", next);
  if (deviceState.set(null)) emit("update:deviceId", null);
  await restartIfActive();
}

async function selectDevice(next: string | undefined): Promise<void> {
  if (deviceState.set(next ?? null)) emit("update:deviceId", next ?? null);
  await restartIfActive();
}

function setPhoto(next: WebcamCapturePhotoResult | null): void {
  if (photoUrl.value !== undefined && typeof URL.revokeObjectURL === "function") {
    URL.revokeObjectURL(photoUrl.value);
  }
  photo.value = next;
  photoUrl.value =
    next !== null && typeof URL.createObjectURL === "function"
      ? URL.createObjectURL(next.blob)
      : undefined;
}

async function capture(): Promise<WebcamCapturePhotoResult | null> {
  if (videoElement.value === null || status.value !== "active") return null;
  capturing.value = true;
  try {
    const result = await captureFrame(videoElement.value, {
      aspectRatio,
      maxWidth,
      mirrored: mirroredState.value,
      quality: captureQuality,
      type: captureType,
    });
    setPhoto(result);
    announcement.value = resolvedMessages.value.captured;
    emit("capture", result);
    return result;
  } catch (error) {
    emit("captureError", error);
    return null;
  } finally {
    capturing.value = false;
  }
}

function wait(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

async function shoot(seconds: number): Promise<WebcamCapturePhotoResult | null> {
  if (status.value !== "active" || countdown.value > 0 || capturing.value) return null;
  const token = ++countdownToken;
  for (let remaining = Math.floor(seconds); remaining > 0; remaining -= 1) {
    countdown.value = remaining;
    announcement.value = resolvedMessages.value.countdown(remaining);
    await wait(Math.max(0, countdownInterval));
    if (token !== countdownToken) return null;
  }
  countdown.value = 0;
  return capture();
}

watch(status, (next, previous) => {
  if (next === "requesting") announcement.value = resolvedMessages.value.requesting;
  else if (next === "active") announcement.value = resolvedMessages.value.active;
  else if (next === "error" && failure.value !== undefined) {
    announcement.value = resolvedMessages.value.error(failure.value.code);
  }
  emit("statusChange", next, previous);
});

// The preview is bound on the client only; `srcObject` has no attribute form.
watch(
  [currentStream, videoElement],
  () => {
    if (videoElement.value === null || !("srcObject" in videoElement.value)) return;
    try {
      // A reactive proxy is not a `MediaStream`; always hand the browser the raw object.
      Reflect.set(videoElement.value, "srcObject", toRaw(currentStream.value) ?? null);
    } catch {
      // Browsers only accept native `MediaStream`s; structural stand-ins stay unbound.
    }
  },
  { flush: "post" },
);

watch([() => facingState.value.value, () => deviceState.value.value], (next, previous) => {
  if (next[0] !== previous[0] || next[1] !== previous[1]) void restartIfActive();
});

function onDeviceChange(): void {
  void refreshDevices();
}

onMounted(() => {
  listenedHost = resolveHost();
  listenedHost?.addEventListener("devicechange", onDeviceChange);
  void refreshDevices();
  if (autoStart) void start();
});

onBeforeUnmount(() => {
  listenedHost?.removeEventListener("devicechange", onDeviceChange);
  listenedHost = undefined;
  stop();
  setPhoto(null);
});

webcamCaptureContext.provide({
  announcement,
  id: baseId,
  messages: resolvedMessages,
  photo,
  photoUrl,
  selectDevice,
  setVideoElement: (next) => {
    videoElement.value = next;
  },
  shoot,
  slotState,
  start,
  stop,
  stream: currentStream,
  switchCamera,
} satisfies WebcamCaptureContextValue);

type WebcamCaptureRootSetupExpose = Omit<
  WebcamCaptureRootExpose,
  keyof WebcamCaptureSlotState | "element" | "photo" | "photoUrl" | "stream"
> & {
  readonly capturing: typeof capturing;
  readonly countdown: typeof countdown;
  readonly deviceId: ComputedRef<string | undefined>;
  readonly devices: typeof devices;
  readonly element: typeof element;
  readonly error: ComputedRef<WebcamCaptureFailure | undefined>;
  readonly external: ComputedRef<boolean>;
  readonly facingMode: ComputedRef<WebcamFacingMode>;
  readonly mirrored: ComputedRef<boolean>;
  readonly photo: typeof photo;
  readonly photoUrl: typeof photoUrl;
  readonly status: ComputedRef<WebcamCaptureStatus>;
  readonly stream: ComputedRef<WebcamMediaStreamLike | undefined>;
};

const exposed = {
  capture,
  capturing,
  clearPhoto: () => setPhoto(null),
  countdown,
  deviceId: selectedDevice,
  devices,
  element,
  error: computed(() => slotState.value.error),
  external,
  facingMode: facingModeValue,
  mirrored: mirroredState,
  photo,
  photoUrl,
  selectDevice,
  start,
  status,
  stop,
  stream: currentStream,
  switchCamera,
} satisfies WebcamCaptureRootSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    :id="baseId"
    ref="element"
    data-vize-ui="webcam-capture-root"
    part="root"
    :data-status="status"
    :data-facing-mode="facingModeValue"
    :data-mirrored="mirroredState ? 'true' : 'false'"
    :data-external="external ? 'true' : undefined"
  >
    <slot v-bind="slotState" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
