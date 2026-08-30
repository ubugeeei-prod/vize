import assert from "node:assert/strict";

import { h } from "vue";

import AnnouncerProvider from "./announcer-provider.vue";
import { dialogRuntimeFixtures } from "./runtime-conformance-dialog-fixtures.ts";
import LiveRegion from "./live-region.vue";
import LocaleProvider from "./locale-provider.vue";
import Portal from "./portal.vue";
import PositionerArrow from "./positioner-arrow.vue";
import Positioner from "./positioner.vue";
import {
  PopoverArrow,
  PopoverContent,
  PopoverRoot,
  PopoverTrigger,
} from "./families/overlays/popover/popover.ts";
import Presence from "./presence.vue";
import Transition from "./transition.vue";
import { TooltipContent, TooltipRoot, TooltipTrigger } from "./tooltip.ts";
import type { RuntimeFixture } from "./runtime-conformance-fixtures.ts";

function tooltip(children: () => unknown): ReturnType<typeof h> {
  return h(TooltipRoot, { defaultOpen: true, delayDuration: 0, id: "runtime-tooltip" }, children);
}

function popover(children: () => unknown): ReturnType<typeof h> {
  return h(PopoverRoot, { defaultOpen: true, id: "runtime-popover" }, children);
}

function assertTooltipRoot(host: HTMLElement): void {
  const root = host.querySelector('[data-vize-ui="tooltip-root"]');
  assert.ok(root instanceof HTMLElement);
  assert.equal(root.getAttribute("data-state"), "open");
}

function assertPopoverRoot(host: HTMLElement): void {
  const root = host.querySelector('[data-vize-ui="popover-root"]');
  assert.ok(root instanceof HTMLElement);
  assert.equal(root.getAttribute("data-state"), "open");
}

export const overlayRuntimeFixtures: readonly RuntimeFixture[] = [
  ...dialogRuntimeFixtures,
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
    name: "popover-root",
    sourceFile: "families/overlays/popover/popover-root.vue",
    render: () => popover(() => "Popover"),
    assertServerMarkup(html) {
      assert.match(html, /id="runtime-popover"/);
      assert.match(html, /data-vize-ui="popover-root"/);
      assert.match(html, /data-state="open"/);
    },
    assertHydratedDom: assertPopoverRoot,
  },
  {
    name: "popover-trigger",
    sourceFile: "families/overlays/popover/popover-trigger.vue",
    render: () => popover(() => h(PopoverTrigger, null, () => "Open filters")),
    assertServerMarkup(html) {
      assert.match(html, /data-vize-ui="popover-trigger"/);
      assert.match(html, /aria-controls="runtime-popover-content"/);
      assert.match(html, /aria-expanded="true"/);
      assert.match(html, /type="button"/);
    },
    assertHydratedDom(host) {
      const trigger = host.querySelector('[data-vize-ui="popover-trigger"]');
      assert.ok(trigger instanceof HTMLButtonElement);
      assert.equal(trigger.getAttribute("aria-haspopup"), "dialog");
    },
  },
  {
    name: "popover-content",
    sourceFile: "families/overlays/popover/popover-content.vue",
    render: () =>
      popover(() => [
        h(PopoverTrigger, null, () => "Open filters"),
        h(PopoverContent, { portalDisabled: true, placement: "bottom-start" }, () => "Filters"),
      ]),
    assertServerMarkup(html) {
      assert.match(html, /data-vize-ui="popover-content"/);
      assert.match(html, /role="dialog"/);
      assert.match(html, /data-side="bottom"/);
      assert.match(html, /data-align="start"/);
      assert.match(html, /data-vize-ui="positioner"/);
      assert.match(html, /data-vize-dismissable-layer/);
    },
    assertHydratedDom(host) {
      const content = host.querySelector('[data-vize-ui="popover-content"]');
      assert.ok(content instanceof HTMLElement);
      assert.equal(content.getAttribute("role"), "dialog");
      assert.equal(content.textContent, "Filters");
    },
  },
  {
    name: "popover-arrow",
    sourceFile: "families/overlays/popover/popover-arrow.vue",
    render: () =>
      popover(() => [
        h(PopoverTrigger, null, () => "Open filters"),
        h(PopoverContent, { portalDisabled: true }, () => [
          h(PopoverArrow, null, () => "Arrow"),
          "Filters",
        ]),
      ]),
    assertServerMarkup(html) {
      assert.match(html, /data-vize-ui="popover-arrow"/);
      assert.match(html, /part="arrow"/);
      assert.match(html, /Arrow/);
    },
    assertHydratedDom(host) {
      const arrow = host.querySelector('[data-vize-ui="popover-arrow"]');
      assert.ok(arrow instanceof HTMLElement);
      assert.equal(arrow.textContent, "Arrow");
    },
  },
  {
    name: "tooltip-root",
    sourceFile: "tooltip-root.vue",
    render: () => tooltip(() => "Tip"),
    assertServerMarkup(html) {
      assert.match(html, /id="runtime-tooltip"/);
      assert.match(html, /data-vize-ui="tooltip-root"/);
      assert.match(html, /data-state="open"/);
    },
    assertHydratedDom: assertTooltipRoot,
  },
  {
    name: "tooltip-trigger",
    sourceFile: "tooltip-trigger.vue",
    render: () => tooltip(() => h(TooltipTrigger, null, () => "More info")),
    assertServerMarkup(html) {
      assert.match(html, /data-vize-ui="tooltip-trigger"/);
      assert.match(html, /aria-describedby="runtime-tooltip-content"/);
      assert.match(html, /type="button"/);
    },
    assertHydratedDom(host) {
      const trigger = host.querySelector('[data-vize-ui="tooltip-trigger"]');
      assert.ok(trigger instanceof HTMLButtonElement);
      assert.equal(trigger.getAttribute("data-state"), "open");
    },
  },
  {
    name: "tooltip-content",
    sourceFile: "tooltip-content.vue",
    render: () =>
      tooltip(() => [
        h(TooltipTrigger, null, () => "More info"),
        h(TooltipContent, { portalDisabled: true }, () => "Extra context"),
      ]),
    assertServerMarkup(html) {
      assert.match(html, /data-vize-ui="tooltip-content"/);
      assert.match(html, /role="tooltip"/);
      assert.match(html, /data-vize-ui="positioner"/);
      assert.match(html, /data-vize-dismissable-layer/);
    },
    assertHydratedDom(host) {
      const content = host.querySelector('[data-vize-ui="tooltip-content"]');
      assert.ok(content instanceof HTMLElement);
      assert.equal(content.getAttribute("role"), "tooltip");
      assert.equal(content.textContent, "Extra context");
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
