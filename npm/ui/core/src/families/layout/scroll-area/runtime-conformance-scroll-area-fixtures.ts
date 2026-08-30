import assert from "node:assert/strict";

import { h } from "vue";

import type { RuntimeFixture } from "../../../runtime-conformance-fixtures.ts";
import ScrollArea from "./scroll-area.vue";

export const scrollAreaRuntimeFixture: RuntimeFixture = {
  name: "scroll-area",
  sourceFile: "families/layout/scroll-area/scroll-area.vue",
  render: () =>
    h(
      ScrollArea,
      {
        ariaDescribedby: "runtime-scroll-area-help",
        ariaLabelledby: "runtime-scroll-area-title",
        as: "section",
        blockSize: 256,
        dir: "rtl",
        focusable: true,
        orientation: "both",
        overscrollBehavior: "contain",
        scrollbarGutter: "stable",
        scrollbarWidth: "thin",
      },
      {
        default: () => [
          h("h2", { id: "runtime-scroll-area-title" }, "Runtime scroll area"),
          h("p", { id: "runtime-scroll-area-help" }, "Hydrates without replacement"),
        ],
      },
    ),
  assertServerMarkup(html) {
    assert.match(html, /^<section/);
    assert.match(html, /part="root"/);
    assert.match(html, /data-vize-ui="scroll-area"/);
    assert.match(html, /data-state="scrollable"/);
    assert.match(html, /data-orientation="both"/);
    assert.match(html, /data-dir="rtl"/);
    assert.match(html, /data-focusable="true"/);
    assert.match(html, /--vize-ui-scroll-area-block-size:256px/);
    assert.match(html, /--vize-ui-scroll-area-overflow-x:auto/);
    assert.match(html, /--vize-ui-scroll-area-overflow-y:auto/);
    assert.match(html, /<div[^>]*data-vize-ui="scroll-area-viewport"/);
    assert.match(html, /role="region"/);
    assert.match(html, /tabindex="0"/);
    assert.match(html, /aria-labelledby="runtime-scroll-area-title"/);
    assert.match(html, /aria-describedby="runtime-scroll-area-help"/);
    assert.match(html, /Runtime scroll area/);
    assert.doesNotMatch(html, /id="vize/);
    assert.doesNotMatch(html, /aria-hidden=|aria-live=/);
  },
  assertHydratedDom(host) {
    const root = host.querySelector('[data-vize-ui="scroll-area"]');
    const viewport = host.querySelector('[data-vize-ui="scroll-area-viewport"]');

    assert.ok(root instanceof HTMLElement);
    assert.ok(viewport instanceof HTMLDivElement);
    assert.equal(root.tagName, "SECTION");
    assert.equal(root.getAttribute("part"), "root");
    assert.equal(root.getAttribute("data-orientation"), "both");
    assert.equal(root.getAttribute("data-dir"), "rtl");
    assert.equal(root.getAttribute("data-focusable"), "true");
    assert.equal(root.style.getPropertyValue("--vize-ui-scroll-area-block-size"), "256px");
    assert.equal(root.style.getPropertyValue("--vize-ui-scroll-area-overflow-x"), "auto");
    assert.equal(root.style.getPropertyValue("--vize-ui-scroll-area-overflow-y"), "auto");
    assert.equal(viewport.getAttribute("part"), "viewport");
    assert.equal(viewport.getAttribute("role"), "region");
    assert.equal(viewport.getAttribute("tabindex"), "0");
    assert.equal(viewport.getAttribute("aria-labelledby"), "runtime-scroll-area-title");
    assert.equal(viewport.getAttribute("aria-describedby"), "runtime-scroll-area-help");
    assert.equal(viewport.textContent, "Runtime scroll areaHydrates without replacement");
  },
};
