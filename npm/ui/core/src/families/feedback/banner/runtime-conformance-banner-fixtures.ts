import assert from "node:assert/strict";

import { h } from "vue";

import type { RuntimeFixture } from "../../../runtime-conformance-fixtures.ts";
import Banner from "./banner.vue";

export const bannerRuntimeFixture: RuntimeFixture = {
  name: "banner",
  sourceFile: "families/feedback/banner/banner.vue",
  render: () =>
    h(
      Banner,
      {
        description: "Scheduled from 02:00 to 02:15 UTC.",
        id: "maintenance-banner",
        title: "System maintenance",
        tone: "warning",
      },
      {
        default: () => h("p", "Some features will pause briefly."),
      },
    ),
  assertServerMarkup(html) {
    assert.match(html, /^<section/);
    assert.match(html, /id="maintenance-banner"/);
    assert.match(html, /role="region"/);
    assert.match(html, /aria-labelledby="maintenance-banner-title"/);
    assert.match(html, /aria-describedby="maintenance-banner-description"/);
    assert.match(html, /data-vize-ui="banner"/);
    assert.match(html, /part="root"/);
    assert.match(html, /data-state="open"/);
    assert.match(html, /data-tone="warning"/);
    assert.match(html, /data-aria-state="named"/);
    assert.match(html, /System maintenance/);
    assert.doesNotMatch(html, /class=|style=|tabindex=/);
  },
  assertHydratedDom(host) {
    const banner = host.querySelector('[data-vize-ui="banner"]');
    const title = host.querySelector('[data-vize-ui="banner-title"]');
    const description = host.querySelector('[data-vize-ui="banner-description"]');

    assert.ok(banner instanceof HTMLElement);
    assert.ok(title instanceof HTMLElement);
    assert.ok(description instanceof HTMLElement);
    assert.equal(banner.tagName, "SECTION");
    assert.equal(banner.getAttribute("role"), "region");
    assert.equal(banner.getAttribute("aria-labelledby"), title.id);
    assert.equal(banner.getAttribute("aria-describedby"), description.id);
    assert.equal(banner.getAttribute("data-state"), "open");
    assert.equal(banner.getAttribute("data-tone"), "warning");
    assert.equal(banner.textContent?.includes("System maintenance"), true);
  },
};
