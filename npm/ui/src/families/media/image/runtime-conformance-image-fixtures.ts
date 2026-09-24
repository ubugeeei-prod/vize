import assert from "node:assert/strict";

import { h } from "vue";

import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";
import { ImageContent, ImageFallback, ImagePlaceholder, ImageRoot } from "./image.ts";

export const imageRuntimeFixtures: readonly RuntimeFixture[] = [
  {
    name: "image",
    sourceFile: "families/media/image/image-root.vue",
    render: () =>
      h(ImageRoot, { src: ["/gallery/cover.avif", "/gallery/cover.jpg"] }, () => [
        h(ImageContent, { alt: "Gallery cover", width: 320, height: 240 }),
        h(ImagePlaceholder, null, () => "Loading cover"),
        h(ImageFallback, null, () => "GC"),
      ]),
    assertServerMarkup(html) {
      assert.match(html, /data-vize-ui="image-root"/);
      assert.match(html, /data-status="loading"/);
      assert.match(html, /<img alt="Gallery cover"/);
      assert.match(html, /src="\/gallery\/cover\.avif"/);
      assert.match(html, /data-vize-ui="image-placeholder"/);
      assert.doesNotMatch(html, /image-fallback/);
    },
    assertHydratedDom(host) {
      const root = host.querySelector('[data-vize-ui="image-root"]');
      const image = host.querySelector('[data-vize-ui="image-content"]');
      assert.ok(root instanceof HTMLSpanElement);
      assert.ok(image instanceof HTMLImageElement);
      assert.equal(image.alt, "Gallery cover");
      assert.equal(image.getAttribute("src"), "/gallery/cover.avif");
    },
  },
  {
    name: "image-content",
    sourceFile: "families/media/image/image-content.vue",
    render: () =>
      h(ImageRoot, { src: "/gallery/hero.webp" }, () =>
        h(ImageContent, { alt: "", loading: "eager", fetchPriority: "high" }),
      ),
    assertServerMarkup(html) {
      assert.match(html, /<img alt(?:="")? /);
      assert.match(html, /loading="eager"/);
      assert.match(html, /fetchpriority="high"/);
      assert.match(html, /data-vize-ui="image-content"/);
      assert.match(html, /data-candidate="0"/);
    },
    assertHydratedDom(host) {
      const image = host.querySelector('[data-vize-ui="image-content"]');
      assert.ok(image instanceof HTMLImageElement);
      assert.equal(image.getAttribute("loading"), "eager");
    },
  },
  {
    name: "image-fallback",
    sourceFile: "families/media/image/image-fallback.vue",
    render: () =>
      h(ImageRoot, { src: "javascript:alert(1)" }, () => [
        h(ImageContent, { alt: "Unsafe" }),
        h(ImageFallback, null, () => "No image"),
      ]),
    assertServerMarkup(html) {
      assert.match(html, /data-status="error"/);
      assert.match(html, /data-vize-ui="image-fallback"/);
      assert.match(html, /No image/);
      assert.doesNotMatch(html, /<img/);
    },
    assertHydratedDom(host) {
      const fallback = host.querySelector('[data-vize-ui="image-fallback"]');
      assert.ok(fallback instanceof HTMLSpanElement);
      assert.equal(fallback.textContent, "No image");
      assert.equal(host.querySelector("img"), null);
    },
  },
  {
    name: "image-placeholder",
    sourceFile: "families/media/image/image-placeholder.vue",
    render: () =>
      h(ImageRoot, { src: "/gallery/deferred.png", defer: true }, () => [
        h(ImageContent, { alt: "Deferred" }),
        h(ImagePlaceholder, null, () => "Waiting"),
      ]),
    assertServerMarkup(html) {
      assert.match(html, /data-status="idle"/);
      assert.match(html, /data-vize-ui="image-placeholder"/);
      assert.match(html, /aria-hidden="true"/);
      assert.doesNotMatch(html, /src=/);
    },
    assertHydratedDom(host) {
      const placeholder = host.querySelector('[data-vize-ui="image-placeholder"]');
      assert.ok(placeholder instanceof HTMLSpanElement);
      assert.equal(placeholder.getAttribute("aria-hidden"), "true");
    },
  },
];
