import assert from "node:assert/strict";

import { h } from "vue";

import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";
import {
  HotspotArea,
  HotspotContent,
  HotspotImage,
  HotspotMarker,
  HotspotRoot,
} from "./hotspot.ts";

function hotspots(id: string) {
  return h(HotspotRoot, { id, defaultActive: "chair" }, () => [
    h(HotspotImage, { src: "/studio.jpg", alt: "Studio" }),
    h(HotspotMarker, { id: "chair", x: 30, y: 60, label: "Chair" }, () =>
      h(HotspotContent, { portalDisabled: true }, () => "Oak chair"),
    ),
    h(HotspotArea, {
      id: "rug",
      label: "Rug",
      shape: { type: "circle", cx: 50, cy: 80, r: 10 },
    }),
  ]);
}

function part(host: HTMLElement, name: string): Element {
  const element = host.querySelector(`[data-vize-ui="${name}"]`);
  assert.ok(element, `${name} must render`);
  return element;
}

export const hotspotRuntimeFixtures: readonly RuntimeFixture[] = [
  {
    name: "hotspot",
    sourceFile: "families/media/hotspot/hotspot-root.vue",
    render: () => hotspots("hotspot-root"),
    assertServerMarkup(html) {
      assert.match(html, /id="hotspot-root" data-vize-ui="hotspot-root"/);
      assert.match(html, /data-active="chair"/);
    },
    assertHydratedDom(host) {
      assert.equal(part(host, "hotspot-root").getAttribute("data-state"), "open");
    },
  },
  {
    name: "hotspot-image",
    sourceFile: "families/media/hotspot/hotspot-image.vue",
    render: () => hotspots("hotspot-image"),
    assertServerMarkup(html) {
      assert.match(html, /<img src="\/studio\.jpg" alt="Studio"/);
    },
    assertHydratedDom(host) {
      assert.equal(part(host, "hotspot-image").getAttribute("alt"), "Studio");
    },
  },
  {
    name: "hotspot-marker",
    sourceFile: "families/media/hotspot/hotspot-marker.vue",
    render: () => hotspots("hotspot-marker"),
    assertServerMarkup(html) {
      assert.match(html, /data-vize-ui="hotspot-marker" part="marker" data-state="open"/);
      assert.match(html, /aria-label="Chair"/);
    },
    assertHydratedDom(host) {
      const marker = part(host, "hotspot-marker");
      assert.ok(marker instanceof HTMLElement);
      assert.equal(marker.style.getPropertyValue("--vize-ui-hotspot-x"), "30%");
    },
  },
  {
    name: "hotspot-content",
    sourceFile: "families/media/hotspot/hotspot-content.vue",
    render: () => hotspots("hotspot-content"),
    assertServerMarkup(html) {
      assert.match(html, /data-hotspot-id="chair"/);
      assert.match(html, /Oak chair/);
    },
    assertHydratedDom(host) {
      assert.ok(host.querySelector('[data-hotspot-id="chair"]'));
    },
  },
  {
    name: "hotspot-area",
    sourceFile: "families/media/hotspot/hotspot-area.vue",
    render: () => hotspots("hotspot-area"),
    assertServerMarkup(html) {
      assert.match(
        html,
        /data-vize-ui="hotspot-area" part="area" data-id="rug" data-shape="circle"/,
      );
      assert.match(html, /<circle cx="50" cy="80" r="10"/);
    },
    assertHydratedDom(host) {
      assert.equal(part(host, "hotspot-area-button").getAttribute("role"), "button");
    },
  },
];
