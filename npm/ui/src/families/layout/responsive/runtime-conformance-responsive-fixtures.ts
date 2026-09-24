import assert from "node:assert/strict";

import { h } from "vue";

import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";
import ResponsiveShow from "./responsive-show.vue";
import ResponsiveSwitch from "./responsive-switch.vue";

export const responsiveRuntimeFixtures: readonly RuntimeFixture[] = [
  {
    name: "responsive-switch",
    sourceFile: "families/layout/responsive/responsive-switch.vue",
    render: () =>
      h(
        ResponsiveSwitch,
        { as: "section", ssrWidth: 1024 },
        {
          base: () => "Phone navigation",
          lg: () => h("nav", { "aria-label": "Primary" }, "Desktop navigation"),
        },
      ),
    assertServerMarkup(html) {
      assert.match(html, /^<section/);
      assert.match(html, /data-vize-ui="responsive-switch"/);
      assert.match(html, /data-breakpoint="lg"/);
      assert.match(html, /<nav aria-label="Primary">Desktop navigation<\/nav>/);
    },
    assertHydratedDom(host) {
      const root = host.querySelector('[data-vize-ui="responsive-switch"]');
      assert.ok(root instanceof HTMLElement);
      assert.equal(root.getAttribute("role"), null);
      assert.ok(host.querySelector('nav[aria-label="Primary"]'));
    },
  },
  {
    name: "responsive-show",
    sourceFile: "families/layout/responsive/responsive-show.vue",
    render: () => h(ResponsiveShow, { above: "md", ssrWidth: 1024 }, () => h("aside", "Sidebar")),
    assertServerMarkup(html) {
      assert.match(html, /data-vize-ui="responsive-show"/);
      assert.match(html, /data-state="visible"/);
      assert.doesNotMatch(html, /hidden/);
      assert.match(html, /display:contents/);
      assert.match(html, /<aside>Sidebar<\/aside>/);
    },
    assertHydratedDom(host) {
      const root = host.querySelector('[data-vize-ui="responsive-show"]');
      assert.ok(root instanceof HTMLElement);
      assert.equal(root.hidden, false);
      assert.ok(host.querySelector("aside"));
    },
  },
];
