import assert from "node:assert/strict";

import { h } from "vue";

import type { RuntimeFixture } from "../../../runtime-conformance-fixtures.ts";
import Skeleton from "./skeleton.vue";

export const skeletonRuntimeFixture: RuntimeFixture = {
  name: "skeleton",
  sourceFile: "families/feedback/skeleton/skeleton.vue",
  render: () =>
    h(
      Skeleton,
      {
        ariaLabel: "Loading profile",
        as: "section",
        blockSize: "2rem",
      },
      {
        default: () => "Loading",
      },
    ),
  assertServerMarkup(html) {
    assert.match(html, /^<section/);
    assert.match(html, /role="status"/);
    assert.match(html, /aria-label="Loading profile"/);
    assert.match(html, /data-vize-ui="skeleton"/);
    assert.match(html, /data-state="loading"/);
    assert.match(html, /data-aria-state="status"/);
    assert.match(html, /Loading/);
  },
  assertHydratedDom(host) {
    const skeleton = host.querySelector('[data-vize-ui="skeleton"]');
    assert.ok(skeleton instanceof HTMLElement);
    assert.equal(skeleton.tagName, "SECTION");
    assert.equal(skeleton.getAttribute("role"), "status");
    assert.equal(skeleton.getAttribute("data-state"), "loading");
    assert.equal(skeleton.style.getPropertyValue("--vize-ui-skeleton-block-size"), "2rem");
    assert.equal(skeleton.textContent, "Loading");
  },
};
