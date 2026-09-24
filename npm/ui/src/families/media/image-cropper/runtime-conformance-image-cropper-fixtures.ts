import assert from "node:assert/strict";

import { h } from "vue";

import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";
import {
  ImageCropperArea,
  ImageCropperGrid,
  ImageCropperHandle,
  ImageCropperImage,
  ImageCropperRoot,
  ImageCropperViewport,
} from "./image-cropper.ts";

function cropper(id: string) {
  return h(ImageCropperRoot, { id, aspectRatio: 16 / 9 }, () =>
    h(ImageCropperViewport, null, () => [
      h(ImageCropperImage, { src: "/banner.jpg", alt: "Banner" }),
      h(ImageCropperArea, null, () => [
        h(ImageCropperGrid),
        h(ImageCropperHandle, { position: "nw" }),
        h(ImageCropperHandle, { position: "se" }),
      ]),
    ]),
  );
}

function part(host: HTMLElement, name: string): Element {
  const element = host.querySelector(`[data-vize-ui="${name}"]`);
  assert.ok(element, `${name} must render`);
  return element;
}

export const imageCropperRuntimeFixtures: readonly RuntimeFixture[] = [
  {
    name: "image-cropper",
    sourceFile: "families/media/image-cropper/image-cropper-root.vue",
    render: () => cropper("banner-crop"),
    assertServerMarkup(html) {
      assert.match(html, /<div id="banner-crop" data-vize-ui="image-cropper-root"/);
      assert.match(html, /data-interaction="idle"/);
    },
    assertHydratedDom(host) {
      assert.equal(part(host, "image-cropper-root").id, "banner-crop");
    },
  },
  {
    name: "image-cropper-viewport",
    sourceFile: "families/media/image-cropper/image-cropper-viewport.vue",
    render: () => cropper("banner-viewport"),
    assertServerMarkup(html) {
      assert.match(
        html,
        /data-vize-ui="image-cropper-viewport"[^>]*overflow:hidden;position:relative;touch-action:none;/,
      );
    },
    assertHydratedDom(host) {
      const viewport = part(host, "image-cropper-viewport");
      assert.ok(viewport instanceof HTMLElement);
      assert.equal(viewport.style.overflow, "hidden");
    },
  },
  {
    name: "image-cropper-image",
    sourceFile: "families/media/image-cropper/image-cropper-image.vue",
    render: () => cropper("banner-image"),
    assertServerMarkup(html) {
      assert.match(
        html,
        /<img alt="Banner" src="\/banner\.jpg" draggable="false" data-vize-ui="image-cropper-image"/,
      );
    },
    assertHydratedDom(host) {
      assert.ok(part(host, "image-cropper-image") instanceof HTMLImageElement);
    },
  },
  {
    name: "image-cropper-area",
    sourceFile: "families/media/image-cropper/image-cropper-area.vue",
    render: () => cropper("banner-area"),
    assertServerMarkup(html) {
      assert.match(
        html,
        /role="group" tabindex="0" aria-roledescription="crop area" data-vize-ui="image-cropper-area"/,
      );
    },
    assertHydratedDom(host) {
      assert.equal(part(host, "image-cropper-area").getAttribute("tabindex"), "0");
    },
  },
  {
    name: "image-cropper-handle",
    sourceFile: "families/media/image-cropper/image-cropper-handle.vue",
    render: () => cropper("banner-handle"),
    assertServerMarkup(html) {
      assert.match(
        html,
        /aria-hidden="true" data-vize-ui="image-cropper-handle" part="handle" data-position="nw"/,
      );
    },
    assertHydratedDom(host) {
      assert.equal(part(host, "image-cropper-handle").getAttribute("data-position"), "nw");
    },
  },
  {
    name: "image-cropper-grid",
    sourceFile: "families/media/image-cropper/image-cropper-grid.vue",
    render: () => cropper("banner-grid"),
    assertServerMarkup(html) {
      assert.match(html, /aria-hidden="true" data-vize-ui="image-cropper-grid"/);
      assert.match(html, /--vize-ui-image-cropper-grid-offset:33\.333%/);
    },
    assertHydratedDom(host) {
      assert.equal(host.querySelectorAll('[data-vize-ui="image-cropper-grid-line"]').length, 4);
    },
  },
];
