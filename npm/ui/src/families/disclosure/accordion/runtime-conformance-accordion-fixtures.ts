import assert from "node:assert/strict";

import { h } from "vue";

import {
  AccordionContent,
  AccordionHeader,
  AccordionItem,
  AccordionRoot,
  AccordionTrigger,
} from "./accordion.ts";
import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";

const accordionFamilyRoot = "families/disclosure/accordion/";

function accordion(children: () => unknown): ReturnType<typeof h> {
  return h(
    AccordionRoot,
    { type: "single", defaultValue: "shipping", id: "runtime-accordion" },
    () => h(AccordionItem, { value: "shipping" }, children),
  );
}

function fullItem(): unknown {
  return [
    h(AccordionHeader, null, () => h(AccordionTrigger, null, () => "Shipping")),
    h(AccordionContent, null, () => "Shipping details"),
  ];
}

export const accordionRuntimeFixtures: readonly RuntimeFixture[] = [
  {
    name: "accordion-root",
    sourceFile: `${accordionFamilyRoot}accordion-root.vue`,
    render: () => accordion(fullItem),
    assertServerMarkup(html) {
      assert.match(html, /id="runtime-accordion"/);
      assert.match(html, /data-vize-ui="accordion-root"/);
      assert.match(html, /data-type="single"/);
    },
    assertHydratedDom(host) {
      const root = host.querySelector('[data-vize-ui="accordion-root"]');
      assert.ok(root instanceof HTMLElement);
      assert.equal(root.getAttribute("data-orientation"), "vertical");
    },
  },
  {
    name: "accordion-item",
    sourceFile: `${accordionFamilyRoot}accordion-item.vue`,
    render: () => accordion(fullItem),
    assertServerMarkup(html) {
      assert.match(html, /id="runtime-accordion-item-shipping"/);
      assert.match(html, /data-vize-ui="accordion-item"/);
    },
    assertHydratedDom(host) {
      const item = host.querySelector('[data-vize-ui="accordion-item"]');
      assert.ok(item instanceof HTMLElement);
      assert.equal(item.getAttribute("data-state"), "open");
    },
  },
  {
    name: "accordion-header",
    sourceFile: `${accordionFamilyRoot}accordion-header.vue`,
    render: () => accordion(fullItem),
    assertServerMarkup(html) {
      assert.match(html, /<h3[^>]*data-vize-ui="accordion-header"/);
    },
    assertHydratedDom(host) {
      assert.ok(
        host.querySelector('[data-vize-ui="accordion-header"]') instanceof HTMLHeadingElement,
      );
    },
  },
  {
    name: "accordion-trigger",
    sourceFile: `${accordionFamilyRoot}accordion-trigger.vue`,
    render: () => accordion(fullItem),
    assertServerMarkup(html) {
      assert.match(html, /data-vize-ui="accordion-trigger"/);
      assert.match(html, /aria-controls="runtime-accordion-item-shipping-content"/);
      assert.match(html, /aria-expanded="true"/);
    },
    assertHydratedDom(host) {
      const trigger = host.querySelector('[data-vize-ui="accordion-trigger"]');
      assert.ok(trigger instanceof HTMLButtonElement);
      assert.equal(trigger.getAttribute("aria-disabled"), "true");
    },
  },
  {
    name: "accordion-content",
    sourceFile: `${accordionFamilyRoot}accordion-content.vue`,
    render: () => accordion(fullItem),
    assertServerMarkup(html) {
      assert.match(html, /data-vize-ui="accordion-content"/);
      assert.match(html, /role="region"/);
      assert.match(html, /aria-labelledby="runtime-accordion-item-shipping-trigger"/);
    },
    assertHydratedDom(host) {
      const content = host.querySelector('[data-vize-ui="accordion-content"]');
      assert.ok(content instanceof HTMLElement);
      assert.equal(content.hidden, false);
    },
  },
];
