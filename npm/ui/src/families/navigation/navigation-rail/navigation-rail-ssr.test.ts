import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h } from "vue";

import NavigationRail from "./navigation-rail.vue";
import NavigationRailItem from "./navigation-rail-item.vue";
import NavigationRailToggle from "./navigation-rail-toggle.vue";
import { renderAndHydrate } from "../../../testing/ssr-hydration.ts";

test("renders byte-identical rail markup and hydrates without mismatches", async () => {
  const { html, dispose } = await renderAndHydrate(
    defineComponent({
      name: "NavigationRailSsrProbe",
      setup: () => () =>
        h(
          NavigationRail,
          { destinations: ["feed", "profile"], defaultValue: "profile", defaultExpanded: true },
          {
            default: () => [
              h(NavigationRailToggle, { ariaLabel: "Toggle navigation" }),
              h(NavigationRailItem, { value: "feed", href: "/feed" }, { default: () => "Feed" }),
              h(
                NavigationRailItem,
                { value: "profile", href: "/me", badge: "New" },
                { default: () => "Me" },
              ),
            ],
          },
        ),
    }),
  );
  try {
    assert.match(html, /^<nav id="[^"]+" aria-label="Primary" part="root"/);
    assert.match(html, /data-vize-ui="navigation-rail"[^>]* data-expanded="true"/);
    assert.match(html, /href="\/me" aria-current="page"/);
    assert.doesNotMatch(html, /href="\/feed" aria-current/);
    const railId = /^<nav id="([^"]+)"/.exec(html)?.[1];
    assert.ok(railId);
    assert.match(html, new RegExp(`aria-expanded="true" aria-controls="${railId}"`));
  } finally {
    dispose();
  }
});
