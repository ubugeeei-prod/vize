import assert from "node:assert/strict";

import { h } from "vue";

import Card from "./card.vue";
import type { RuntimeFixture } from "./runtime-conformance-fixtures.ts";

export const cardRuntimeFixture: RuntimeFixture = {
  name: "card",
  sourceFile: "card.vue",
  render: () =>
    h(
      Card,
      {
        "aria-label": "Release summary",
        as: "article",
        density: "compact",
        role: "region",
        tone: "info",
        variant: "panel",
      },
      {
        default: () => [h("h2", "Release"), h("p", "Ready")],
      },
    ),
  assertServerMarkup(html) {
    assert.match(html, /^<article/);
    assert.match(html, /part="root"/);
    assert.match(html, /data-vize-ui="card"/);
    assert.match(html, /data-variant="panel"/);
    assert.match(html, /data-density="compact"/);
    assert.match(html, /data-tone="info"/);
    assert.match(html, /role="region"/);
    assert.match(html, /aria-label="Release summary"/);
    assert.match(html, /<h2>Release<\/h2><p>Ready<\/p>/);
    assert.doesNotMatch(html, /class=/);
    assert.doesNotMatch(html, /style=/);
    assert.doesNotMatch(html, /tabindex=/);
    assert.doesNotMatch(html, /aria-hidden=/);
    assert.doesNotMatch(html, /aria-live=/);
  },
  assertHydratedDom(host) {
    const card = host.querySelector('[data-vize-ui="card"]');
    assert.ok(card instanceof HTMLElement);
    assert.equal(card.tagName, "ARTICLE");
    assert.equal(card.getAttribute("part"), "root");
    assert.equal(card.getAttribute("data-variant"), "panel");
    assert.equal(card.getAttribute("data-density"), "compact");
    assert.equal(card.getAttribute("data-tone"), "info");
    assert.equal(card.getAttribute("role"), "region");
    assert.equal(card.getAttribute("aria-label"), "Release summary");
    assert.equal(card.getAttribute("class"), null);
    assert.equal(card.getAttribute("style"), null);
    assert.equal(card.getAttribute("tabindex"), null);
    assert.equal(card.getAttribute("aria-hidden"), null);
    assert.equal(card.getAttribute("aria-live"), null);
    assert.equal(card.textContent, "ReleaseReady");
  },
};
