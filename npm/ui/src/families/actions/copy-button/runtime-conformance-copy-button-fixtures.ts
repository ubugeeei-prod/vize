import assert from "node:assert/strict";

import { h } from "vue";

import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";
import { CopyButton } from "./copy-button.ts";
import type { CopyButtonSlotState } from "./copy-button.ts";

function renderCopyButtonFixture() {
  return h(
    CopyButton,
    {
      ariaLabel: "Copy invite link",
      idleLabel: "Copy invite",
      value: "https://vize.dev/invite",
      writer: () => {},
    },
    {
      default: ({ label, state }: CopyButtonSlotState) =>
        h("span", { "data-rendered-state": state }, label),
    },
  );
}

function assertCopyButtonServerMarkup(html: string): void {
  assert.match(html, /^<button/);
  assert.match(html, /type="button"/);
  assert.match(html, /aria-label="Copy invite link"/);
  assert.match(html, /data-vize-ui="copy-button"/);
  assert.match(html, /part="root"/);
  assert.match(html, /data-state="idle"/);
  assert.match(html, /data-rendered-state="idle"/);
  assert.match(html, /Copy invite/);
  assert.doesNotMatch(html, /data-writing=|data-disabled=|aria-busy=|data-value=/);
}

function assertCopyButtonHydratedDom(host: HTMLElement): void {
  const button = host.querySelector('[data-vize-ui="copy-button"]');
  const label = host.querySelector("[data-rendered-state]");

  assert.ok(button instanceof HTMLButtonElement);
  assert.equal(button.type, "button");
  assert.equal(button.getAttribute("aria-label"), "Copy invite link");
  assert.equal(button.getAttribute("data-state"), "idle");
  assert.equal(button.getAttribute("data-writing"), null);
  assert.equal(button.getAttribute("data-value"), null);
  assert.ok(label instanceof HTMLSpanElement);
  assert.equal(label.textContent, "Copy invite");
}

export const copyButtonRuntimeFixture: RuntimeFixture = {
  name: "copy-button",
  sourceFile: "families/actions/copy-button/copy-button.vue",
  render: renderCopyButtonFixture,
  assertServerMarkup: assertCopyButtonServerMarkup,
  assertHydratedDom: assertCopyButtonHydratedDom,
};
