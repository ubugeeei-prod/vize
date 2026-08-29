import assert from "node:assert/strict";

import { h } from "vue";

import { CollapsibleContent, CollapsibleRoot, CollapsibleTrigger } from "./collapsible.ts";
import type { RuntimeFixture } from "./runtime-conformance-fixtures.ts";

function collapsible(children: () => unknown): ReturnType<typeof h> {
  return h(CollapsibleRoot, { defaultOpen: true, id: "runtime-collapsible" }, children);
}

function assertRoot(host: HTMLElement): void {
  const root = host.querySelector('[data-vize-ui="collapsible-root"]');
  assert.ok(root instanceof HTMLElement);
  assert.equal(root.getAttribute("data-state"), "open");
}

export const collapsibleRuntimeFixtures: readonly RuntimeFixture[] = [
  {
    name: "collapsible-root",
    sourceFile: "collapsible-root.vue",
    render: () => collapsible(() => "Disclosure"),
    assertServerMarkup(html) {
      assert.match(html, /id="runtime-collapsible"/);
      assert.match(html, /data-vize-ui="collapsible-root"/);
      assert.match(html, /data-state="open"/);
    },
    assertHydratedDom: assertRoot,
  },
  {
    name: "collapsible-trigger",
    sourceFile: "collapsible-trigger.vue",
    render: () => collapsible(() => h(CollapsibleTrigger, null, () => "Filters")),
    assertServerMarkup(html) {
      assert.match(html, /data-vize-ui="collapsible-trigger"/);
      assert.match(html, /aria-controls="runtime-collapsible-content"/);
      assert.match(html, /aria-expanded="true"/);
    },
    assertHydratedDom(host) {
      const trigger = host.querySelector('[data-vize-ui="collapsible-trigger"]');
      assert.ok(trigger instanceof HTMLButtonElement);
      assert.equal(trigger.getAttribute("aria-expanded"), "true");
      assert.equal(trigger.getAttribute("aria-controls"), "runtime-collapsible-content");
    },
  },
  {
    name: "collapsible-content",
    sourceFile: "collapsible-content.vue",
    render: () => collapsible(() => h(CollapsibleContent, null, () => "Filter controls")),
    assertServerMarkup(html) {
      assert.match(html, /data-vize-ui="collapsible-content"/);
      assert.match(html, /role="region"/);
      assert.match(html, /aria-labelledby="runtime-collapsible-trigger"/);
      assert.doesNotMatch(html, /hidden/);
    },
    assertHydratedDom(host) {
      const content = host.querySelector('[data-vize-ui="collapsible-content"]');
      assert.ok(content instanceof HTMLDivElement);
      assert.equal(content.hidden, false);
      assert.equal(content.getAttribute("role"), "region");
      assert.equal(content.getAttribute("aria-labelledby"), "runtime-collapsible-trigger");
    },
  },
];
