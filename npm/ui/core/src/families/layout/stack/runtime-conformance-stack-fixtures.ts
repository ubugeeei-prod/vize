import assert from "node:assert/strict";

import { h } from "vue";

import type { RuntimeFixture } from "../../../runtime-conformance-fixtures.ts";
import Stack from "./stack.vue";

export const stackRuntimeFixture: RuntimeFixture = {
  name: "stack",
  sourceFile: "families/layout/stack/stack.vue",
  render: () =>
    h(
      Stack,
      {
        align: "center",
        as: "section",
        axis: "inline",
        gap: "2rem",
        justify: "space-between",
      },
      {
        default: () => [h("span", "Alpha"), h("span", "Beta")],
      },
    ),
  assertServerMarkup(html) {
    assert.match(html, /^<section/);
    assert.match(html, /part="root"/);
    assert.match(html, /data-vize-ui="stack"/);
    assert.match(html, /data-state="stacked"/);
    assert.match(html, /data-axis="inline"/);
    assert.match(html, /data-reversed="false"/);
    assert.match(html, /data-vize-stack-direction="row"/);
    assert.match(html, /data-vize-stack-gap="2rem"/);
    assert.match(html, /data-vize-stack-align="center"/);
    assert.match(html, /data-vize-stack-justify="space-between"/);
    assert.match(html, /--vize-ui-stack-gap:2rem/);
    assert.match(html, /flex-direction:row/);
    assert.match(html, /<span>Alpha<\/span><span>Beta<\/span>/);
  },
  assertHydratedDom(host) {
    const stack = host.querySelector('[data-vize-ui="stack"]');
    assert.ok(stack instanceof HTMLElement);
    assert.equal(stack.getAttribute("role"), null);
    assert.equal(stack.getAttribute("aria-hidden"), null);
    assert.equal(stack.getAttribute("tabindex"), null);
    assert.equal(stack.getAttribute("part"), "root");
    assert.equal(stack.getAttribute("data-axis"), "inline");
    assert.equal(stack.getAttribute("data-reversed"), "false");
    assert.equal(stack.style.getPropertyValue("--vize-ui-stack-gap"), "2rem");
    assert.equal(stack.style.display, "flex");
    assert.equal(stack.style.flexDirection, "row");
    assert.equal(stack.textContent, "AlphaBeta");
  },
};
