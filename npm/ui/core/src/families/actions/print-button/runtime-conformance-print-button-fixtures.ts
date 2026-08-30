import assert from "node:assert/strict";

import { h } from "vue";

import type { RuntimeFixture } from "../../../runtime-conformance-fixtures.ts";
import { PrintButton } from "./print-button.ts";
import type { PrintButtonSlotState } from "./print-button.ts";

function renderPrintButtonFixture() {
  return h(
    PrintButton,
    {
      ariaLabel: "Print invoice",
      idleLabel: "Print invoice",
      action: () => {},
    },
    {
      default: ({ label, state }: PrintButtonSlotState) =>
        h("span", { "data-rendered-state": state }, label),
    },
  );
}

function assertPrintButtonServerMarkup(html: string): void {
  assert.match(html, /^<button/);
  assert.match(html, /type="button"/);
  assert.match(html, /aria-label="Print invoice"/);
  assert.match(html, /data-vize-ui="print-button"/);
  assert.match(html, /part="root"/);
  assert.match(html, /data-state="idle"/);
  assert.match(html, /data-rendered-state="idle"/);
  assert.match(html, /Print invoice/);
  assert.doesNotMatch(html, /data-printing=|data-disabled=|aria-busy=|data-action=/);
}

function assertPrintButtonHydratedDom(host: HTMLElement): void {
  const button = host.querySelector('[data-vize-ui="print-button"]');
  const label = host.querySelector("[data-rendered-state]");

  assert.ok(button instanceof HTMLButtonElement);
  assert.equal(button.type, "button");
  assert.equal(button.getAttribute("aria-label"), "Print invoice");
  assert.equal(button.getAttribute("data-state"), "idle");
  assert.equal(button.getAttribute("data-printing"), null);
  assert.equal(button.getAttribute("data-action"), null);
  assert.ok(label instanceof HTMLSpanElement);
  assert.equal(label.textContent, "Print invoice");
}

export const printButtonRuntimeFixture: RuntimeFixture = {
  name: "print-button",
  sourceFile: "families/actions/print-button/print-button.vue",
  render: renderPrintButtonFixture,
  assertServerMarkup: assertPrintButtonServerMarkup,
  assertHydratedDom: assertPrintButtonHydratedDom,
};
