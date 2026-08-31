import assert from "node:assert/strict";

import { h } from "vue";

import Code from "./code.vue";
import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";

export const codeRuntimeFixture: RuntimeFixture = {
  name: "code",
  sourceFile: "families/typography/code/code.vue",
  render: () =>
    h(
      Code,
      {
        size: "lg",
        tone: "muted",
        variant: "snippet",
      },
      {
        default: () => "const value = 1;",
      },
    ),
  assertServerMarkup(html) {
    assert.match(html, /^<code/);
    assert.match(html, /part="root"/);
    assert.match(html, /data-vize-ui="code"/);
    assert.match(html, /data-size="lg"/);
    assert.match(html, /data-variant="snippet"/);
    assert.match(html, /data-tone="muted"/);
    assert.match(html, /const value = 1;/);
    assert.doesNotMatch(html, /role=/);
    assert.doesNotMatch(html, /tabindex=/);
    assert.doesNotMatch(html, /aria-hidden=/);
    assert.doesNotMatch(html, /aria-live=/);
    assert.doesNotMatch(html, /style=/);
  },
  assertHydratedDom(host) {
    const code = host.querySelector('[data-vize-ui="code"]');
    assert.ok(code instanceof HTMLElement);
    assert.equal(code.tagName, "CODE");
    assert.equal(code.getAttribute("part"), "root");
    assert.equal(code.getAttribute("data-size"), "lg");
    assert.equal(code.getAttribute("data-variant"), "snippet");
    assert.equal(code.getAttribute("data-tone"), "muted");
    assert.equal(code.getAttribute("role"), null);
    assert.equal(code.getAttribute("tabindex"), null);
    assert.equal(code.getAttribute("aria-hidden"), null);
    assert.equal(code.getAttribute("aria-live"), null);
    assert.equal(code.getAttribute("style"), null);
    assert.equal(code.textContent, "const value = 1;");
  },
};
