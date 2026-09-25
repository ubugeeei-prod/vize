import type { ComposableProbeMap } from "./probe-types.ts";

/**
 * Conformance probes for catalog entries in group F (test-only).
 *
 * happy-dom implements none of these platform APIs, so most probes inject a
 * fake host that exists only on the client (`typeof window` guard). The
 * server renders the unsupported state; the client must hydrate that same
 * state and only then activate, which the `client` states prove.
 */
export const probesGroupF: ComposableProbeMap = {
  "./use-scheduler-post-task": {
    kind: "component",
    setup: `import { useSchedulerPostTask } from "@/use-scheduler-post-task.ts";
const scheduler = useSchedulerPostTask({
  scheduler:
    typeof window === "undefined"
      ? null
      : { postTask: (callback) => Promise.resolve().then(callback) },
});`,
    state:
      "{ supported: scheduler.supported.value, priority: scheduler.priority.value, pending: scheduler.pending.value }",
    server: { supported: false, priority: "user-visible", pending: 0 },
    client: { supported: true, priority: "user-visible", pending: 0 },
  },
  "./use-screen-details": {
    kind: "component",
    setup: `import { useScreenDetails } from "@/use-screen-details.ts";
const details = useScreenDetails({
  host:
    typeof window === "undefined"
      ? null
      : {
          screen: { isExtended: true },
          getScreenDetails: () => Promise.reject(new Error("not requested")),
          open: () => null,
        },
});`,
    state:
      "{ supported: details.supported.value, extended: details.isExtended.value, screens: details.screens.value.length }",
    server: { supported: false, extended: false, screens: 0 },
    client: { supported: true, extended: true, screens: 0 },
  },
  "./use-serial": {
    kind: "component",
    setup: `import { useSerial } from "@/use-serial.ts";
const serial = useSerial({
  host:
    typeof window === "undefined"
      ? null
      : Object.assign(new EventTarget(), {
          getPorts: () => Promise.resolve([]),
          requestPort: () => Promise.reject(new Error("no port")),
        }),
});`,
    state:
      "{ supported: serial.supported.value, ports: serial.ports.value.length, connected: serial.connected.value }",
    server: { supported: false, ports: 0, connected: false },
    client: { supported: true, ports: 0, connected: false },
  },
  "./use-service-worker": {
    kind: "component",
    setup: `import { useServiceWorker } from "@/use-service-worker.ts";
const worker = useServiceWorker("/sw.js", {
  immediate: false,
  container:
    typeof window === "undefined"
      ? null
      : Object.assign(new EventTarget(), {
          controller: null,
          register: () => Promise.reject(new Error("offline")),
        }),
});`,
    state:
      "{ supported: worker.supported.value, state: worker.state.value, update: worker.updateAvailable.value }",
    server: {
      supported: false,
      state: { installing: null, waiting: null, active: null },
      update: false,
    },
    client: {
      supported: true,
      state: { installing: null, waiting: null, active: null },
      update: false,
    },
  },
  "./use-storage-estimate": {
    kind: "component",
    setup: `import { usePersistentStorage, useStorageEstimate } from "@/use-storage-estimate.ts";
const storage =
  typeof window === "undefined"
    ? null
    : {
        estimate: () => Promise.resolve({ usage: 10, quota: 100 }),
        persisted: () => Promise.resolve(true),
        persist: () => Promise.resolve(true),
      };
const estimate = useStorageEstimate({ storage });
const persistent = usePersistentStorage({ storage });`,
    state:
      "{ supported: estimate.supported.value, usage: estimate.usage.value, quota: estimate.quota.value, percent: estimate.percentUsed.value, persistSupported: persistent.supported.value, persisted: persistent.persisted.value }",
    server: {
      supported: false,
      usage: null,
      quota: null,
      percent: null,
      persistSupported: false,
      persisted: false,
    },
    client: {
      supported: true,
      usage: 10,
      quota: 100,
      percent: 10,
      persistSupported: true,
      persisted: true,
    },
  },
  "./use-usb": {
    kind: "component",
    setup: `import { useUSB } from "@/use-usb.ts";
const usb = useUSB({
  host:
    typeof window === "undefined"
      ? null
      : Object.assign(new EventTarget(), {
          getDevices: () => Promise.resolve([]),
          requestDevice: () => Promise.reject(new Error("no device")),
        }),
});`,
    state: "{ supported: usb.supported.value, devices: usb.devices.value.length }",
    server: { supported: false, devices: 0 },
    client: { supported: true, devices: 0 },
  },
  "./use-user-activation": {
    kind: "component",
    setup: `import { useUserActivation } from "@/use-user-activation.ts";
const activation = useUserActivation({
  userActivation: typeof window === "undefined" ? null : { hasBeenActive: true, isActive: false },
});`,
    state:
      "{ supported: activation.supported.value, hasBeenActive: activation.hasBeenActive.value, isActive: activation.isActive.value }",
    server: { supported: false, hasBeenActive: false, isActive: false },
    client: { supported: true, hasBeenActive: true, isActive: false },
  },
  "./use-view-transition": {
    kind: "component",
    setup: `import { useViewTransition } from "@/use-view-transition.ts";
const transition = useViewTransition({
  reducedMotion: false,
  document:
    typeof window === "undefined"
      ? null
      : {
          startViewTransition: () => ({
            finished: Promise.resolve(),
            ready: Promise.resolve(),
            updateCallbackDone: Promise.resolve(),
            skipTransition: () => undefined,
          }),
        },
});`,
    state:
      "{ supported: transition.supported.value, transitioning: transition.isTransitioning.value }",
    server: { supported: false, transitioning: false },
    client: { supported: true, transitioning: false },
  },
  "./use-web-authn": {
    kind: "component",
    setup: `import { useWebAuthn } from "@/use-web-authn.ts";
const authn = useWebAuthn({
  credentials:
    typeof window === "undefined"
      ? null
      : {
          create: () => Promise.reject(new Error("cancelled")),
          get: () => Promise.reject(new Error("cancelled")),
        },
});`,
    state: "{ supported: authn.supported.value, pending: authn.pending.value }",
    server: { supported: false, pending: false },
    client: { supported: true, pending: false },
  },
  "./use-web-locks": {
    kind: "component",
    setup: `import { useWebLocks } from "@/use-web-locks.ts";
const locks = useWebLocks({
  locks:
    typeof window === "undefined"
      ? null
      : {
          request: (_name, _options, callback) => Promise.resolve(callback(null)),
          query: () => Promise.resolve({ held: [], pending: [] }),
        },
});`,
    state:
      "{ supported: locks.supported.value, held: locks.held.value, pending: locks.pending.value }",
    server: { supported: false, held: [], pending: [] },
    client: { supported: true, held: [], pending: [] },
  },
  "./use-web-midi": {
    kind: "component",
    setup: `import { useWebMIDI } from "@/use-web-midi.ts";
const midi = useWebMIDI({
  host:
    typeof window === "undefined"
      ? null
      : { requestMIDIAccess: () => Promise.reject(new Error("denied")) },
});`,
    state:
      "{ supported: midi.supported.value, status: midi.status.value, inputs: midi.inputs.value.length }",
    server: { supported: false, status: "idle", inputs: 0 },
    client: { supported: true, status: "idle", inputs: 0 },
  },
  "./use-web-vitals": {
    kind: "component",
    setup: `import { useWebVitals } from "@/use-web-vitals.ts";
const vitals = useWebVitals();`,
    state: "{ supported: vitals.supported.value, metrics: vitals.metrics.value }",
    // happy-dom has no PerformanceObserver, so the client stays unsupported;
    // the probe still proves hydration renders the server state.
    server: { supported: false, metrics: {} },
    client: { supported: false, metrics: {} },
  },
};
