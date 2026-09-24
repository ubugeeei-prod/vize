import assert from "node:assert/strict";

import { afterEach, beforeEach, test } from "vite-plus/test";
import { h, nextTick } from "vue";

import type { CropArea, ImageCropperRootExpose, ImageCropperSlotState } from "./image-cropper.ts";
import ImageCropperArea from "./image-cropper-area.vue";
import ImageCropperGrid from "./image-cropper-grid.vue";
import ImageCropperHandle from "./image-cropper-handle.vue";
import ImageCropperImage from "./image-cropper-image.vue";
import ImageCropperRoot from "./image-cropper-root.vue";
import ImageCropperViewport from "./image-cropper-viewport.vue";
import { mountInteraction } from "../../../testing/mount.ts";

const restorers: (() => void)[] = [];

function override(target: object, name: PropertyKey, descriptor: PropertyDescriptor): void {
  const previous = Object.getOwnPropertyDescriptor(target, name);
  Object.defineProperty(target, name, { configurable: true, ...descriptor });
  restorers.push(() => {
    if (previous) Object.defineProperty(target, name, previous);
    else Reflect.deleteProperty(target, name);
  });
}

let natural = { width: 400, height: 200 };

beforeEach(() => {
  natural = { width: 400, height: 200 };
  override(HTMLImageElement.prototype, "naturalWidth", { get: () => natural.width });
  override(HTMLImageElement.prototype, "naturalHeight", { get: () => natural.height });
  override(HTMLImageElement.prototype, "complete", { get: () => false });
  override(HTMLElement.prototype, "getBoundingClientRect", {
    value(this: HTMLElement) {
      const viewport = this.getAttribute("data-vize-ui") === "image-cropper-viewport";
      const size = viewport ? 200 : 0;
      return { left: 0, top: 0, width: size, height: size, right: size, bottom: size };
    },
  });
});

afterEach(() => {
  while (restorers.length > 0) restorers.pop()?.();
});

function mountCropper(
  props: Record<string, unknown> = {},
  imageProps: Record<string, unknown> = {},
) {
  return mountInteraction(ImageCropperRoot, {
    props: { id: "crop", ...props },
    record: ["update:modelValue", "update:zoom", "update:rotation", "change", "cropEnd"],
    slots: {
      default: (state: ImageCropperSlotState) => [
        h("output", { "data-ready": String(state.ready) }, state.interaction),
        h(ImageCropperViewport, null, () => [
          h(ImageCropperImage, { src: "/photo.png", alt: "Team photo", ...imageProps }),
          h(ImageCropperArea, null, () => [
            h(ImageCropperGrid),
            ...(["n", "e", "s", "w", "ne", "nw", "se", "sw"] as const).map((position) =>
              h(ImageCropperHandle, { position, key: position }),
            ),
          ]),
        ]),
      ],
    },
  });
}

function part(root: HTMLElement, name: string): HTMLElement {
  const element = root.querySelector<HTMLElement>(`[data-vize-ui="${name}"]`);
  assert.ok(element, `${name} must render`);
  return element;
}

async function load(root: HTMLElement): Promise<void> {
  part(root, "image-cropper-image").dispatchEvent(new Event("load"));
  await nextTick();
  await nextTick();
}

function pointer(type: string, init: PointerEventInit = {}): PointerEvent {
  return new PointerEvent(type, {
    bubbles: true,
    cancelable: true,
    pointerId: 1,
    pointerType: "mouse",
    button: 0,
    ...init,
  });
}

function crop(handle: ReturnType<typeof mountCropper>): CropArea | null {
  return handle.exposes<ImageCropperRootExpose>().crop;
}

test("publishes the initial centered crop and measured geometry once the image loads", async () => {
  const handle = mountCropper();
  const root = handle.root();
  const area = part(root, "image-cropper-area");
  assert.equal(root.id, "crop");
  assert.equal(root.getAttribute("data-vize-ui"), "image-cropper-root");
  assert.equal(root.hasAttribute("data-ready"), false);
  assert.equal(area.getAttribute("style"), null);
  assert.equal(area.getAttribute("aria-label"), null);

  await load(root);
  assert.equal(root.getAttribute("data-ready"), "true");
  assert.deepEqual(crop(handle), { x: 40, y: 20, width: 320, height: 160 });
  assert.deepEqual(handle.wrapper.emitted("change")?.[0], [
    { x: 40, y: 20, width: 320, height: 160 },
    "init",
  ]);
  const viewport = part(root, "image-cropper-viewport");
  assert.equal(viewport.style.position, "relative");
  assert.equal(viewport.style.overflow, "hidden");
  assert.equal(viewport.style.getPropertyValue("--vize-ui-image-cropper-scale"), "0.5");
  const image = part(root, "image-cropper-image");
  assert.equal(image.getAttribute("draggable"), "false");
  assert.equal(image.getAttribute("alt"), "Team photo");
  assert.equal(image.style.left, "0px");
  assert.equal(image.style.top, "50px");
  assert.equal(image.style.width, "200px");
  assert.equal(image.style.height, "100px");
  assert.equal(image.style.transform, "rotate(0deg)");
  assert.equal(area.style.left, "20px");
  assert.equal(area.style.top, "60px");
  assert.equal(area.style.width, "160px");
  assert.equal(area.style.height, "80px");
  assert.equal(area.style.getPropertyValue("--vize-ui-image-cropper-crop-width"), "320");
  assert.equal(area.getAttribute("role"), "group");
  assert.equal(area.getAttribute("tabindex"), "0");
  assert.equal(area.getAttribute("aria-roledescription"), "crop area");
  assert.equal(area.getAttribute("aria-label"), "Crop area 320 by 160 pixels at 40, 20");
  assert.equal(root.querySelectorAll('[data-vize-ui="image-cropper-handle"]').length, 8);
  assert.equal(
    root.querySelector('[data-vize-ui="image-cropper-handle"]')?.getAttribute("aria-hidden"),
    "true",
  );
  handle.unmount();
});

test("dragging the area moves the crop in image pixels and commits on release", async () => {
  const handle = mountCropper();
  const root = handle.root();
  await load(root);
  const area = part(root, "image-cropper-area");

  area.dispatchEvent(pointer("pointerdown", { clientX: 50, clientY: 70 }));
  await nextTick();
  assert.equal(root.getAttribute("data-interaction"), "moving");
  assert.ok(document.activeElement === area);
  area.dispatchEvent(pointer("pointermove", { clientX: 60, clientY: 80 }));
  await nextTick();
  assert.deepEqual(crop(handle), { x: 60, y: 40, width: 320, height: 160 });
  area.dispatchEvent(pointer("pointermove", { clientX: 500, clientY: 500 }));
  await nextTick();
  assert.deepEqual(crop(handle), { x: 80, y: 40, width: 320, height: 160 });
  area.dispatchEvent(pointer("pointerup"));
  await nextTick();
  assert.equal(root.getAttribute("data-interaction"), "idle");
  assert.deepEqual(handle.wrapper.emitted("cropEnd"), [
    [{ x: 80, y: 40, width: 320, height: 160 }],
  ]);
  assert.equal(handle.wrapper.emitted("change")?.at(-1)?.[1], "move");

  area.dispatchEvent(pointer("pointerdown", { button: 2 }));
  await nextTick();
  assert.equal(root.getAttribute("data-interaction"), "idle");
  handle.unmount();
});

test("handles resize from their edge or corner, including aspect-locked crops", async () => {
  const handle = mountCropper();
  const root = handle.root();
  await load(root);
  const southEast = root.querySelector<HTMLElement>('[data-position="se"]');
  assert.ok(southEast);
  southEast.dispatchEvent(pointer("pointerdown", { clientX: 0, clientY: 0 }));
  await nextTick();
  assert.equal(root.getAttribute("data-interaction"), "resizing");
  southEast.dispatchEvent(pointer("pointermove", { clientX: -20, clientY: -10 }));
  southEast.dispatchEvent(pointer("pointerup"));
  await nextTick();
  assert.deepEqual(crop(handle), { x: 40, y: 20, width: 280, height: 140 });
  assert.equal(handle.wrapper.emitted("change")?.at(-1)?.[1], "resize");
  assert.equal(handle.wrapper.emitted("cropEnd")?.length, 1);
  handle.unmount();

  const locked = mountCropper({ aspectRatio: 1 });
  await load(locked.root());
  assert.deepEqual(crop(locked), { x: 120, y: 20, width: 160, height: 160 });
  const east = locked.root().querySelector<HTMLElement>('[data-position="e"]');
  east?.dispatchEvent(pointer("pointerdown", { clientX: 0, clientY: 0 }));
  east?.dispatchEvent(pointer("pointermove", { clientX: -20, clientY: 0 }));
  east?.dispatchEvent(pointer("pointerup"));
  await nextTick();
  assert.deepEqual(crop(locked), { x: 120, y: 40, width: 120, height: 120 });
  locked.unmount();
});

test("keyboard nudges, resizes, zooms, and rotates the focused crop area", async () => {
  const handle = mountCropper({ nudgeStep: 2 });
  const root = handle.root();
  await load(root);
  const area = part(root, "image-cropper-area");
  const key = async (init: KeyboardEventInit) => {
    const event = new KeyboardEvent("keydown", { bubbles: true, cancelable: true, ...init });
    area.dispatchEvent(event);
    await nextTick();
    return event.defaultPrevented;
  };

  assert.equal(await key({ key: "ArrowRight" }), true);
  assert.deepEqual(crop(handle), { x: 42, y: 20, width: 320, height: 160 });
  await key({ key: "ArrowUp", shiftKey: true });
  assert.deepEqual(crop(handle), { x: 42, y: 0, width: 320, height: 160 });
  await key({ key: "ArrowLeft", altKey: true });
  assert.deepEqual(crop(handle), { x: 42, y: 0, width: 318, height: 160 });
  await key({ key: "ArrowDown", ctrlKey: true, shiftKey: true });
  assert.deepEqual(crop(handle), { x: 42, y: 0, width: 318, height: 180 });
  assert.equal(handle.wrapper.emitted("change")?.at(-1)?.[1], "keyboard");

  await key({ key: "+" });
  assert.ok(Math.abs(handle.exposes<ImageCropperRootExpose>().zoom - 1.1) < 1e-9);
  await key({ key: "-" });
  assert.ok(Math.abs(handle.exposes<ImageCropperRootExpose>().zoom - 1) < 1e-9);
  await key({ key: "]" });
  assert.equal(handle.exposes<ImageCropperRootExpose>().rotation, 90);
  await key({ key: "[" });
  assert.equal(handle.exposes<ImageCropperRootExpose>().rotation, 0);
  assert.equal(await key({ key: "x" }), false);
  assert.ok((handle.wrapper.emitted("cropEnd")?.length ?? 0) >= 8);

  const inner = root.querySelector<HTMLElement>('[data-vize-ui="image-cropper-grid"]');
  const fromChild = new KeyboardEvent("keydown", {
    key: "ArrowRight",
    bubbles: true,
    cancelable: true,
  });
  inner?.dispatchEvent(fromChild);
  assert.equal(fromChild.defaultPrevented, false, "only the focused area handles keys");
  handle.unmount();
});

test("wheel zoom keeps the pointer anchor and background drags pan the zoomed view", async () => {
  const handle = mountCropper();
  const root = handle.root();
  await load(root);
  const viewport = part(root, "image-cropper-viewport");
  const image = part(root, "image-cropper-image");

  const wheel = new WheelEvent("wheel", { deltaY: -100, bubbles: true, cancelable: true });
  Object.defineProperty(wheel, "clientX", { value: 0 });
  Object.defineProperty(wheel, "clientY", { value: 100 });
  viewport.dispatchEvent(wheel);
  await nextTick();
  assert.equal(wheel.defaultPrevented, true);
  const zoom = handle.exposes<ImageCropperRootExpose>().zoom;
  assert.ok(Math.abs(zoom - 1.1) < 1e-9);
  assert.deepEqual(handle.wrapper.emitted("update:zoom"), [[zoom]]);
  assert.equal(image.style.left, "0px", "the left edge stays under the pointer");
  assert.equal(image.style.width, "220px");

  viewport.dispatchEvent(pointer("pointerdown", { clientX: 100, clientY: 100 }));
  await nextTick();
  assert.equal(root.getAttribute("data-interaction"), "panning");
  viewport.dispatchEvent(pointer("pointermove", { clientX: 90, clientY: 100 }));
  await nextTick();
  assert.equal(image.style.left, "-10px", `during ${root.getAttribute("data-interaction")}`);
  viewport.dispatchEvent(pointer("pointerup"));
  await nextTick();
  assert.equal(image.style.left, "-10px");
  assert.equal(root.getAttribute("data-interaction"), "idle");

  const flat = new WheelEvent("wheel", { deltaY: 0, bubbles: true, cancelable: true });
  viewport.dispatchEvent(flat);
  assert.equal(flat.defaultPrevented, false);
  handle.unmount();

  const noWheel = mountInteraction(ImageCropperRoot, {
    slots: {
      default: () =>
        h(ImageCropperViewport, { wheelZoom: false }, () =>
          h(ImageCropperImage, { src: "/photo.png", alt: "" }),
        ),
    },
  });
  await load(noWheel.root());
  const ignored = new WheelEvent("wheel", { deltaY: -1, bubbles: true, cancelable: true });
  part(noWheel.root(), "image-cropper-viewport").dispatchEvent(ignored);
  assert.equal(ignored.defaultPrevented, false);
  noWheel.unmount();
});

test("rotation swaps the bounds and re-fits the crop around its relative center", async () => {
  const handle = mountCropper();
  const root = handle.root();
  await load(root);
  const exposed = handle.exposes<ImageCropperRootExpose>();
  assert.equal(exposed.setRotation(-90), true);
  await nextTick();
  assert.equal(exposed.rotation, 270);
  assert.deepEqual(handle.wrapper.emitted("update:rotation"), [[270]]);
  assert.deepEqual(exposed.crop, { x: 0, y: 120, width: 200, height: 160 });
  assert.equal(handle.wrapper.emitted("change")?.at(-1)?.[1], "rotate");
  assert.equal(part(root, "image-cropper-image").style.transform, "rotate(270deg)");
  assert.equal(exposed.setRotation(270), false);
  handle.unmount();
});

test("controlled crop, zoom, and rotation win until the parent accepts them", async () => {
  const handle = mountCropper({
    modelValue: { x: 0, y: 0, width: 100, height: 100 },
    zoom: 2,
    rotation: 0,
  });
  await load(handle.root());
  const exposed = handle.exposes<ImageCropperRootExpose>();
  assert.deepEqual(exposed.crop, { x: 0, y: 0, width: 100, height: 100 });
  assert.equal(
    handle.wrapper.emitted("change"),
    undefined,
    "a controlled crop is not re-initialized",
  );
  assert.equal(exposed.setCrop({ x: 10, y: 0, width: 100, height: 100 }), true);
  assert.equal(exposed.crop?.x, 0);
  assert.equal(exposed.setZoom(3), true);
  assert.equal(exposed.zoom, 2);
  assert.equal(exposed.setZoom(99), true);
  assert.deepEqual(handle.wrapper.emitted("update:zoom"), [[3], [5]]);
  await handle.wrapper.setProps({ modelValue: { x: 10, y: 0, width: 100, height: 100 }, zoom: 9 });
  assert.equal(exposed.crop?.x, 10);
  assert.equal(exposed.zoom, 5, "zoom clamps to maxZoom");
  handle.unmount();
});

test("reset, localized messages, and disabled croppers", async () => {
  const handle = mountCropper({
    messages: {
      areaRoleDescription: "Zuschnitt",
      area: (area: CropArea) => `${area.width}×${area.height}`,
    },
  });
  const root = handle.root();
  await load(root);
  const area = part(root, "image-cropper-area");
  assert.equal(area.getAttribute("aria-roledescription"), "Zuschnitt");
  assert.equal(area.getAttribute("aria-label"), "320×160");
  const exposed = handle.exposes<ImageCropperRootExpose>();
  exposed.setZoom(2);
  exposed.setRotation(90);
  exposed.setCrop({ x: 0, y: 0, width: 10, height: 10 });
  exposed.reset();
  await nextTick();
  assert.equal(exposed.zoom, 1);
  assert.equal(exposed.rotation, 0);
  assert.deepEqual(exposed.crop, { x: 40, y: 20, width: 320, height: 160 });
  handle.unmount();

  const disabled = mountCropper({ disabled: true });
  await load(disabled.root());
  const disabledArea = part(disabled.root(), "image-cropper-area");
  assert.equal(disabledArea.getAttribute("tabindex"), "-1");
  assert.equal(disabledArea.getAttribute("aria-disabled"), "true");
  assert.equal(disabled.root().getAttribute("data-disabled"), "true");
  disabledArea.dispatchEvent(pointer("pointerdown", { clientX: 50, clientY: 70 }));
  disabledArea.dispatchEvent(pointer("pointermove", { clientX: 90, clientY: 90 }));
  disabledArea.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowRight", bubbles: true }));
  const wheel = new WheelEvent("wheel", { deltaY: -1, bubbles: true, cancelable: true });
  part(disabled.root(), "image-cropper-viewport").dispatchEvent(wheel);
  await nextTick();
  assert.deepEqual(crop(disabled), { x: 40, y: 20, width: 320, height: 160 });
  assert.equal(wheel.defaultPrevented, false);
  disabled.unmount();
});

test("images are sanitized, read when already complete, and reset the crop when replaced", async () => {
  const unsafe = mountCropper({}, { src: "javascript:alert(1)" });
  assert.equal(part(unsafe.root(), "image-cropper-image").getAttribute("src"), null);
  unsafe.unmount();
  const insecure = mountCropper(
    {},
    { src: "http://localhost/a.png", allowInsecure: true, crossOrigin: "anonymous" },
  );
  const insecureImage = part(insecure.root(), "image-cropper-image");
  assert.equal(insecureImage.getAttribute("src"), "http://localhost/a.png");
  assert.equal(insecureImage.getAttribute("crossorigin"), "anonymous");
  insecure.unmount();

  override(HTMLImageElement.prototype, "complete", { get: () => true });
  const cached = mountCropper();
  await nextTick();
  await nextTick();
  assert.equal(cached.root().getAttribute("data-ready"), "true");
  const exposed = cached.exposes<ImageCropperRootExpose>();
  exposed.setCrop({ x: 0, y: 0, width: 50, height: 50 });
  natural = { width: 100, height: 100 };
  await load(cached.root());
  assert.deepEqual(exposed.crop, { x: 10, y: 10, width: 80, height: 80 });
  assert.equal(cached.wrapper.emitted("change")?.at(-1)?.[1], "init");
  cached.unmount();
});

test("the grid renders division lines with offsets", async () => {
  const handle = mountCropper();
  const lines = [
    ...handle.root().querySelectorAll<HTMLElement>('[data-vize-ui="image-cropper-grid-line"]'),
  ];
  assert.equal(lines.length, 4);
  assert.deepEqual(
    lines.map((line) => [
      line.getAttribute("data-axis"),
      line.style.getPropertyValue("--vize-ui-image-cropper-grid-offset"),
    ]),
    [
      ["x", "33.333%"],
      ["x", "66.667%"],
      ["y", "33.333%"],
      ["y", "66.667%"],
    ],
  );
  assert.equal(part(handle.root(), "image-cropper-grid").getAttribute("aria-hidden"), "true");
  handle.unmount();
});

test("exports the loaded crop through the canvas helper", async () => {
  const handle = mountCropper();
  const exposed = handle.exposes<ImageCropperRootExpose>();
  await assert.rejects(exposed.toBlob(), /VIZE_UI_IMAGE_CROPPER_IMAGE_LOAD/);
  await load(handle.root());
  override(HTMLImageElement.prototype, "complete", { get: () => true });
  const createElement = document.createElement.bind(document);
  const draws: string[] = [];
  document.createElement = ((tag: string) => {
    const element = createElement(tag);
    if (tag === "canvas") {
      const canvas = element as HTMLCanvasElement;
      Object.defineProperty(canvas, "getContext", {
        value: () => ({
          scale: () => undefined,
          translate: (x: number, y: number) => draws.push(`${x},${y}`),
          rotate: () => undefined,
          drawImage: () => undefined,
        }),
      });
      Object.defineProperty(canvas, "toDataURL", {
        value: () => `data:${canvas.width}x${canvas.height}`,
      });
      Object.defineProperty(canvas, "toBlob", {
        value: (callback: (blob: Blob) => void) => callback(new Blob(["x"])),
      });
    }
    return element;
  }) as typeof document.createElement;
  try {
    assert.equal(await exposed.toDataUrl(), "data:320x160");
    assert.ok((await exposed.toBlob({ type: "image/webp" })) instanceof Blob);
    assert.equal(draws[0], "160,80");
  } finally {
    document.createElement = createElement as typeof document.createElement;
  }
  handle.unmount();
});

test("compound parts require a matching root provider", () => {
  for (const [component, props] of [
    [ImageCropperViewport, {}],
    [ImageCropperImage, { src: "/a.png", alt: "" }],
    [ImageCropperArea, {}],
    [ImageCropperHandle, { position: "n" }],
    [ImageCropperGrid, {}],
  ] as const) {
    assert.throws(
      () => mountInteraction(component, { props }),
      /VIZE_UI_CONTEXT_MISSING: ImageCropper requires a matching provider/,
    );
  }
});
