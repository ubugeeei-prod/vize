import assert from "node:assert/strict";

import { h } from "vue";

import { HoverCardContent, HoverCardRoot, HoverCardTrigger } from "./hover-card.ts";
import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";

const hoverCardFamilyRoot = "families/overlays/hover-card/";

function hoverCard(children: () => unknown): ReturnType<typeof h> {
  return h(HoverCardRoot, { defaultOpen: true, id: "runtime-hover-card" }, children);
}

export const hoverCardRuntimeFixtures: readonly RuntimeFixture[] = [
  {
    name: "hover-card-root",
    sourceFile: `${hoverCardFamilyRoot}hover-card-root.vue`,
    render: () => hoverCard(() => "Profile"),
    assertServerMarkup(html) {
      assert.match(html, /<span id="runtime-hover-card"/);
      assert.match(html, /data-vize-ui="hover-card-root"/);
      assert.match(html, /data-state="open"/);
    },
    assertHydratedDom(host) {
      const root = host.querySelector('[data-vize-ui="hover-card-root"]');
      assert.ok(root instanceof HTMLSpanElement);
    },
  },
  {
    name: "hover-card-trigger",
    sourceFile: `${hoverCardFamilyRoot}hover-card-trigger.vue`,
    render: () => hoverCard(() => h(HoverCardTrigger, { href: "/users/ada" }, () => "@ada")),
    assertServerMarkup(html) {
      assert.match(html, /<a id="runtime-hover-card-trigger"/);
      assert.match(html, /aria-describedby="runtime-hover-card-content"/);
    },
    assertHydratedDom(host) {
      const trigger = host.querySelector('[data-vize-ui="hover-card-trigger"]');
      assert.ok(trigger instanceof HTMLAnchorElement);
      assert.equal(trigger.getAttribute("href"), "/users/ada");
    },
  },
  {
    name: "hover-card-content",
    sourceFile: `${hoverCardFamilyRoot}hover-card-content.vue`,
    render: () =>
      hoverCard(() => h(HoverCardContent, { portalDisabled: true }, () => "Ada Lovelace")),
    assertServerMarkup(html) {
      assert.match(html, /data-vize-ui="hover-card-content"/);
      assert.match(html, /id="runtime-hover-card-content"/);
    },
    assertHydratedDom(host) {
      const content = host.querySelector('[data-vize-ui="hover-card-content"]');
      assert.ok(content instanceof HTMLElement);
      assert.equal(content.getAttribute("data-state"), "open");
    },
  },
];
