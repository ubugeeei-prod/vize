import assert from "node:assert/strict";

import { h } from "vue";

import List from "./list.vue";
import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";

export const listRuntimeFixture: RuntimeFixture = {
  name: "list",
  sourceFile: "families/layout/list/list.vue",
  render: () =>
    h(
      List,
      {
        marker: "disc",
        spacing: "loose",
        tone: "muted",
      },
      {
        default: () => h("li", "Structured item"),
      },
    ),
  assertServerMarkup(html) {
    assert.match(html, /^<ul/);
    assert.match(html, /part="root"/);
    assert.match(html, /data-vize-ui="list"/);
    assert.match(html, /data-marker="disc"/);
    assert.match(html, /data-spacing="loose"/);
    assert.match(html, /data-tone="muted"/);
    assert.match(html, /<li>Structured item<\/li>/);
    assert.doesNotMatch(html, /role=/);
    assert.doesNotMatch(html, /tabindex=/);
    assert.doesNotMatch(html, /aria-hidden=/);
    assert.doesNotMatch(html, /aria-live=/);
    assert.doesNotMatch(html, /style=/);
  },
  assertHydratedDom(host) {
    const list = host.querySelector('[data-vize-ui="list"]');
    assert.ok(list instanceof HTMLUListElement);
    assert.equal(list.tagName, "UL");
    assert.equal(list.getAttribute("part"), "root");
    assert.equal(list.getAttribute("data-marker"), "disc");
    assert.equal(list.getAttribute("data-spacing"), "loose");
    assert.equal(list.getAttribute("data-tone"), "muted");
    assert.equal(list.getAttribute("role"), null);
    assert.equal(list.getAttribute("tabindex"), null);
    assert.equal(list.getAttribute("aria-hidden"), null);
    assert.equal(list.getAttribute("aria-live"), null);
    assert.equal(list.getAttribute("style"), null);
    assert.equal(list.textContent, "Structured item");
  },
};
