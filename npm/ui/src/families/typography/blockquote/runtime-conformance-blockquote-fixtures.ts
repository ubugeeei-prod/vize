import assert from "node:assert/strict";

import { h } from "vue";

import Blockquote from "./blockquote.vue";
import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";

export const blockquoteRuntimeFixture: RuntimeFixture = {
  name: "blockquote",
  sourceFile: "families/typography/blockquote/blockquote.vue",
  render: () =>
    h(
      Blockquote,
      {
        cite: "https://example.com/source",
        size: "lg",
        tone: "muted",
      },
      {
        default: () => "Readable pull quote",
      },
    ),
  assertServerMarkup(html) {
    assert.match(html, /^<blockquote/);
    assert.match(html, /part="root"/);
    assert.match(html, /data-vize-ui="blockquote"/);
    assert.match(html, /cite="https:\/\/example\.com\/source"/);
    assert.match(html, /data-size="lg"/);
    assert.match(html, /data-tone="muted"/);
    assert.match(html, /Readable pull quote/);
    assert.doesNotMatch(html, /role=/);
    assert.doesNotMatch(html, /tabindex=/);
    assert.doesNotMatch(html, /aria-hidden=/);
    assert.doesNotMatch(html, /aria-live=/);
    assert.doesNotMatch(html, /style=/);
  },
  assertHydratedDom(host) {
    const blockquote = host.querySelector('[data-vize-ui="blockquote"]');
    assert.ok(blockquote instanceof HTMLQuoteElement);
    assert.equal(blockquote.tagName, "BLOCKQUOTE");
    assert.equal(blockquote.getAttribute("part"), "root");
    assert.equal(blockquote.getAttribute("cite"), "https://example.com/source");
    assert.equal(blockquote.getAttribute("data-size"), "lg");
    assert.equal(blockquote.getAttribute("data-tone"), "muted");
    assert.equal(blockquote.getAttribute("role"), null);
    assert.equal(blockquote.getAttribute("tabindex"), null);
    assert.equal(blockquote.getAttribute("aria-hidden"), null);
    assert.equal(blockquote.getAttribute("aria-live"), null);
    assert.equal(blockquote.getAttribute("style"), null);
    assert.equal(blockquote.textContent, "Readable pull quote");
  },
};
