import assert from "node:assert/strict";

import { h } from "vue";

import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";
import Heading from "./heading.vue";

export const headingRuntimeFixture: RuntimeFixture = {
  name: "heading",
  sourceFile: "families/typography/heading/heading.vue",
  render: () =>
    h(
      Heading,
      {
        level: 3,
        size: "lg",
        tone: "muted",
        truncate: true,
        weight: "bold",
      },
      {
        default: () => "Release notes",
      },
    ),
  assertServerMarkup(html) {
    assert.match(html, /^<h3/);
    assert.match(html, /part="root"/);
    assert.match(html, /data-vize-ui="heading"/);
    assert.match(html, /data-level="3"/);
    assert.match(html, /data-size="lg"/);
    assert.match(html, /data-weight="bold"/);
    assert.match(html, /data-tone="muted"/);
    assert.match(html, /data-truncate="true"/);
    assert.match(html, /Release notes/);
    assert.doesNotMatch(html, /role=/);
    assert.doesNotMatch(html, /tabindex=/);
    assert.doesNotMatch(html, /aria-level=/);
    assert.doesNotMatch(html, /aria-hidden=/);
    assert.doesNotMatch(html, /style=/);
  },
  assertHydratedDom(host) {
    const heading = host.querySelector('[data-vize-ui="heading"]');
    assert.ok(heading instanceof HTMLHeadingElement);
    assert.equal(heading.tagName, "H3");
    assert.equal(heading.getAttribute("part"), "root");
    assert.equal(heading.getAttribute("data-level"), "3");
    assert.equal(heading.getAttribute("data-size"), "lg");
    assert.equal(heading.getAttribute("data-weight"), "bold");
    assert.equal(heading.getAttribute("data-tone"), "muted");
    assert.equal(heading.getAttribute("data-truncate"), "true");
    assert.equal(heading.getAttribute("role"), null);
    assert.equal(heading.getAttribute("tabindex"), null);
    assert.equal(heading.getAttribute("aria-level"), null);
    assert.equal(heading.getAttribute("aria-hidden"), null);
    assert.equal(heading.getAttribute("style"), null);
    assert.equal(heading.textContent, "Release notes");
  },
};
