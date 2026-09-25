import assert from "node:assert/strict";

import { h } from "vue";

import PullToRefresh from "./pull-to-refresh.vue";
import PullToRefreshTrigger from "./pull-to-refresh-trigger.vue";
import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";

const render = () =>
  h(
    PullToRefresh,
    { id: "feed" },
    { default: () => h(PullToRefreshTrigger, null, { default: () => "Refresh" }) },
  );

export const pullToRefreshRuntimeFixtures: readonly RuntimeFixture[] = [
  "pull-to-refresh.vue",
  "pull-to-refresh-trigger.vue",
].map((file) => ({
  name: file.replace(/\.vue$/, ""),
  sourceFile: `families/interaction/pull-to-refresh/${file}`,
  render,
  assertServerMarkup(html: string) {
    assert.match(html, /^<div id="feed"/);
    assert.match(html, /aria-controls="feed"/);
  },
  assertHydratedDom(host: HTMLElement) {
    assert.equal(host.querySelector("#feed")?.getAttribute("data-state"), "idle");
  },
}));
