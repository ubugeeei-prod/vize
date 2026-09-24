import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope, nextTick, ref } from "vue";

import { renderComposableOnServer } from "./testing/ssr-harness.ts";
import { useBarcodeDetector } from "./use-barcode-detector.ts";
import type { BarcodeFormat, DetectedBarcodeLike } from "./use-barcode-detector.ts";
import type { FrameScheduler } from "./use-raf-fn.ts";

const qr: DetectedBarcodeLike = {
  rawValue: "https://vize.dev",
  format: "qr_code",
  boundingBox: { x: 1, y: 2, width: 3, height: 4 },
  cornerPoints: [
    { x: 1, y: 2 },
    { x: 4, y: 2 },
  ],
};

let detectResult: readonly DetectedBarcodeLike[] | Error = [qr];
const constructed: (BarcodeFormat[] | undefined)[] = [];
const scanned: unknown[] = [];

class FakeBarcodeDetector {
  static getSupportedFormats(): Promise<readonly string[]> {
    return Promise.resolve(["qr_code", "ean_13", "future_format"]);
  }

  constructor(options?: { formats?: BarcodeFormat[] }) {
    constructed.push(options?.formats);
  }

  detect(source: ImageBitmapSource): Promise<readonly DetectedBarcodeLike[]> {
    scanned.push(source);
    return detectResult instanceof Error
      ? Promise.reject(detectResult)
      : Promise.resolve(detectResult);
  }
}

function reset(): void {
  detectResult = [qr];
  constructed.length = 0;
  scanned.length = 0;
}

function createScheduler(): FrameScheduler & {
  frame: (timestamp: number) => void;
  pending: number;
} {
  let callback: ((timestamp: number) => void) | undefined;
  const scheduler = {
    pending: 0,
    requestAnimationFrame: (next: (timestamp: number) => void) => {
      callback = next;
      scheduler.pending += 1;
      return scheduler.pending;
    },
    cancelAnimationFrame: () => {
      callback = undefined;
    },
    frame: (timestamp: number) => {
      const current = callback;
      callback = undefined;
      current?.(timestamp);
    },
  };
  return scheduler;
}

async function flush(): Promise<void> {
  for (let index = 0; index < 4; index += 1) await Promise.resolve();
}

void test("detect normalizes results into plain objects", async () => {
  reset();
  const detector = useBarcodeDetector({ BarcodeDetector: FakeBarcodeDetector });
  assert.equal(detector.supported.value, true);
  const source = new Blob();
  const found = await detector.detect(source);
  assert.deepEqual(found, [
    {
      rawValue: "https://vize.dev",
      format: "qr_code",
      boundingBox: { x: 1, y: 2, width: 3, height: 4 },
      cornerPoints: [
        { x: 1, y: 2 },
        { x: 4, y: 2 },
      ],
    },
  ]);
  assert.deepEqual(detector.barcodes.value, found);
  assert.equal(scanned[0], source);

  detectResult = [{ ...qr, format: "future_format" }];
  assert.equal((await detector.detect(source))[0]?.format, "unknown");
  assert.equal(constructed.length, 1, "the detector instance is reused");
});

void test("recreates the detector when formats change", async () => {
  reset();
  const formats = ref<BarcodeFormat[]>(["qr_code"]);
  const detector = useBarcodeDetector({ BarcodeDetector: FakeBarcodeDetector, formats });
  await detector.detect(new Blob());
  formats.value = ["ean_13", "upc_a"];
  await detector.detect(new Blob());
  assert.deepEqual(constructed, [["qr_code"], ["ean_13", "upc_a"]]);
});

void test("failures land in error and resolve empty", async () => {
  reset();
  const failure = new DOMException("bad image", "InvalidStateError");
  detectResult = failure;
  const detector = useBarcodeDetector({ BarcodeDetector: FakeBarcodeDetector });
  assert.deepEqual(await detector.detect(new Blob()), []);
  assert.equal(detector.error.value, failure);
  detectResult = [];
  await detector.detect(new Blob());
  assert.equal(detector.error.value, undefined);
});

void test("getSupportedFormats filters unknown formats", async () => {
  reset();
  const detector = useBarcodeDetector({ BarcodeDetector: FakeBarcodeDetector });
  assert.deepEqual(await detector.getSupportedFormats(), ["qr_code", "ean_13"]);
});

void test("continuous detection is throttled, skips unready video, and never overlaps", async () => {
  reset();
  const scheduler = createScheduler();
  // Only `readyState` is read from the video source.
  const video = { readyState: 1 } as unknown as HTMLVideoElement;
  const scope = effectScope();
  const detector = scope.run(() =>
    useBarcodeDetector({
      BarcodeDetector: FakeBarcodeDetector,
      source: video,
      fpsLimit: 10,
      scheduler,
    }),
  );
  assert.ok(detector);
  assert.equal(detector.isActive.value, true);

  scheduler.frame(0);
  assert.equal(scanned.length, 0, "video without data is skipped");
  Object.assign(video, { readyState: 4 });
  scheduler.frame(100);
  assert.equal(scanned.length, 1);
  scheduler.frame(150);
  assert.equal(scanned.length, 1, "throttled by fpsLimit");
  await flush();
  scheduler.frame(200);
  assert.equal(scanned.length, 2);
  scheduler.frame(300);
  assert.equal(scanned.length, 2, "an in-flight detection blocks the next frame");
  await flush();
  await nextTick();
  assert.equal(detector.barcodes.value.length, 1);

  scope.stop();
  assert.equal(detector.isActive.value, false);
  scheduler.frame(1_000);
  assert.equal(scanned.length, 2);
});

void test("start and stop control the loop", () => {
  reset();
  const scheduler = createScheduler();
  const detector = useBarcodeDetector({ BarcodeDetector: FakeBarcodeDetector, scheduler });
  assert.equal(detector.isActive.value, false, "no source means no autostart");
  detector.start();
  assert.equal(detector.isActive.value, true);
  scheduler.frame(0);
  assert.equal(scanned.length, 0, "no source to scan");
  detector.stop();
  assert.equal(detector.isActive.value, false);
});

void test("reports unsupported without a constructor", async () => {
  const detector = useBarcodeDetector({ BarcodeDetector: null });
  assert.equal(detector.supported.value, false);
  assert.deepEqual(await detector.detect(new Blob()), []);
  assert.deepEqual(await detector.getSupportedFormats(), []);
});

void test("rejects an invalid fpsLimit", () => {
  assert.throws(
    () => useBarcodeDetector({ BarcodeDetector: null, fpsLimit: 0 }),
    /VIZE_COMPOSE_RAF_INVALID_FPS_LIMIT/u,
  );
});

void test("server rendering schedules nothing", async () => {
  const state = await renderComposableOnServer(() => {
    const detector = useBarcodeDetector({ source: () => undefined });
    return {
      supported: detector.supported,
      barcodes: detector.barcodes,
      isActive: detector.isActive,
    };
  });
  assert.equal(state, '{"supported":false,"barcodes":[],"isActive":true}');
});
