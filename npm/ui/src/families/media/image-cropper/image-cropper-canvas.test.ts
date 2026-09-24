import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import { ImageCropperError, cropImage } from "./image-cropper-canvas.ts";

interface CanvasLog {
  readonly calls: string[];
  readonly canvases: HTMLCanvasElement[];
  readonly images: HTMLImageElement[];
}

function withCanvas(
  run: (log: CanvasLog) => Promise<void>,
  options: { readonly context?: boolean; readonly blob?: boolean } = {},
): Promise<void> {
  const log: CanvasLog = { calls: [], canvases: [], images: [] };
  const createElement = document.createElement.bind(document);
  document.createElement = ((tag: string) => {
    const element = createElement(tag);
    if (element instanceof HTMLImageElement) log.images.push(element);
    if (tag !== "canvas") return element;
    const canvas = element as HTMLCanvasElement;
    log.canvases.push(canvas);
    const context = {
      scale: (x: number, y: number) => log.calls.push(`scale ${x} ${y}`),
      translate: (x: number, y: number) => log.calls.push(`translate ${x} ${y}`),
      rotate: (radians: number) =>
        log.calls.push(`rotate ${Math.round((radians * 180) / Math.PI)}`),
      drawImage: (_image: unknown, x: number, y: number) => log.calls.push(`draw ${x} ${y}`),
    };
    Object.defineProperty(canvas, "getContext", {
      value: () => (options.context === false ? null : context),
    });
    Object.defineProperty(canvas, "toDataURL", {
      value: (type: string, quality?: number) =>
        `data:${type};${canvas.width}x${canvas.height};q=${String(quality)}`,
    });
    Object.defineProperty(canvas, "toBlob", {
      value: (callback: (blob: Blob | null) => void, type: string) =>
        callback(
          options.blob === false ? null : new Blob([`${canvas.width}x${canvas.height}`], { type }),
        ),
    });
    return canvas;
  }) as typeof document.createElement;
  return run(log).finally(() => {
    document.createElement = createElement as typeof document.createElement;
  });
}

function loadedImage(width = 400, height = 200): HTMLImageElement {
  const image = document.createElement("img");
  Object.defineProperty(image, "complete", { value: true });
  Object.defineProperty(image, "naturalWidth", { value: width });
  Object.defineProperty(image, "naturalHeight", { value: height });
  return image;
}

test("crops, rotates, downscales, and encodes blobs and data URLs", async () => {
  await withCanvas(async (log) => {
    const blob = await cropImage(loadedImage(), { x: 10.4, y: 20.6, width: 100, height: 50 });
    assert.ok(blob instanceof Blob);
    assert.equal(blob.type, "image/png");
    assert.equal(await blob.text(), "100x50");
    assert.deepEqual(log.calls, ["scale 1 1", "translate 190 79", "rotate 0", "draw -200 -100"]);

    log.calls.length = 0;
    const url = await cropImage(
      loadedImage(),
      { x: 0, y: 0, width: 200, height: 400 },
      { output: "data-url", rotation: 90, type: "image/jpeg", quality: 0.7, maxWidth: 100 },
    );
    assert.equal(url, "data:image/jpeg;100x200;q=0.7");
    assert.deepEqual(log.calls, [
      "scale 0.5 0.5",
      "translate 100 200",
      "rotate 90",
      "draw -200 -100",
    ]);

    const capped = await cropImage(
      loadedImage(),
      { x: 0, y: 0, width: 100, height: 100 },
      { output: "data-url", maxHeight: 25 },
    );
    assert.equal(capped, "data:image/png;25x25;q=undefined");
  });
});

test("loads URL sources with CORS before cropping", async () => {
  await withCanvas(async (log) => {
    const pending = cropImage(
      "/photo.png",
      { x: 0, y: 0, width: 10, height: 10 },
      { output: "data-url" },
    );
    const image = log.images[0];
    assert.ok(image);
    assert.equal(image.crossOrigin, "anonymous");
    assert.match(image.src, /\/photo\.png$/);
    Object.defineProperty(image, "naturalWidth", { value: 20 });
    Object.defineProperty(image, "naturalHeight", { value: 20 });
    image.dispatchEvent(new Event("load"));
    assert.equal(await pending, "data:image/png;10x10;q=undefined");

    const failing = cropImage(
      "/missing.png",
      { x: 0, y: 0, width: 10, height: 10 },
      {
        crossOrigin: "use-credentials",
      },
    );
    const second = log.images[1];
    assert.equal(second?.crossOrigin, "use-credentials");
    second?.dispatchEvent(new Event("error"));
    await assert.rejects(
      failing,
      (error: unknown) =>
        error instanceof ImageCropperError && error.code === "VIZE_UI_IMAGE_CROPPER_IMAGE_LOAD",
    );
  });
});

test("rejects empty crops, missing canvases, failed loads, and failed encodes with typed errors", async () => {
  const isCode = (code: string) => (error: unknown) =>
    error instanceof ImageCropperError &&
    error.code === code &&
    error.message.startsWith(`${code}: `);

  await withCanvas(async () => {
    await assert.rejects(
      cropImage(loadedImage(), { x: 0, y: 0, width: 0.2, height: 10 }),
      isCode("VIZE_UI_IMAGE_CROPPER_EMPTY_CROP"),
    );
  });
  await withCanvas(
    async () => {
      await assert.rejects(
        cropImage(loadedImage(), { x: 0, y: 0, width: 10, height: 10 }),
        isCode("VIZE_UI_IMAGE_CROPPER_CANVAS_UNAVAILABLE"),
      );
    },
    { context: false },
  );
  await withCanvas(
    async () => {
      await assert.rejects(
        cropImage(loadedImage(), { x: 0, y: 0, width: 10, height: 10 }),
        isCode("VIZE_UI_IMAGE_CROPPER_ENCODE_FAILED"),
      );
    },
    { blob: false },
  );

  const pendingImage = document.createElement("img");
  Object.defineProperty(pendingImage, "complete", { value: false });
  const loading = cropImage(pendingImage, { x: 0, y: 0, width: 10, height: 10 });
  pendingImage.dispatchEvent(new Event("error"));
  await assert.rejects(loading, isCode("VIZE_UI_IMAGE_CROPPER_IMAGE_LOAD"));

  const documentDescriptor = Object.getOwnPropertyDescriptor(globalThis, "document");
  Reflect.deleteProperty(globalThis, "document");
  try {
    await assert.rejects(
      cropImage("/photo.png", { x: 0, y: 0, width: 10, height: 10 }),
      isCode("VIZE_UI_IMAGE_CROPPER_CANVAS_UNAVAILABLE"),
    );
  } finally {
    if (documentDescriptor) Object.defineProperty(globalThis, "document", documentDescriptor);
  }
});

test("waits for pending images to load", async () => {
  await withCanvas(async () => {
    const image = document.createElement("img");
    let complete = false;
    Object.defineProperty(image, "complete", { get: () => complete });
    Object.defineProperty(image, "naturalWidth", { get: () => (complete ? 40 : 0) });
    Object.defineProperty(image, "naturalHeight", { get: () => (complete ? 20 : 0) });
    const pending = cropImage(image, { x: 0, y: 0, width: 40, height: 20 }, { output: "data-url" });
    complete = true;
    image.dispatchEvent(new Event("load"));
    assert.equal(await pending, "data:image/png;40x20;q=undefined");
  });
});
