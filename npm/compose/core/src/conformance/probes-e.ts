import type { ComposableProbeMap } from "./probe-types.ts";

/** Conformance probes for catalog entries in group E (test-only). */
export const probesGroupE: ComposableProbeMap = {
  "./use-barcode-detector": {
    kind: "component",
    setup: `import { useBarcodeDetector } from "@/use-barcode-detector.ts";
const client = typeof window !== "undefined";
class FakeBarcodeDetector {
  detect() {
    return Promise.resolve([]);
  }
}
const detector = useBarcodeDetector({ BarcodeDetector: client ? FakeBarcodeDetector : undefined });`,
    state:
      "{ supported: detector.supported.value, active: detector.isActive.value, barcodes: detector.barcodes.value }",
    server: { supported: false, active: false, barcodes: [] },
    client: { supported: true, active: false, barcodes: [] },
  },
  "./use-bluetooth": {
    kind: "component",
    setup: `import { useBluetooth } from "@/use-bluetooth.ts";
const client = typeof window !== "undefined";
const host = client
  ? Object.assign(new EventTarget(), {
      getAvailability: () => Promise.resolve(true),
      requestDevice: () => Promise.reject(new Error("no device")),
    })
  : undefined;
const bluetooth = useBluetooth({ host });`,
    state:
      "{ supported: bluetooth.supported.value, available: bluetooth.available.value, connected: bluetooth.connected.value }",
    server: { supported: false, available: false, connected: false },
    client: { supported: true, available: true, connected: false },
  },
  "./use-builtin-ai": {
    kind: "component",
    setup: `import { useLanguageDetector, useSummarizer, useTranslator } from "@/use-builtin-ai.ts";
const client = typeof window !== "undefined";
const factory = client
  ? {
      availability: () => Promise.resolve("available"),
      create: () => Promise.reject(new Error("not in probes")),
    }
  : undefined;
const translator = useTranslator({ sourceLanguage: "en", targetLanguage: "ja", host: factory });
const detector = useLanguageDetector();
const summarizer = useSummarizer();`,
    state:
      "{ translator: [translator.supported.value, translator.status.value], detector: [detector.supported.value, detector.status.value], summarizer: [summarizer.supported.value, summarizer.status.value] }",
    server: {
      translator: [false, "unsupported"],
      detector: [false, "unsupported"],
      summarizer: [false, "unsupported"],
    },
    client: {
      translator: [true, "idle"],
      detector: [false, "unsupported"],
      summarizer: [false, "unsupported"],
    },
  },
  "./use-compression-stream": {
    kind: "component",
    setup: `import { useCompressionStream } from "@/use-compression-stream.ts";
const client = typeof window !== "undefined";
class FakeStream {
  readonly readable = new ReadableStream();
  readonly writable = new WritableStream();
}
const compression = useCompressionStream({
  CompressionStream: client ? FakeStream : undefined,
  DecompressionStream: client ? FakeStream : undefined,
});`,
    state: "{ supported: compression.supported.value, pending: compression.pending.value }",
    server: { supported: false, pending: false },
    client: { supported: true, pending: false },
  },
  "./use-contact-picker": {
    kind: "component",
    setup: `import { useContactPicker } from "@/use-contact-picker.ts";
const client = typeof window !== "undefined";
const contacts = client
  ? { getProperties: () => Promise.resolve(["name"]), select: () => Promise.resolve([]) }
  : undefined;
const picker = useContactPicker({ contacts });`,
    state: "{ supported: picker.supported.value, pending: picker.pending.value }",
    server: { supported: false, pending: false },
    client: { supported: true, pending: false },
  },
  "./use-gamepad": {
    kind: "component",
    setup: `import { useGamepad } from "@/use-gamepad.ts";
const client = typeof window !== "undefined";
const host = client
  ? {
      navigator: { getGamepads: () => [] },
      addEventListener: () => undefined,
      removeEventListener: () => undefined,
    }
  : undefined;
const gamepad = useGamepad({ host });`,
    state:
      "{ supported: gamepad.supported.value, active: gamepad.isActive.value, pads: gamepad.gamepads.value.length }",
    server: { supported: false, active: false, pads: 0 },
    client: { supported: true, active: true, pads: 0 },
  },
  "./use-hid": {
    kind: "component",
    setup: `import { useHID } from "@/use-hid.ts";
const client = typeof window !== "undefined";
const host = client
  ? Object.assign(new EventTarget(), {
      getDevices: () => Promise.resolve([]),
      requestDevice: () => Promise.resolve([]),
    })
  : undefined;
const hid = useHID({ host });`,
    state: "{ supported: hid.supported.value, devices: hid.devices.value.length }",
    server: { supported: false, devices: 0 },
    client: { supported: true, devices: 0 },
  },
  "./use-navigation-api": {
    kind: "component",
    setup: `import { useNavigationApi } from "@/use-navigation-api.ts";
const navigation = useNavigationApi();`,
    state:
      "{ supported: navigation.supported.value, back: navigation.canGoBack.value, forward: navigation.canGoForward.value, entry: navigation.currentEntry.value }",
    server: { supported: false, back: false, forward: false, entry: null },
    client: { supported: false, back: false, forward: false, entry: null },
    // happy-dom has no Navigation API and a faithful fake needs the whole
    // history surface; this probe proves the unsupported state is stable.
  },
  "./use-payment-request": {
    kind: "component",
    setup: `import { usePaymentRequest } from "@/use-payment-request.ts";
const client = typeof window !== "undefined";
class FakePaymentRequest extends EventTarget {
  show() {
    return Promise.reject(new Error("not in probes"));
  }
  abort() {
    return Promise.resolve();
  }
  canMakePayment() {
    return Promise.resolve(false);
  }
}
const payment = usePaymentRequest({
  PaymentRequest: client ? FakePaymentRequest : undefined,
  methods: [{ supportedMethods: "https://pay.example" }],
  details: { total: { label: "Total", amount: { currency: "USD", value: "1.00" } } },
});`,
    state: "{ supported: payment.supported.value, state: payment.state.value }",
    server: { supported: false, state: "idle" },
    client: { supported: true, state: "idle" },
  },
  "./use-performance-observer": {
    kind: "component",
    setup: `import { usePerformanceObserver } from "@/use-performance-observer.ts";
const observer = usePerformanceObserver("mark", () => undefined);`,
    state: "{ supported: observer.supported.value, active: observer.isActive.value }",
    server: { supported: false, active: false },
    client: { supported: true, active: true },
  },
  "./use-push-subscription": {
    kind: "component",
    setup: `import { usePushSubscription } from "@/use-push-subscription.ts";
const client = typeof window !== "undefined";
const registration = client
  ? {
      pushManager: {
        getSubscription: () => Promise.resolve(null),
        subscribe: () => Promise.reject(new Error("not in probes")),
        permissionState: () => Promise.resolve("prompt"),
      },
    }
  : undefined;
const push = usePushSubscription({ registration });`,
    state: "{ supported: push.supported.value, json: push.json.value }",
    server: { supported: false, json: null },
    client: { supported: true, json: null },
  },
  "./use-request-idle-callback": {
    kind: "component",
    setup: `import { useRequestIdleCallback } from "@/use-request-idle-callback.ts";
const client = typeof window !== "undefined";
const host = client
  ? { requestIdleCallback: () => 1, cancelIdleCallback: () => undefined }
  : undefined;
const idle = useRequestIdleCallback(() => undefined, { host });`,
    state: "{ supported: idle.supported.value, pending: idle.isPending.value }",
    server: { supported: false, pending: false },
    client: { supported: true, pending: true },
  },
};
