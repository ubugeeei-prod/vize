import assert from "node:assert/strict";

import { h } from "vue";

import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";
import {
  CarouselAutoplayToggle,
  CarouselIndicator,
  CarouselIndicatorGroup,
  CarouselNext,
  CarouselPrevious,
  CarouselRoot,
  CarouselSlide,
  CarouselViewport,
} from "./carousel.ts";

function carousel(id: string) {
  return h(CarouselRoot, { id, slideCount: 2, ariaLabel: "Products" }, () => [
    h(CarouselAutoplayToggle, null, () => "Start rotation"),
    h(CarouselPrevious, null, () => "Previous"),
    h(CarouselNext, null, () => "Next"),
    h(CarouselViewport, null, () => [
      h(CarouselSlide, { index: 0 }, () => "Lamp"),
      h(CarouselSlide, { index: 1 }, () => "Chair"),
    ]),
    h(CarouselIndicatorGroup, { ariaLabel: "Choose product" }, () => [
      h(CarouselIndicator, { index: 0 }),
      h(CarouselIndicator, { index: 1 }),
    ]),
  ]);
}

function part(host: HTMLElement, name: string): Element {
  const element = host.querySelector(`[data-vize-ui="${name}"]`);
  assert.ok(element, `${name} must render`);
  return element;
}

export const carouselRuntimeFixtures: readonly RuntimeFixture[] = [
  {
    name: "carousel",
    sourceFile: "families/media/carousel/carousel-root.vue",
    render: () => carousel("products"),
    assertServerMarkup(html) {
      assert.match(html, /<section id="products" aria-roledescription="carousel"/);
      assert.match(html, /aria-label="Products"/);
      assert.match(html, /data-autoplay="stopped"/);
    },
    assertHydratedDom(host) {
      const root = part(host, "carousel-root");
      assert.equal(root.id, "products");
      assert.equal(root.getAttribute("data-index"), "0");
    },
  },
  {
    name: "carousel-viewport",
    sourceFile: "families/media/carousel/carousel-viewport.vue",
    render: () => carousel("products-viewport"),
    assertServerMarkup(html) {
      assert.match(html, /id="products-viewport-viewport" tabindex="0" aria-live="polite"/);
    },
    assertHydratedDom(host) {
      assert.equal(part(host, "carousel-viewport").getAttribute("tabindex"), "0");
    },
  },
  {
    name: "carousel-slide",
    sourceFile: "families/media/carousel/carousel-slide.vue",
    render: () => carousel("products-slide"),
    assertServerMarkup(html) {
      assert.match(html, /role="group" aria-roledescription="slide" aria-label="1 of 2"/);
      assert.match(html, /aria-label="2 of 2"/);
    },
    assertHydratedDom(host) {
      assert.equal(part(host, "carousel-slide").getAttribute("data-state"), "active");
    },
  },
  {
    name: "carousel-previous",
    sourceFile: "families/media/carousel/carousel-previous.vue",
    render: () => carousel("products-previous"),
    assertServerMarkup(html) {
      assert.match(
        html,
        /<button type="button" disabled aria-controls="products-previous-viewport"/,
      );
    },
    assertHydratedDom(host) {
      const previous = part(host, "carousel-previous");
      assert.ok(previous instanceof HTMLButtonElement);
      assert.equal(previous.disabled, true);
    },
  },
  {
    name: "carousel-next",
    sourceFile: "families/media/carousel/carousel-next.vue",
    render: () => carousel("products-next"),
    assertServerMarkup(html) {
      assert.match(html, /aria-controls="products-next-viewport" data-vize-ui="carousel-next"/);
    },
    assertHydratedDom(host) {
      const next = part(host, "carousel-next");
      assert.ok(next instanceof HTMLButtonElement);
      assert.equal(next.disabled, false);
    },
  },
  {
    name: "carousel-indicator-group",
    sourceFile: "families/media/carousel/carousel-indicator-group.vue",
    render: () => carousel("products-group"),
    assertServerMarkup(html) {
      assert.match(
        html,
        /role="tablist" aria-label="Choose product" aria-orientation="horizontal"/,
      );
    },
    assertHydratedDom(host) {
      assert.equal(part(host, "carousel-indicator-group").getAttribute("role"), "tablist");
    },
  },
  {
    name: "carousel-indicator",
    sourceFile: "families/media/carousel/carousel-indicator.vue",
    render: () => carousel("products-indicator"),
    assertServerMarkup(html) {
      assert.match(html, /role="tab" tabindex="0" aria-label="Slide 1" aria-selected="true"/);
      assert.match(html, /aria-controls="products-indicator-slide-1"/);
    },
    assertHydratedDom(host) {
      assert.equal(part(host, "carousel-indicator").getAttribute("aria-selected"), "true");
    },
  },
  {
    name: "carousel-autoplay-toggle",
    sourceFile: "families/media/carousel/carousel-autoplay-toggle.vue",
    render: () => carousel("products-toggle"),
    assertServerMarkup(html) {
      assert.match(html, /data-vize-ui="carousel-autoplay-toggle"[^>]*data-state="stopped"/);
    },
    assertHydratedDom(host) {
      assert.equal(part(host, "carousel-autoplay-toggle").getAttribute("data-state"), "stopped");
    },
  },
];
