import assert from "node:assert/strict";

import { h } from "vue";

import AnnouncerProvider from "./announcer-provider.vue";
import LiveRegion from "./live-region.vue";
import LocaleProvider from "./locale-provider.vue";
import Portal from "./portal.vue";
import PositionerArrow from "./positioner-arrow.vue";
import Positioner from "./positioner.vue";
import Presence from "./presence.vue";
import Transition from "./transition.vue";
import type { RuntimeFixture } from "./runtime-conformance-fixtures.ts";

export const overlayRuntimeFixtures: readonly RuntimeFixture[] = [
  {
    name: "announcer-provider",
    sourceFile: "announcer-provider.vue",
    render: () =>
      h(AnnouncerProvider, null, {
        default: () => "Content",
      }),
    assertServerMarkup(html) {
      assert.match(html, /data-vize-ui="announcer"/);
      assert.match(html, /data-vize-announcer="owner"/);
      assert.match(html, /aria-live="polite"/);
      assert.match(html, /aria-live="assertive"/);
      assert.match(html, /Content/);
    },
    assertHydratedDom(host) {
      const regions = host.querySelectorAll('[data-vize-ui="announcer-region"]');
      assert.equal(regions.length, 2);
      assert.equal(regions[0]?.getAttribute("role"), "status");
      assert.equal(regions[1]?.getAttribute("role"), "alert");
    },
  },
  {
    name: "live-region",
    sourceFile: "live-region.vue",
    render: () => h(LiveRegion),
    assertServerMarkup(html) {
      assert.match(html, /data-vize-ui="live-region"/);
      assert.match(html, /aria-live="polite"/);
      assert.match(html, /role="status"/);
    },
    assertHydratedDom(host) {
      const region = host.querySelector('[data-vize-ui="live-region"]');
      assert.ok(region instanceof HTMLElement);
      assert.equal(region.getAttribute("aria-live"), "polite");
    },
  },
  {
    name: "locale-provider",
    sourceFile: "locale-provider.vue",
    render: () =>
      h(
        LocaleProvider,
        { locale: "ja-JP", direction: "ltr" },
        {
          default: () => "本文",
        },
      ),
    assertServerMarkup(html) {
      assert.match(html, /data-vize-ui="locale"/);
      assert.match(html, /lang="ja-JP"/);
      assert.match(html, /dir="ltr"/);
      assert.match(html, /本文/);
    },
    assertHydratedDom(host) {
      const locale = host.querySelector('[data-vize-ui="locale"]');
      assert.ok(locale instanceof HTMLElement);
      assert.equal(locale.getAttribute("lang"), "ja-JP");
      assert.equal(locale.textContent, "本文");
    },
  },
  {
    name: "portal",
    sourceFile: "portal.vue",
    render: () =>
      h(
        Portal,
        { disabled: true },
        {
          default: () => "Portalled",
        },
      ),
    assertServerMarkup(html) {
      assert.match(html, /data-vize-ui="portal-host"/);
      assert.match(html, /data-vize-ui="portal"/);
      assert.match(html, /Portalled/);
    },
    assertHydratedDom(host) {
      const portal = host.querySelector('[data-vize-ui="portal"]');
      assert.ok(portal instanceof HTMLElement);
      assert.equal(portal.textContent, "Portalled");
    },
  },
  {
    name: "presence",
    sourceFile: "presence.vue",
    render: () =>
      h(
        Presence,
        { present: true },
        {
          default: () => "Overlay",
        },
      ),
    assertServerMarkup(html) {
      assert.match(html, /data-vize-ui="presence-host"/);
      assert.match(html, /data-vize-ui="presence"/);
      assert.match(html, /data-vize-presence="present"/);
      assert.match(html, /Overlay/);
    },
    assertHydratedDom(host) {
      const presence = host.querySelector('[data-vize-ui="presence"]');
      assert.ok(presence instanceof HTMLElement);
      assert.equal(presence.getAttribute("data-vize-presence"), "present");
      assert.equal(presence.textContent, "Overlay");
    },
  },
  {
    name: "positioner",
    sourceFile: "positioner.vue",
    render: () =>
      h(Positioner, null, {
        default: () => "Menu",
      }),
    assertServerMarkup(html) {
      assert.match(html, /data-vize-ui="positioner"/);
      assert.match(html, /data-vize-positioner-ready="false"/);
      assert.match(html, /Menu/);
    },
    assertHydratedDom(host) {
      const positioner = host.querySelector('[data-vize-ui="positioner"]');
      assert.ok(positioner instanceof HTMLElement);
      assert.equal(positioner.getAttribute("data-vize-placement"), "bottom");
      assert.equal(positioner.textContent, "Menu");
    },
  },
  {
    name: "positioner-arrow",
    sourceFile: "positioner-arrow.vue",
    render: () =>
      h(Positioner, null, {
        default: () => [h(PositionerArrow), "Menu"],
      }),
    assertServerMarkup(html) {
      assert.match(html, /data-vize-ui="positioner-arrow"/);
      assert.match(html, /Menu/);
    },
    assertHydratedDom(host) {
      const arrow = host.querySelector('[data-vize-ui="positioner-arrow"]');
      assert.ok(arrow instanceof HTMLElement);
    },
  },
  {
    name: "transition",
    sourceFile: "transition.vue",
    render: () =>
      h(
        Transition,
        { present: true },
        {
          default: () => "Overlay",
        },
      ),
    assertServerMarkup(html) {
      assert.match(html, /data-vize-ui="transition-host"/);
      assert.match(html, /data-vize-ui="transition"/);
      assert.match(html, /data-vize-transition="present"/);
      assert.match(html, /Overlay/);
    },
    assertHydratedDom(host) {
      const transition = host.querySelector('[data-vize-ui="transition"]');
      assert.ok(transition instanceof HTMLElement);
      assert.equal(transition.getAttribute("data-vize-transition"), "present");
      assert.equal(transition.textContent, "Overlay");
    },
  },
];
