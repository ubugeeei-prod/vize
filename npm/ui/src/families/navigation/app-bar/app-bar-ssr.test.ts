import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h } from "vue";

import AppBar from "./app-bar.vue";
import { renderAndHydrate } from "../../../testing/ssr-hydration.ts";

test("renders byte-identical app bar markup and hydrates without mismatches", async () => {
  const { html, host, dispose } = await renderAndHydrate(
    defineComponent({
      name: "AppBarSsrProbe",
      setup: () => () =>
        h(
          AppBar,
          { hideOnScroll: true, ariaLabel: "Application" },
          {
            start: () => h("button", { type: "button" }, "Menu"),
            default: () => h("span", null, "Inbox"),
            end: () => h("button", { type: "button" }, "Search"),
          },
        ),
    }),
  );
  try {
    assert.match(html, /^<header aria-label="Application" part="root" data-vize-ui="app-bar"/);
    assert.match(html, /data-state="visible"/);
    assert.doesNotMatch(html, /data-hidden|data-elevated/, "the server never reports scroll state");
    assert.equal(host.querySelectorAll('[part="start"], [part="content"], [part="end"]').length, 3);
  } finally {
    dispose();
  }
});
