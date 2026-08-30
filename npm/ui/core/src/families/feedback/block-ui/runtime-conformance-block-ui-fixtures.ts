import assert from "node:assert/strict";

import { h } from "vue";

import BlockUI from "./block-ui.vue";
import type { RuntimeFixture } from "../../../runtime-conformance-fixtures.ts";

export const blockUIRuntimeFixture: RuntimeFixture = {
  name: "block-ui",
  sourceFile: "families/feedback/block-ui/block-ui.vue",
  render: () =>
    h(
      BlockUI,
      {
        announce: "polite",
        as: "article",
        blocked: true,
        interaction: "inert",
        label: "Syncing account",
        reason: "syncing",
      },
      {
        default: () => "Account content",
      },
    ),
  assertServerMarkup(html) {
    assert.match(html, /^<article/);
    assert.match(html, /part="root"/);
    assert.match(html, /data-vize-ui="block-ui"/);
    assert.match(html, /data-state="blocked"/);
    assert.match(html, /data-reason="syncing"/);
    assert.match(html, /data-interaction="inert"/);
    assert.match(html, /data-announcement="polite"/);
    assert.match(html, /aria-busy="true"/);
    assert.match(html, /\sinert(?:=""|(?=[\s>]))/);
    assert.match(html, /role="status"/);
    assert.match(html, /aria-live="polite"/);
    assert.match(html, /aria-label="Syncing account"/);
    assert.match(html, /Account content/);
  },
  assertHydratedDom(host) {
    const blockUI = host.querySelector('[data-vize-ui="block-ui"]');
    assert.ok(blockUI instanceof HTMLElement);
    assert.equal(blockUI.tagName, "ARTICLE");
    assert.equal(blockUI.getAttribute("part"), "root");
    assert.equal(blockUI.getAttribute("data-state"), "blocked");
    assert.equal(blockUI.getAttribute("data-reason"), "syncing");
    assert.equal(blockUI.getAttribute("data-interaction"), "inert");
    assert.equal(blockUI.getAttribute("data-announcement"), "polite");
    assert.equal(blockUI.getAttribute("aria-busy"), "true");
    assert.equal(blockUI.hasAttribute("inert"), true);
    assert.equal(blockUI.getAttribute("role"), "status");
    assert.equal(blockUI.getAttribute("aria-live"), "polite");
    assert.equal(blockUI.getAttribute("aria-label"), "Syncing account");
    assert.equal(blockUI.textContent, "Account content");
  },
};
