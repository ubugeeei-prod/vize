import assert from "node:assert/strict";

import { h } from "vue";

import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";
import { FullscreenButton } from "./fullscreen-button.ts";
import type { FullscreenButtonController, FullscreenButtonSlotState } from "./fullscreen-button.ts";

const inertController: FullscreenButtonController = {
  getFullscreenElement: () => null,
  requestFullscreen: () => {},
  exitFullscreen: () => {},
};

function renderFullscreenButtonFixture() {
  return h(
    FullscreenButton,
    {
      ariaLabel: "Toggle fullscreen",
      controller: inertController,
      enterLabel: "Enter fullscreen",
    },
    {
      default: ({ label, state }: FullscreenButtonSlotState) =>
        h("span", { "data-rendered-state": state }, label),
    },
  );
}

function assertFullscreenButtonServerMarkup(html: string): void {
  assert.match(html, /^<button/);
  assert.match(html, /type="button"/);
  assert.match(html, /aria-label="Toggle fullscreen"/);
  assert.match(html, /aria-pressed="false"/);
  assert.match(html, /data-vize-ui="fullscreen-button"/);
  assert.match(html, /part="root"/);
  assert.match(html, /data-state="idle"/);
  assert.match(html, /data-rendered-state="idle"/);
  assert.match(html, /Enter fullscreen/);
  assert.doesNotMatch(html, /data-pending=|data-active=|data-disabled=|aria-busy=|data-target=/);
}

function assertFullscreenButtonHydratedDom(host: HTMLElement): void {
  const button = host.querySelector('[data-vize-ui="fullscreen-button"]');
  const label = host.querySelector("[data-rendered-state]");

  assert.ok(button instanceof HTMLButtonElement);
  assert.equal(button.type, "button");
  assert.equal(button.getAttribute("aria-label"), "Toggle fullscreen");
  assert.equal(button.getAttribute("aria-pressed"), "false");
  assert.equal(button.getAttribute("data-state"), "idle");
  assert.equal(button.getAttribute("data-pending"), null);
  assert.equal(button.getAttribute("data-target"), null);
  assert.ok(label instanceof HTMLSpanElement);
  assert.equal(label.textContent, "Enter fullscreen");
}

export const fullscreenButtonRuntimeFixture: RuntimeFixture = {
  name: "fullscreen-button",
  sourceFile: "families/actions/fullscreen-button/fullscreen-button.vue",
  render: renderFullscreenButtonFixture,
  assertServerMarkup: assertFullscreenButtonServerMarkup,
  assertHydratedDom: assertFullscreenButtonHydratedDom,
};
