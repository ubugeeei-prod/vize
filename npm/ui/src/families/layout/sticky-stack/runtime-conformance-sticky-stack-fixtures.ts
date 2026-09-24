import assert from "node:assert/strict";

import { h } from "vue";

import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";
import StickyStackItem from "./sticky-stack-item.vue";
import StickyStack from "./sticky-stack.vue";

const render = () =>
  h(StickyStack, { offset: 8 }, () => [
    h(StickyStackItem, { as: "header", estimatedHeight: 48 }, () => "App bar"),
    h(StickyStackItem, { as: "nav", estimatedHeight: 32 }, () => h("a", { href: "#top" }, "Top")),
  ]);

function assertServer(html: string): void {
  assert.match(html, /data-vize-ui="sticky-stack"/);
  assert.match(html, /<header[^>]*data-vize-ui="sticky-stack-item"[^>]*position:sticky;top:8px;/);
  assert.match(html, /<nav[^>]*position:sticky;top:56px;/);
}

function assertHydrated(host: HTMLElement): void {
  assert.ok(host.querySelector('[data-vize-ui="sticky-stack"]') instanceof HTMLElement);
  assert.equal(host.querySelectorAll('[data-vize-ui="sticky-stack-item"]').length, 2);
  assert.ok(host.querySelector('nav a[href="#top"]'));
}

export const stickyStackRuntimeFixtures: readonly RuntimeFixture[] = [
  {
    name: "sticky-stack",
    sourceFile: "families/layout/sticky-stack/sticky-stack.vue",
    render,
    assertServerMarkup: assertServer,
    assertHydratedDom: assertHydrated,
  },
  {
    name: "sticky-stack-item",
    sourceFile: "families/layout/sticky-stack/sticky-stack-item.vue",
    render,
    assertServerMarkup: assertServer,
    assertHydratedDom: assertHydrated,
  },
];
