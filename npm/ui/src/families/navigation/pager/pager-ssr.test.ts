import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h } from "vue";

import Pager from "./pager.vue";
import PagerPage from "./pager-page.vue";
import PagerTab from "./pager-tab.vue";
import PagerTabList from "./pager-tab-list.vue";
import PagerViewport from "./pager-viewport.vue";
import { renderAndHydrate } from "../../../testing/ssr-hydration.ts";

const pages = ["photos", "albums"] as const;

test("renders byte-identical tabs and pages and hydrates without mismatches", async () => {
  const { html, dispose } = await renderAndHydrate(
    defineComponent({
      name: "PagerSsrProbe",
      setup: () => () =>
        h(Pager, { pages, defaultValue: "albums" }, () => [
          h(PagerTabList, null, () =>
            pages.map((page) => h(PagerTab, { key: page, page }, () => page)),
          ),
          h(PagerViewport, null, () =>
            pages.map((page) => h(PagerPage, { key: page, page }, () => page)),
          ),
        ]),
    }),
  );
  try {
    assert.match(html, /data-vize-ui="pager" data-page="albums"/);
    assert.match(
      html,
      /id="vize-v-\d+-pager-tab-albums"[^>]*role="tab" tabindex="0" aria-selected="true"/,
    );
    assert.match(html, /id="vize-v-\d+-pager-panel-photos" role="tabpanel"[^>]*inert/);
  } finally {
    dispose();
  }
});
