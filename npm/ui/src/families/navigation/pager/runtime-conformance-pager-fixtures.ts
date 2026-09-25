import assert from "node:assert/strict";

import { h } from "vue";

import Pager from "./pager.vue";
import PagerPage from "./pager-page.vue";
import PagerTab from "./pager-tab.vue";
import PagerTabList from "./pager-tab-list.vue";
import PagerViewport from "./pager-viewport.vue";
import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";

const pages = ["one", "two"] as const;
const render = () =>
  h(
    Pager,
    { pages, id: "pager" },
    {
      default: () => [
        h(PagerTabList, null, () =>
          pages.map((page) => h(PagerTab, { key: page, page }, () => page)),
        ),
        h(PagerViewport, null, () =>
          pages.map((page) => h(PagerPage, { key: page, page }, () => page)),
        ),
      ],
    },
  );

export const pagerRuntimeFixtures: readonly RuntimeFixture[] = [
  "pager.vue",
  "pager-tab-list.vue",
  "pager-tab.vue",
  "pager-viewport.vue",
  "pager-page.vue",
].map((file) => ({
  name: file.replace(/\.vue$/, ""),
  sourceFile: `families/navigation/pager/${file}`,
  render,
  assertServerMarkup(html: string) {
    assert.match(html, /role="tablist"/);
    assert.match(html, /id="pager-panel-one"/);
  },
  assertHydratedDom(host: HTMLElement) {
    assert.equal(host.querySelector("#pager-tab-one")?.getAttribute("aria-selected"), "true");
  },
}));
