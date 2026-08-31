import assert from "node:assert/strict";

import { h } from "vue";

import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";
import { ShareButton } from "./share-button.ts";
import type { ShareButtonAction, ShareButtonSlotState } from "./share-button.ts";

const inertAction: ShareButtonAction = () => {};

function renderShareButtonFixture() {
  return h(
    ShareButton,
    {
      action: inertAction,
      ariaLabel: "Share docs",
      idleLabel: "Share docs",
      text: "Read the Vize docs",
      title: "Vize docs",
      url: "https://vize.dev/docs",
    },
    {
      default: ({ label, state }: ShareButtonSlotState) =>
        h("span", { "data-rendered-state": state }, label),
    },
  );
}

function assertShareButtonServerMarkup(html: string): void {
  assert.match(html, /^<button/);
  assert.match(html, /type="button"/);
  assert.match(html, /aria-label="Share docs"/);
  assert.match(html, /data-vize-ui="share-button"/);
  assert.match(html, /part="root"/);
  assert.match(html, /data-state="idle"/);
  assert.match(html, /data-rendered-state="idle"/);
  assert.match(html, /Share docs/);
  assert.doesNotMatch(
    html,
    /data-sharing=|data-disabled=|aria-busy=|data-title=|data-text=|data-url=|data-files=/,
  );
}

function assertShareButtonHydratedDom(host: HTMLElement): void {
  const button = host.querySelector('[data-vize-ui="share-button"]');
  const label = host.querySelector("[data-rendered-state]");

  assert.ok(button instanceof HTMLButtonElement);
  assert.equal(button.type, "button");
  assert.equal(button.getAttribute("aria-label"), "Share docs");
  assert.equal(button.getAttribute("data-state"), "idle");
  assert.equal(button.getAttribute("data-sharing"), null);
  assert.equal(button.getAttribute("data-title"), null);
  assert.equal(button.getAttribute("data-url"), null);
  assert.ok(label instanceof HTMLSpanElement);
  assert.equal(label.textContent, "Share docs");
}

export const shareButtonRuntimeFixture: RuntimeFixture = {
  name: "share-button",
  sourceFile: "families/actions/share-button/share-button.vue",
  render: renderShareButtonFixture,
  assertServerMarkup: assertShareButtonServerMarkup,
  assertHydratedDom: assertShareButtonHydratedDom,
};
