import assert from "node:assert/strict";

import { h } from "vue";
import type { VNode } from "vue";

import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";
import {
  TourArrow,
  TourClose,
  TourContent,
  TourDescription,
  TourNext,
  TourPrev,
  TourProgress,
  TourRoot,
  TourSpotlight,
  TourStep,
  TourTitle,
} from "./tour.ts";

const steps = [{ value: "welcome" }, { value: "search", target: "#runtime-tour-search" }] as const;

function tour(children: () => unknown): VNode {
  return h(TourRoot, { defaultOpen: true, id: "runtime-tour", steps }, children);
}

function inContent(children: () => unknown): VNode {
  return tour(() => h(TourContent, { portalDisabled: true }, children));
}

function part(host: HTMLElement, name: string): HTMLElement {
  const element = host.querySelector(`[data-vize-ui="tour-${name}"]`);
  assert.ok(element instanceof HTMLElement, `tour-${name} must hydrate`);
  return element;
}

export const tourRuntimeFixtures: readonly RuntimeFixture[] = [
  {
    name: "tour-root",
    sourceFile: "families/overlays/tour/tour-root.vue",
    render: () => tour(() => "Tour"),
    assertServerMarkup(html) {
      assert.match(html, /id="runtime-tour"/);
      assert.match(html, /data-vize-ui="tour-root"/);
      assert.match(html, /data-state="open"/);
      assert.match(html, /data-step="welcome"/);
      assert.match(html, /data-target="none"/);
    },
    assertHydratedDom(host) {
      const root = part(host, "root");
      assert.equal(root.id, "runtime-tour");
      assert.equal(root.getAttribute("data-state"), "open");
    },
  },
  {
    name: "tour-content",
    sourceFile: "families/overlays/tour/tour-content.vue",
    render: () => inContent(() => "Welcome aboard"),
    assertServerMarkup(html) {
      assert.match(html, /data-vize-ui="tour-content"/);
      assert.match(html, /role="dialog"/);
      assert.match(html, /id="runtime-tour-content"/);
      assert.match(html, /aria-labelledby="runtime-tour-title"/);
      assert.match(html, /aria-describedby="runtime-tour-description"/);
      assert.match(html, /data-placement="center"/);
      assert.match(html, /data-vize-dismissable-layer/);
    },
    assertHydratedDom(host) {
      const content = part(host, "content");
      assert.equal(content.getAttribute("role"), "dialog");
      assert.equal(content.textContent, "Welcome aboard");
    },
  },
  {
    name: "tour-title",
    sourceFile: "families/overlays/tour/tour-title.vue",
    render: () => inContent(() => h(TourTitle, null, () => "Welcome")),
    assertServerMarkup(html) {
      assert.match(html, /<h2 id="runtime-tour-title"[^>]*data-vize-ui="tour-title"/);
      assert.match(html, /Welcome/);
    },
    assertHydratedDom(host) {
      assert.equal(part(host, "title").tagName, "H2");
    },
  },
  {
    name: "tour-description",
    sourceFile: "families/overlays/tour/tour-description.vue",
    render: () => inContent(() => h(TourDescription, null, () => "A quick walkthrough")),
    assertServerMarkup(html) {
      assert.match(html, /id="runtime-tour-description"[^>]*data-vize-ui="tour-description"/);
      assert.match(html, /A quick walkthrough/);
    },
    assertHydratedDom(host) {
      assert.equal(part(host, "description").id, "runtime-tour-description");
    },
  },
  {
    name: "tour-step",
    sourceFile: "families/overlays/tour/tour-step.vue",
    render: () =>
      inContent(() => [
        h(TourStep, { value: "welcome" }, () => "Welcome copy"),
        h(TourStep, { value: "search" }, () => "Search copy"),
      ]),
    assertServerMarkup(html) {
      assert.match(html, /data-vize-ui="tour-step"[^>]*data-value="welcome"/);
      assert.match(html, /Welcome copy/);
      assert.doesNotMatch(html, /Search copy/);
    },
    assertHydratedDom(host) {
      assert.equal(part(host, "step").getAttribute("data-index"), "0");
    },
  },
  {
    name: "tour-progress",
    sourceFile: "families/overlays/tour/tour-progress.vue",
    render: () => inContent(() => h(TourProgress)),
    assertServerMarkup(html) {
      assert.match(html, /data-vize-ui="tour-progress"/);
      assert.match(html, /1 \/ 2/);
    },
    assertHydratedDom(host) {
      assert.equal(part(host, "progress").getAttribute("data-total"), "2");
    },
  },
  {
    name: "tour-prev",
    sourceFile: "families/overlays/tour/tour-prev.vue",
    render: () => inContent(() => h(TourPrev, null, () => "Back")),
    assertServerMarkup(html) {
      assert.match(html, /<button type="button" disabled/);
      assert.match(html, /data-vize-ui="tour-prev"/);
    },
    assertHydratedDom(host) {
      const prev = part(host, "prev");
      assert.ok(prev instanceof HTMLButtonElement);
      assert.equal(prev.disabled, true);
    },
  },
  {
    name: "tour-next",
    sourceFile: "families/overlays/tour/tour-next.vue",
    render: () => inContent(() => h(TourNext, null, () => "Next")),
    assertServerMarkup(html) {
      assert.match(html, /data-vize-ui="tour-next"/);
      assert.match(html, /type="button"/);
      assert.match(html, /Next/);
    },
    assertHydratedDom(host) {
      const next = part(host, "next");
      assert.ok(next instanceof HTMLButtonElement);
      assert.equal(next.disabled, false);
    },
  },
  {
    name: "tour-close",
    sourceFile: "families/overlays/tour/tour-close.vue",
    render: () => inContent(() => h(TourClose, { reason: "skip" }, () => "Skip tour")),
    assertServerMarkup(html) {
      assert.match(html, /data-vize-ui="tour-close"/);
      assert.match(html, /data-reason="skip"/);
    },
    assertHydratedDom(host) {
      assert.equal(part(host, "close").textContent, "Skip tour");
    },
  },
  {
    name: "tour-arrow",
    sourceFile: "families/overlays/tour/tour-arrow.vue",
    render: () => inContent(() => [h(TourArrow), "Welcome"]),
    assertServerMarkup(html) {
      assert.match(html, /data-vize-ui="tour-arrow"/);
      assert.match(html, /aria-hidden="true"/);
      assert.match(html, /data-side="center"/);
    },
    assertHydratedDom(host) {
      assert.equal(part(host, "arrow").hidden, true);
    },
  },
  {
    name: "tour-spotlight",
    sourceFile: "families/overlays/tour/tour-spotlight.vue",
    render: () => tour(() => h(TourSpotlight, { interactive: true })),
    assertServerMarkup(html) {
      assert.match(html, /data-vize-ui="tour-spotlight"/);
      assert.match(html, /aria-hidden="true"/);
      assert.match(html, /data-interactive="true"/);
    },
    assertHydratedDom(host) {
      assert.equal(part(host, "spotlight").hidden, false);
    },
  },
];
