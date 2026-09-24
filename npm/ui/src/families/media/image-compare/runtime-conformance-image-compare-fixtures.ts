import assert from "node:assert/strict";

import { h } from "vue";

import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";
import {
  ImageCompareAfter,
  ImageCompareBefore,
  ImageCompareHandle,
  ImageCompareLabel,
  ImageCompareRoot,
} from "./image-compare.ts";

function compare(id: string) {
  return h(ImageCompareRoot, { id, defaultValue: 60 }, () => [
    h(ImageCompareBefore, null, () => h("img", { alt: "Retouched", src: "/retouched.jpg" })),
    h(ImageCompareAfter, null, () => h("img", { alt: "Original", src: "/original.jpg" })),
    h(ImageCompareLabel, { side: "before" }, () => "Retouched"),
    h(ImageCompareLabel, { side: "after" }, () => "Original"),
    h(ImageCompareHandle),
  ]);
}

function part(host: HTMLElement, name: string): Element {
  const element = host.querySelector(`[data-vize-ui="${name}"]`);
  assert.ok(element, `${name} must render`);
  return element;
}

export const imageCompareRuntimeFixtures: readonly RuntimeFixture[] = [
  {
    name: "image-compare",
    sourceFile: "families/media/image-compare/image-compare-root.vue",
    render: () => compare("compare-root"),
    assertServerMarkup(html) {
      assert.match(html, /id="compare-root"/);
      assert.match(html, /data-vize-ui="image-compare-root"/);
      assert.match(html, /--vize-ui-image-compare-position:60%/);
    },
    assertHydratedDom(host) {
      assert.equal(part(host, "image-compare-root").getAttribute("data-state"), "idle");
    },
  },
  {
    name: "image-compare-before",
    sourceFile: "families/media/image-compare/image-compare-before.vue",
    render: () => compare("compare-before"),
    assertServerMarkup(html) {
      assert.match(html, /data-vize-ui="image-compare-before" part="before" data-side="before"/);
    },
    assertHydratedDom(host) {
      assert.ok(part(host, "image-compare-before").querySelector("img"));
    },
  },
  {
    name: "image-compare-after",
    sourceFile: "families/media/image-compare/image-compare-after.vue",
    render: () => compare("compare-after"),
    assertServerMarkup(html) {
      assert.match(html, /data-vize-ui="image-compare-after" part="after" data-side="after"/);
    },
    assertHydratedDom(host) {
      assert.ok(part(host, "image-compare-after").querySelector("img"));
    },
  },
  {
    name: "image-compare-label",
    sourceFile: "families/media/image-compare/image-compare-label.vue",
    render: () => compare("compare-label"),
    assertServerMarkup(html) {
      assert.match(html, /data-vize-ui="image-compare-label" part="label" data-side="after"/);
    },
    assertHydratedDom(host) {
      assert.equal(part(host, "image-compare-label").textContent, "Retouched");
    },
  },
  {
    name: "image-compare-handle",
    sourceFile: "families/media/image-compare/image-compare-handle.vue",
    render: () => compare("compare-handle"),
    assertServerMarkup(html) {
      assert.match(html, /id="compare-handle-handle" role="slider" tabindex="0"/);
      assert.match(html, /aria-valuenow="60"/);
    },
    assertHydratedDom(host) {
      const handle = part(host, "image-compare-handle");
      assert.equal(handle.getAttribute("role"), "slider");
      assert.equal(handle.getAttribute("aria-valuetext"), "60%");
    },
  },
];
