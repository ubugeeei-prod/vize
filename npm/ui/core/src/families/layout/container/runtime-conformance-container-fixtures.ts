import assert from "node:assert/strict";

import { h } from "vue";

import Container from "./container.vue";
import type { RuntimeFixture } from "../../../runtime-conformance-fixtures.ts";

export const containerRuntimeFixture: RuntimeFixture = {
  name: "container",
  sourceFile: "families/layout/container/container.vue",
  render: () =>
    h(
      Container,
      {
        as: "main",
        paddingInline: 16,
        size: "lg",
      },
      {
        default: () => [h("h1", "Reports"), h("p", "Container content")],
      },
    ),
  assertServerMarkup(html) {
    assert.match(html, /^<main/);
    assert.match(html, /part="root"/);
    assert.match(html, /data-vize-ui="container"/);
    assert.match(html, /data-size="lg"/);
    assert.match(html, /data-centered="true"/);
    assert.match(html, /--vize-ui-container-max-inline-size:80rem/);
    assert.match(html, /--vize-ui-container-padding-inline:16px/);
    assert.match(html, /max-inline-size:var\(--vize-ui-container-max-inline-size\)/);
    assert.match(html, /padding-inline:var\(--vize-ui-container-padding-inline\)/);
    assert.match(html, /margin-inline:auto/);
    assert.match(html, /<h1>Reports<\/h1><p>Container content<\/p>/);
  },
  assertHydratedDom(host) {
    const container = host.querySelector('[data-vize-ui="container"]');
    assert.ok(container instanceof HTMLElement);
    assert.equal(container.getAttribute("role"), null);
    assert.equal(container.getAttribute("aria-hidden"), null);
    assert.equal(container.getAttribute("tabindex"), null);
    assert.equal(container.getAttribute("part"), "root");
    assert.equal(container.getAttribute("data-size"), "lg");
    assert.equal(container.getAttribute("data-centered"), "true");
    assert.equal(container.style.getPropertyValue("--vize-ui-container-max-inline-size"), "80rem");
    assert.equal(container.style.getPropertyValue("--vize-ui-container-padding-inline"), "16px");
    assert.equal(container.style.getPropertyValue("margin-inline"), "auto");
    assert.equal(
      container.style.getPropertyValue("max-inline-size"),
      "var(--vize-ui-container-max-inline-size)",
    );
    assert.equal(
      container.style.getPropertyValue("padding-inline"),
      "var(--vize-ui-container-padding-inline)",
    );
    assert.equal(container.textContent, "ReportsContainer content");
  },
};
