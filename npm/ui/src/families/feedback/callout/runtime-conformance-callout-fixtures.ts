import assert from "node:assert/strict";

import { h } from "vue";

import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";
import Callout from "./callout.vue";

export const calloutRuntimeFixture: RuntimeFixture = {
  name: "callout",
  sourceFile: "families/feedback/callout/callout.vue",
  render: () =>
    h(
      Callout,
      {
        density: "compact",
        tone: "info",
      },
      {
        actions: () => h("a", { href: "/uploads" }, "View uploads"),
        default: () => "Uploads continue in the background.",
        description: () => "Large files may take a few minutes.",
        icon: () => "i",
        title: () => "Upload queued",
      },
    ),
  assertServerMarkup(html) {
    assert.match(html, /^<section/);
    assert.match(html, /role="note"/);
    assert.match(html, /data-vize-ui="callout"/);
    assert.match(html, /part="root"/);
    assert.match(html, /data-state="open"/);
    assert.match(html, /data-tone="info"/);
    assert.match(html, /data-density="compact"/);
    assert.match(html, /data-aria-state="note"/);
    assert.match(html, /data-live="off"/);
    assert.match(html, /aria-labelledby="vize-v-[^"]+-callout-title"/);
    assert.match(html, /aria-describedby="vize-v-[^"]+-callout-description"/);
    assert.match(html, /data-vize-ui="callout-icon" aria-hidden="true"/);
    assert.match(html, /data-vize-ui="callout-title"/);
    assert.match(html, /data-vize-ui="callout-description"/);
    assert.match(html, /data-vize-ui="callout-actions"/);
    assert.match(html, /href="\/uploads"/);
    assert.match(html, /Upload queued/);
    assert.match(html, /Large files may take a few minutes/);
    assert.doesNotMatch(html, /class=|style=|tabindex=/);
  },
  assertHydratedDom(host) {
    const callout = host.querySelector('[data-vize-ui="callout"]');
    const title = host.querySelector('[data-vize-ui="callout-title"]');
    const description = host.querySelector('[data-vize-ui="callout-description"]');
    const action = host.querySelector('[data-vize-ui="callout-actions"] a');

    assert.ok(callout instanceof HTMLElement);
    assert.ok(title instanceof HTMLElement);
    assert.ok(description instanceof HTMLElement);
    assert.ok(action instanceof HTMLAnchorElement);
    assert.equal(callout.tagName, "SECTION");
    assert.equal(callout.getAttribute("role"), "note");
    assert.equal(callout.getAttribute("aria-labelledby"), title.id);
    assert.equal(callout.getAttribute("aria-describedby"), description.id);
    assert.equal(callout.getAttribute("data-tone"), "info");
    assert.equal(callout.getAttribute("data-density"), "compact");
    assert.equal(callout.getAttribute("data-live"), "off");
    assert.equal(action.getAttribute("href"), "/uploads");
    assert.match(callout.textContent ?? "", /Upload queued/);
  },
};
