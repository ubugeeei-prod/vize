import assert from "node:assert/strict";

import { h } from "vue";

import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";
import SkipLink from "./skip-link.vue";
import type { SkipLinkSlotState } from "./skip-link-types.ts";

export const skipLinkRuntimeFixture: RuntimeFixture = {
  name: "skip-link",
  sourceFile: "families/navigation/skip-link/skip-link.vue",
  render: () =>
    h(
      SkipLink,
      { href: "#main", id: "skip-main" },
      {
        default: ({ targetId }: SkipLinkSlotState) => `Skip to ${targetId}`,
      },
    ),
  assertServerMarkup(html) {
    assert.match(html, /^<a/);
    assert.match(html, /id="skip-main"/);
    assert.match(html, /href="#main"/);
    assert.match(html, /data-vize-ui="skip-link"/);
    assert.match(html, /part="root"/);
    assert.match(html, /data-state="idle"/);
    assert.match(html, /data-target-id="main"/);
    assert.match(html, /Skip to main/);
    assert.doesNotMatch(html, /class=/);
    assert.doesNotMatch(html, /style=/);
    assert.doesNotMatch(html, /tabindex=/);
  },
  assertHydratedDom(host) {
    const link = host.querySelector('[data-vize-ui="skip-link"]');
    assert.ok(link instanceof HTMLAnchorElement);
    assert.equal(link.id, "skip-main");
    assert.equal(link.getAttribute("href"), "#main");
    assert.equal(link.getAttribute("data-state"), "idle");
    assert.equal(link.textContent, "Skip to main");
  },
};
