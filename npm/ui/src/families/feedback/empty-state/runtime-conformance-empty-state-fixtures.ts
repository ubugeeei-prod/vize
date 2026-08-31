import assert from "node:assert/strict";

import { h } from "vue";

import EmptyState from "./empty-state.vue";
import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";

export const emptyStateRuntimeFixture: RuntimeFixture = {
  name: "empty-state",
  sourceFile: "families/feedback/empty-state/empty-state.vue",
  render: () =>
    h(
      EmptyState,
      {
        as: "article",
        density: "compact",
        orientation: "inline",
        tone: "info",
      },
      {
        default: () => "No matching projects",
      },
    ),
  assertServerMarkup(html) {
    assert.match(html, /^<article/);
    assert.match(html, /part="root"/);
    assert.match(html, /data-vize-ui="empty-state"/);
    assert.match(html, /data-state="empty"/);
    assert.match(html, /data-tone="info"/);
    assert.match(html, /data-density="compact"/);
    assert.match(html, /data-orientation="inline"/);
    assert.match(html, /No matching projects/);
    assert.doesNotMatch(html, /role=/);
    assert.doesNotMatch(html, /tabindex=/);
    assert.doesNotMatch(html, /aria-hidden=/);
    assert.doesNotMatch(html, /aria-live=/);
  },
  assertHydratedDom(host) {
    const emptyState = host.querySelector('[data-vize-ui="empty-state"]');
    assert.ok(emptyState instanceof HTMLElement);
    assert.equal(emptyState.tagName, "ARTICLE");
    assert.equal(emptyState.getAttribute("part"), "root");
    assert.equal(emptyState.getAttribute("data-state"), "empty");
    assert.equal(emptyState.getAttribute("data-tone"), "info");
    assert.equal(emptyState.getAttribute("data-density"), "compact");
    assert.equal(emptyState.getAttribute("data-orientation"), "inline");
    assert.equal(emptyState.getAttribute("role"), null);
    assert.equal(emptyState.getAttribute("tabindex"), null);
    assert.equal(emptyState.getAttribute("aria-hidden"), null);
    assert.equal(emptyState.getAttribute("aria-live"), null);
    assert.equal(emptyState.textContent, "No matching projects");
  },
};
