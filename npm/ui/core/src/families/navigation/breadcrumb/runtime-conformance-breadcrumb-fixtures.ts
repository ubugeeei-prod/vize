import assert from "node:assert/strict";

import { h } from "vue";

import BreadcrumbRoot from "./breadcrumb.vue";
import BreadcrumbItem from "./breadcrumb-item.vue";
import BreadcrumbLink from "./breadcrumb-link.vue";
import BreadcrumbList from "./breadcrumb-list.vue";
import BreadcrumbSeparator from "./breadcrumb-separator.vue";
import type {
  BreadcrumbItemSlotState,
  BreadcrumbLinkSlotState,
  BreadcrumbSeparatorSlotState,
} from "./breadcrumb-types.ts";
import type { RuntimeFixture } from "../../../runtime-conformance-fixtures.ts";

export const breadcrumbRuntimeFixtures: readonly RuntimeFixture[] = [
  {
    name: "breadcrumb",
    sourceFile: "families/navigation/breadcrumb/breadcrumb.vue",
    render: () =>
      h(
        BreadcrumbRoot,
        { label: "Docs path" },
        {
          default: () =>
            h(BreadcrumbList, null, {
              default: () => [
                h(BreadcrumbItem, null, {
                  default: () => [
                    h(BreadcrumbLink, { href: "/" }, { default: () => "Home" }),
                    h(BreadcrumbSeparator, null, { default: () => "/" }),
                  ],
                }),
                h(
                  BreadcrumbItem,
                  { current: true },
                  {
                    default: () => h(BreadcrumbLink, { current: true }, { default: () => "Docs" }),
                  },
                ),
              ],
            }),
        },
      ),
    assertServerMarkup(html) {
      assert.match(html, /^<nav/);
      assert.match(html, /aria-label="Docs path"/);
      assert.match(html, /data-vize-ui="breadcrumb"/);
      assert.match(html, /data-vize-ui="breadcrumb-list"/);
      assert.match(html, /data-vize-ui="breadcrumb-item"/);
      assert.match(html, /data-vize-ui="breadcrumb-link"/);
      assert.match(html, /data-vize-ui="breadcrumb-separator"/);
      assert.match(html, /aria-hidden="true"/);
      assert.match(html, /role="presentation"/);
      assert.match(html, /aria-current="page"/);
      assert.doesNotMatch(html, /class=/);
      assert.doesNotMatch(html, /style=/);
      assert.doesNotMatch(html, /tabindex=/);
    },
    assertHydratedDom(host) {
      const breadcrumb = host.querySelector('[data-vize-ui="breadcrumb"]');
      assert.ok(breadcrumb instanceof HTMLElement);
      assert.equal(breadcrumb.tagName, "NAV");
      assert.equal(breadcrumb.getAttribute("aria-label"), "Docs path");
      assert.equal(breadcrumb.getAttribute("class"), null);
      assert.equal(breadcrumb.getAttribute("style"), null);
      assert.equal(breadcrumb.getAttribute("tabindex"), null);
    },
  },
  {
    name: "breadcrumb-item",
    sourceFile: "families/navigation/breadcrumb/breadcrumb-item.vue",
    render: () =>
      h(
        BreadcrumbItem,
        { current: true },
        {
          default: ({ current }: BreadcrumbItemSlotState) => String(current),
        },
      ),
    assertServerMarkup(html) {
      assert.match(html, /^<li/);
      assert.match(html, /data-vize-ui="breadcrumb-item"/);
      assert.match(html, /part="item"/);
      assert.match(html, /data-current="true"/);
      assert.match(html, /true/);
      assert.doesNotMatch(html, /class=/);
      assert.doesNotMatch(html, /style=/);
    },
    assertHydratedDom(host) {
      const item = host.querySelector('[data-vize-ui="breadcrumb-item"]');
      assert.ok(item instanceof HTMLLIElement);
      assert.equal(item.getAttribute("data-current"), "true");
      assert.equal(item.textContent, "true");
    },
  },
  {
    name: "breadcrumb-link",
    sourceFile: "families/navigation/breadcrumb/breadcrumb-link.vue",
    render: () =>
      h(
        BreadcrumbLink,
        { current: "location", href: "/settings" },
        {
          default: ({ ariaCurrent }: BreadcrumbLinkSlotState) => ariaCurrent,
        },
      ),
    assertServerMarkup(html) {
      assert.match(html, /^<a/);
      assert.match(html, /href="\/settings"/);
      assert.match(html, /aria-current="location"/);
      assert.match(html, /data-current="true"/);
      assert.match(html, /data-vize-ui="breadcrumb-link"/);
      assert.match(html, /location/);
      assert.doesNotMatch(html, /class=/);
      assert.doesNotMatch(html, /style=/);
    },
    assertHydratedDom(host) {
      const link = host.querySelector('[data-vize-ui="breadcrumb-link"]');
      assert.ok(link instanceof HTMLAnchorElement);
      assert.equal(link.getAttribute("href"), "/settings");
      assert.equal(link.getAttribute("aria-current"), "location");
      assert.equal(link.textContent, "location");
    },
  },
  {
    name: "breadcrumb-list",
    sourceFile: "families/navigation/breadcrumb/breadcrumb-list.vue",
    render: () => h(BreadcrumbList, null, { default: () => h("li", "Home") }),
    assertServerMarkup(html) {
      assert.match(html, /^<ol/);
      assert.match(html, /data-vize-ui="breadcrumb-list"/);
      assert.match(html, /part="list"/);
      assert.match(html, /<li>Home<\/li>/);
      assert.doesNotMatch(html, /role=/);
      assert.doesNotMatch(html, /class=/);
      assert.doesNotMatch(html, /style=/);
    },
    assertHydratedDom(host) {
      const list = host.querySelector('[data-vize-ui="breadcrumb-list"]');
      assert.ok(list instanceof HTMLOListElement);
      assert.equal(list.getAttribute("role"), null);
      assert.equal(list.textContent, "Home");
    },
  },
  {
    name: "breadcrumb-separator",
    sourceFile: "families/navigation/breadcrumb/breadcrumb-separator.vue",
    render: () =>
      h(BreadcrumbSeparator, null, {
        default: ({ decorative }: BreadcrumbSeparatorSlotState) => (decorative ? "/" : ""),
      }),
    assertServerMarkup(html) {
      assert.match(html, /^<span/);
      assert.match(html, /aria-hidden="true"/);
      assert.match(html, /role="presentation"/);
      assert.match(html, /data-vize-ui="breadcrumb-separator"/);
      assert.match(html, /\//);
      assert.doesNotMatch(html, /class=/);
      assert.doesNotMatch(html, /style=/);
    },
    assertHydratedDom(host) {
      const separator = host.querySelector('[data-vize-ui="breadcrumb-separator"]');
      assert.ok(separator instanceof HTMLSpanElement);
      assert.equal(separator.getAttribute("aria-hidden"), "true");
      assert.equal(separator.getAttribute("role"), "presentation");
      assert.equal(separator.textContent, "/");
    },
  },
];
