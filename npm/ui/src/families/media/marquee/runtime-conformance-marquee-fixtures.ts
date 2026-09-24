import assert from "node:assert/strict";

import { h } from "vue";

import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";
import { MarqueeContent, MarqueePauseButton, MarqueeRoot } from "./marquee.ts";

function marquee(id: string) {
  return h(MarqueeRoot, { id, ariaLabel: "Announcements", direction: "left" }, () => [
    h(MarqueePauseButton),
    h(MarqueeContent, null, () => "Maintenance window at 02:00 UTC"),
  ]);
}

export const marqueeRuntimeFixtures: readonly RuntimeFixture[] = [
  {
    name: "marquee",
    sourceFile: "families/media/marquee/marquee-root.vue",
    render: () => marquee("announcements"),
    assertServerMarkup(html) {
      assert.match(html, /^<div id="announcements" role="marquee"/);
      assert.match(html, /data-vize-ui="marquee-root"/);
    },
    assertHydratedDom(host) {
      const root = host.querySelector('[data-vize-ui="marquee-root"]');
      assert.ok(root instanceof HTMLDivElement);
      assert.equal(root.getAttribute("role"), "marquee");
    },
  },
  {
    name: "marquee-content",
    sourceFile: "families/media/marquee/marquee-content.vue",
    render: () => marquee("announcements-content"),
    assertServerMarkup(html) {
      assert.match(html, /data-vize-ui="marquee-track"/);
      assert.match(html, /data-copy="1" aria-hidden="true" inert/);
    },
    assertHydratedDom(host) {
      assert.equal(host.querySelectorAll('[data-vize-ui="marquee-content"]').length, 2);
    },
  },
  {
    name: "marquee-pause-button",
    sourceFile: "families/media/marquee/marquee-pause-button.vue",
    render: () => marquee("announcements-button"),
    assertServerMarkup(html) {
      assert.match(html, /<button type="button" aria-label="Pause scrolling content"/);
      assert.match(html, /aria-controls="announcements-button"/);
    },
    assertHydratedDom(host) {
      const button = host.querySelector('[data-vize-ui="marquee-pause-button"]');
      assert.ok(button instanceof HTMLButtonElement);
    },
  },
];
