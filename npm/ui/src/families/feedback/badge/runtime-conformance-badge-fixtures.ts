import assert from "node:assert/strict";

import { h } from "vue";

import Badge from "./badge.vue";
import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";

export const badgeRuntimeFixture: RuntimeFixture = {
  name: "badge",
  sourceFile: "families/feedback/badge/badge.vue",
  render: () =>
    h(
      Badge,
      {
        as: "strong",
        tone: "success",
        variant: "status",
      },
      {
        default: () => "Online",
      },
    ),
  assertServerMarkup(html) {
    assert.match(html, /^<strong/);
    assert.match(html, /part="root"/);
    assert.match(html, /data-vize-ui="badge"/);
    assert.match(html, /data-variant="status"/);
    assert.match(html, /data-tone="success"/);
    assert.match(html, /Online/);
    assert.doesNotMatch(html, /role=/);
    assert.doesNotMatch(html, /tabindex=/);
    assert.doesNotMatch(html, /aria-hidden=/);
  },
  assertHydratedDom(host) {
    const badge = host.querySelector('[data-vize-ui="badge"]');
    assert.ok(badge instanceof HTMLElement);
    assert.equal(badge.tagName, "STRONG");
    assert.equal(badge.getAttribute("part"), "root");
    assert.equal(badge.getAttribute("data-variant"), "status");
    assert.equal(badge.getAttribute("data-tone"), "success");
    assert.equal(badge.getAttribute("role"), null);
    assert.equal(badge.getAttribute("tabindex"), null);
    assert.equal(badge.getAttribute("aria-hidden"), null);
    assert.equal(badge.textContent, "Online");
  },
};
