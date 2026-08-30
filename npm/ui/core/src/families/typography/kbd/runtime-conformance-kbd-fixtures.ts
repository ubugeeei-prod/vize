import assert from "node:assert/strict";

import { h } from "vue";

import Kbd from "./kbd.vue";
import type { RuntimeFixture } from "../../../runtime-conformance-fixtures.ts";

export const kbdRuntimeFixture: RuntimeFixture = {
  name: "kbd",
  sourceFile: "families/typography/kbd/kbd.vue",
  render: () =>
    h(
      Kbd,
      {
        size: "lg",
        tone: "muted",
        variant: "shortcut",
      },
      {
        default: () => "Ctrl K",
      },
    ),
  assertServerMarkup(html) {
    assert.match(html, /^<kbd/);
    assert.match(html, /part="root"/);
    assert.match(html, /data-vize-ui="kbd"/);
    assert.match(html, /data-size="lg"/);
    assert.match(html, /data-variant="shortcut"/);
    assert.match(html, /data-tone="muted"/);
    assert.match(html, /Ctrl K/);
    assert.doesNotMatch(html, /role=/);
    assert.doesNotMatch(html, /tabindex=/);
    assert.doesNotMatch(html, /aria-hidden=/);
    assert.doesNotMatch(html, /style=/);
  },
  assertHydratedDom(host) {
    const kbd = host.querySelector('[data-vize-ui="kbd"]');
    assert.ok(kbd instanceof HTMLElement);
    assert.equal(kbd.tagName, "KBD");
    assert.equal(kbd.getAttribute("part"), "root");
    assert.equal(kbd.getAttribute("data-size"), "lg");
    assert.equal(kbd.getAttribute("data-variant"), "shortcut");
    assert.equal(kbd.getAttribute("data-tone"), "muted");
    assert.equal(kbd.getAttribute("role"), null);
    assert.equal(kbd.getAttribute("tabindex"), null);
    assert.equal(kbd.getAttribute("aria-hidden"), null);
    assert.equal(kbd.getAttribute("style"), null);
    assert.equal(kbd.textContent, "Ctrl K");
  },
};
