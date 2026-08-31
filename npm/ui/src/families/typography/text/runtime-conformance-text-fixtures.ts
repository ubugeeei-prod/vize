import assert from "node:assert/strict";

import { h } from "vue";

import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";
import Text from "./text.vue";

export const textRuntimeFixture: RuntimeFixture = {
  name: "text",
  sourceFile: "families/typography/text/text.vue",
  render: () =>
    h(
      Text,
      {
        as: "p",
        size: "lg",
        tone: "muted",
        truncate: true,
        weight: "medium",
      },
      {
        default: () => "Readable copy",
      },
    ),
  assertServerMarkup(html) {
    assert.match(html, /^<p/);
    assert.match(html, /part="root"/);
    assert.match(html, /data-vize-ui="text"/);
    assert.match(html, /data-size="lg"/);
    assert.match(html, /data-weight="medium"/);
    assert.match(html, /data-tone="muted"/);
    assert.match(html, /data-truncate="true"/);
    assert.match(html, /Readable copy/);
    assert.doesNotMatch(html, /role=/);
    assert.doesNotMatch(html, /tabindex=/);
    assert.doesNotMatch(html, /aria-hidden=/);
    assert.doesNotMatch(html, /aria-live=/);
    assert.doesNotMatch(html, /style=/);
  },
  assertHydratedDom(host) {
    const text = host.querySelector('[data-vize-ui="text"]');
    assert.ok(text instanceof HTMLParagraphElement);
    assert.equal(text.tagName, "P");
    assert.equal(text.getAttribute("part"), "root");
    assert.equal(text.getAttribute("data-size"), "lg");
    assert.equal(text.getAttribute("data-weight"), "medium");
    assert.equal(text.getAttribute("data-tone"), "muted");
    assert.equal(text.getAttribute("data-truncate"), "true");
    assert.equal(text.getAttribute("role"), null);
    assert.equal(text.getAttribute("tabindex"), null);
    assert.equal(text.getAttribute("aria-hidden"), null);
    assert.equal(text.getAttribute("aria-live"), null);
    assert.equal(text.getAttribute("style"), null);
    assert.equal(text.textContent, "Readable copy");
  },
};
