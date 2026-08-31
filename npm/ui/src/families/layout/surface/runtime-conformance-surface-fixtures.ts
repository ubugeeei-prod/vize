import assert from "node:assert/strict";

import { h } from "vue";

import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";
import Surface from "./surface.vue";

export const surfaceRuntimeFixture: RuntimeFixture = {
  name: "surface",
  sourceFile: "families/layout/surface/surface.vue",
  render: () =>
    h(
      Surface,
      {
        ariaDescribedby: "runtime-surface-help",
        ariaLabelledby: "runtime-surface-title",
        as: "article",
        elevation: "floating",
        tone: "accent",
      },
      {
        default: () => [
          h("h2", { id: "runtime-surface-title" }, "Runtime surface"),
          h("p", { id: "runtime-surface-help" }, "Hydrates without replacement"),
        ],
      },
    ),
  assertServerMarkup(html) {
    assert.match(html, /^<article/);
    assert.match(html, /part="root"/);
    assert.match(html, /data-vize-ui="surface"/);
    assert.match(html, /aria-labelledby="runtime-surface-title"/);
    assert.match(html, /aria-describedby="runtime-surface-help"/);
    assert.match(html, /data-tone="accent"/);
    assert.match(html, /data-elevation="floating"/);
    assert.match(html, /Runtime surface/);
    assert.doesNotMatch(html, /class=|style=|role=|tabindex=|aria-hidden=|aria-live=/);
  },
  assertHydratedDom(host) {
    const surface = host.querySelector('[data-vize-ui="surface"]');

    assert.ok(surface instanceof HTMLElement);
    assert.equal(surface.tagName, "ARTICLE");
    assert.equal(surface.getAttribute("part"), "root");
    assert.equal(surface.getAttribute("aria-labelledby"), "runtime-surface-title");
    assert.equal(surface.getAttribute("aria-describedby"), "runtime-surface-help");
    assert.equal(surface.getAttribute("data-tone"), "accent");
    assert.equal(surface.getAttribute("data-elevation"), "floating");
    assert.equal(surface.getAttribute("role"), null);
    assert.equal(surface.getAttribute("tabindex"), null);
    assert.equal(surface.textContent, "Runtime surfaceHydrates without replacement");
  },
};
