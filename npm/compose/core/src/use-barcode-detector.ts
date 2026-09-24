import {
  computed,
  hasInjectionContext,
  readonly,
  shallowRef,
  toValue,
  unref,
  watchPostEffect,
} from "vue";
import type { ComputedRef, MaybeRef, MaybeRefOrGetter, ShallowRef } from "vue";

import { useRafFn } from "./use-raf-fn.ts";
import type { FrameScheduler } from "./use-raf-fn.ts";

/** Barcode symbology supported by the Barcode Detection API. */
export type BarcodeFormat =
  | "aztec"
  | "code_128"
  | "code_39"
  | "code_93"
  | "codabar"
  | "data_matrix"
  | "ean_13"
  | "ean_8"
  | "itf"
  | "pdf417"
  | "qr_code"
  | "upc_a"
  | "upc_e"
  | "unknown";

/** Axis-aligned bounds of a detected barcode, in source pixels. */
export interface BarcodeBoundingBox {
  /** Left edge. */
  readonly x: number;
  /** Top edge. */
  readonly y: number;
  /** Width. */
  readonly width: number;
  /** Height. */
  readonly height: number;
}

/** Corner of a detected barcode, in source pixels. */
export interface BarcodePoint {
  /** Horizontal coordinate. */
  readonly x: number;
  /** Vertical coordinate. */
  readonly y: number;
}

/** Normalized detection result. */
export interface DetectedBarcode {
  /** Decoded text. */
  readonly rawValue: string;
  /** Symbology (`"unknown"` for formats this module does not know). */
  readonly format: BarcodeFormat;
  /** Bounding box. */
  readonly boundingBox: BarcodeBoundingBox;
  /** Corner points, clockwise from top-left. */
  readonly cornerPoints: readonly BarcodePoint[];
}

/** Raw detection result produced by a host detector. */
export interface DetectedBarcodeLike {
  /** Decoded text. */
  readonly rawValue: string;
  /** Symbology name. */
  readonly format: string;
  /** Bounding box. */
  readonly boundingBox: BarcodeBoundingBox;
  /** Corner points. */
  readonly cornerPoints: readonly BarcodePoint[];
}

/** Minimal `BarcodeDetector` instance. */
export interface BarcodeDetectorLike {
  /** Detect barcodes in an image source. */
  detect(source: ImageBitmapSource): Promise<readonly DetectedBarcodeLike[]>;
}

/** Minimal `BarcodeDetector` constructor. */
export interface BarcodeDetectorHost {
  /** Create a detector restricted to `formats`. */
  new (options?: { formats?: BarcodeFormat[] }): BarcodeDetectorLike;
  /** Formats the platform can detect. */
  getSupportedFormats?(): Promise<readonly string[]>;
}

/** Options for {@link useBarcodeDetector}. */
export interface UseBarcodeDetectorOptions {
  /**
   * `BarcodeDetector` constructor for alternate runtimes and tests. A ref
   * (not a getter) because the host is a constructor function.
   *
   * @default window.BarcodeDetector when it exists
   */
  readonly BarcodeDetector?: MaybeRef<BarcodeDetectorHost | null | undefined>;

  /**
   * Formats to detect. Reactive; a new detector is created when it changes.
   *
   * @default undefined (every supported format)
   */
  readonly formats?: MaybeRefOrGetter<readonly BarcodeFormat[] | undefined>;

  /**
   * Source scanned continuously (for example a camera `<video>`). Video
   * frames are skipped until `readyState >= 2`.
   *
   * @default undefined
   */
  readonly source?: MaybeRefOrGetter<ImageBitmapSource | null | undefined>;

  /**
   * Start continuous detection immediately.
   *
   * @default true when `source` is set
   */
  readonly immediate?: boolean;

  /**
   * Maximum continuous detections per second. A frame is also skipped while
   * the previous detection is still running.
   *
   * @default 10
   */
  readonly fpsLimit?: MaybeRefOrGetter<number | undefined>;

  /**
   * Frame host for continuous detection. An injected scheduler also drives
   * frames without a browser `window` (native or offscreen hosts, tests).
   *
   * @default globalThis animation-frame functions when available
   */
  readonly scheduler?: FrameScheduler;
}

/** Reactive state and actions returned by {@link useBarcodeDetector}. */
export interface BarcodeDetectorControls {
  /** Whether the Barcode Detection API is available. */
  readonly supported: ComputedRef<boolean>;
  /** Barcodes found by the most recent successful detection. */
  readonly barcodes: Readonly<ShallowRef<readonly DetectedBarcode[]>>;
  /** Most recent detection failure, cleared on success. */
  readonly error: Readonly<ShallowRef<unknown>>;
  /** Whether continuous detection is running. */
  readonly isActive: Readonly<ShallowRef<boolean>>;
  /**
   * Detect barcodes once.
   *
   * @param source Image, video, canvas, bitmap, blob, or image data.
   * @returns Detected barcodes; empty when unsupported or failing (see `error`).
   */
  readonly detect: (source: ImageBitmapSource) => Promise<readonly DetectedBarcode[]>;
  /**
   * Formats the platform supports.
   *
   * @returns Known supported formats; empty when unsupported or failing.
   */
  readonly getSupportedFormats: () => Promise<BarcodeFormat[]>;
  /** Start continuous detection of `source`. */
  readonly start: () => void;
  /** Stop continuous detection. Idempotent. */
  readonly stop: () => void;
}

const knownFormats: readonly string[] = [
  "aztec",
  "code_128",
  "code_39",
  "code_93",
  "codabar",
  "data_matrix",
  "ean_13",
  "ean_8",
  "itf",
  "pdf417",
  "qr_code",
  "upc_a",
  "upc_e",
  "unknown",
];

function isBarcodeFormat(value: string): value is BarcodeFormat {
  return knownFormats.includes(value);
}

function isDetectorHost(candidate: unknown): candidate is BarcodeDetectorHost {
  return typeof candidate === "function";
}

function browserDetector(): BarcodeDetectorHost | undefined {
  if (typeof window === "undefined") return undefined;
  const candidate: unknown = Reflect.get(window, "BarcodeDetector");
  return isDetectorHost(candidate) ? candidate : undefined;
}

function isFrameReady(source: ImageBitmapSource): boolean {
  return (
    !("readyState" in source) || typeof source.readyState !== "number" || source.readyState >= 2
  );
}

function normalize(barcode: DetectedBarcodeLike): DetectedBarcode {
  const { x, y, width, height } = barcode.boundingBox;
  return {
    rawValue: barcode.rawValue,
    format: isBarcodeFormat(barcode.format) ? barcode.format : "unknown",
    boundingBox: { x, y, width, height },
    cornerPoints: barcode.cornerPoints.map((point) => ({ x: point.x, y: point.y })),
  };
}

/**
 * Detect barcodes and QR codes with the Barcode Detection API.
 *
 * `detect(source)` scans once; with a `source` (typically a camera video),
 * detection also runs on animation frames through {@link useRafFn}, throttled
 * by `fpsLimit` and never overlapping. Results are normalized into plain
 * {@link DetectedBarcode} objects. The frame loop stops with the owning
 * reactive scope; outside a scope call `stop()`.
 *
 * Server rendering: nothing is constructed or scheduled, `supported` is
 * false and `barcodes` is empty.
 * Inside a component `supported` turns true only after mounting, so
 * hydration renders this server state first.
 *
 * @example
 * ```ts
 * const video = useTemplateRef<HTMLVideoElement>("video");
 * const { barcodes } = useBarcodeDetector({ source: video, formats: ["qr_code"] });
 * ```
 *
 * @param options Constructor host, formats, and continuous-scan policy.
 * @default options {}
 * @throws `RangeError` tagged `VIZE_COMPOSE_RAF_INVALID_FPS_LIMIT` for an invalid `fpsLimit`.
 * @returns Detector state and actions.
 */
export function useBarcodeDetector(
  options: UseBarcodeDetectorOptions = {},
): BarcodeDetectorControls {
  const barcodes = shallowRef<readonly DetectedBarcode[]>([]);
  const error = shallowRef<unknown>(undefined);
  let cached: { host: BarcodeDetectorHost; key: string; detector: BarcodeDetectorLike } | undefined;
  let busy = false;

  // Inside a component the host is resolved only after mounting, so a
  // hydrating client renders the server's unsupported state first. Outside
  // components it resolves synchronously.
  const hydrated = shallowRef(!hasInjectionContext());
  if (!hydrated.value) {
    watchPostEffect(() => {
      hydrated.value = true;
    });
  }

  const resolveHost = (): BarcodeDetectorHost | undefined =>
    options.BarcodeDetector === undefined
      ? browserDetector()
      : (unref(options.BarcodeDetector) ?? undefined);

  const resolveDetector = (Host: BarcodeDetectorHost): BarcodeDetectorLike => {
    const formats = toValue(options.formats);
    const key = formats?.join() ?? "*";
    if (cached?.host !== Host || cached.key !== key) {
      const detector = new Host(formats ? { formats: [...formats] } : undefined);
      cached = { host: Host, key, detector };
    }
    return cached.detector;
  };

  const detect = async (source: ImageBitmapSource): Promise<readonly DetectedBarcode[]> => {
    const host = resolveHost();
    if (!host) return [];
    try {
      const found = (await resolveDetector(host).detect(source)).map(normalize);
      barcodes.value = found;
      error.value = undefined;
      return found;
    } catch (cause) {
      error.value = cause;
      return [];
    }
  };

  const getSupportedFormats = async (): Promise<BarcodeFormat[]> => {
    try {
      return [...((await resolveHost()?.getSupportedFormats?.()) ?? [])].filter(isBarcodeFormat);
    } catch {
      return [];
    }
  };

  const loop = useRafFn(
    () => {
      const source = toValue(options.source);
      if (busy || !source || !isFrameReady(source)) return;
      busy = true;
      void detect(source).finally(() => {
        busy = false;
      });
    },
    {
      immediate: options.immediate ?? options.source !== undefined,
      fpsLimit: () => toValue(options.fpsLimit) ?? 10,
      ...(options.scheduler ? { scheduler: options.scheduler, runOnServer: true } : {}),
    },
  );

  return {
    supported: computed(() => hydrated.value && resolveHost() !== undefined),
    barcodes: readonly(barcodes),
    error: readonly(error),
    isActive: loop.isActive,
    detect,
    getSupportedFormats,
    start: loop.resume,
    stop: loop.pause,
  };
}
