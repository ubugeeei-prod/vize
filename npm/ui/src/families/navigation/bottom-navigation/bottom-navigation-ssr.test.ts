import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h } from "vue";

import BottomNavigation from "./bottom-navigation.vue";
import BottomNavigationItem from "./bottom-navigation-item.vue";
import { renderAndHydrate } from "../../../testing/ssr-hydration.ts";

test("renders byte-identical tab bar markup and hydrates without mismatches", async () => {
  const { html, dispose } = await renderAndHydrate(
    defineComponent({
      name: "BottomNavigationSsrProbe",
      setup: () => () =>
        h(
          BottomNavigation,
          { destinations: ["feed", "profile"], defaultValue: "profile" },
          {
            default: () => [
              h(BottomNavigationItem, { value: "feed", href: "/feed" }, { default: () => "Feed" }),
              h(
                BottomNavigationItem,
                { value: "profile", href: "/me", badge: "New" },
                { default: () => "Me" },
              ),
            ],
          },
        ),
    }),
  );
  try {
    assert.match(html, /^<nav aria-label="Primary" part="root" data-vize-ui="bottom-navigation"/);
    assert.match(html, /href="\/me" aria-current="page"/);
    assert.doesNotMatch(html, /href="\/feed" aria-current/);
  } finally {
    dispose();
  }
});
