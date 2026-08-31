import assert from "node:assert/strict";

import { h } from "vue";

import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";
import {
  StepperContent,
  StepperItem,
  StepperList,
  StepperRoot,
  StepperTrigger,
} from "./stepper.ts";
import type {
  StepperContentSlotState,
  StepperItemSlotState,
  StepperListSlotState,
  StepperTriggerSlotState,
} from "./stepper-types.ts";

function renderCheckoutStepper() {
  return h(StepperRoot, { defaultValue: "shipping", id: "checkout-stepper" }, () => [
    h(StepperList, { ariaLabel: "Checkout steps" }, () => [
      h(StepperItem, { completed: true, value: "shipping" }, () =>
        h(StepperTrigger, () => "Shipping"),
      ),
      h(StepperItem, { value: "billing" }, () => h(StepperTrigger, () => "Billing")),
    ]),
    h(StepperContent, { value: "shipping" }, () => "Shipping panel"),
    h(StepperContent, { value: "billing" }, () => "Billing panel"),
  ]);
}

function assertCheckoutStepperServerMarkup(html: string): void {
  assert.match(html, /id="checkout-stepper"/);
  assert.match(html, /data-vize-ui="stepper-root"/);
  assert.match(html, /role="list"/);
  assert.match(html, /aria-label="Checkout steps"/);
  assert.match(html, /data-vize-ui="stepper-item"/);
  assert.match(html, /id="checkout-stepper-trigger-value-shipping"/);
  assert.match(html, /aria-current="step"/);
  assert.match(html, /aria-controls="checkout-stepper-content-value-shipping"/);
  assert.match(html, /role="region"/);
  assert.match(html, /aria-labelledby="checkout-stepper-trigger-value-shipping"/);
  assert.match(html, /id="checkout-stepper-content-value-billing"[^>]*hidden/);
}

function assertCheckoutStepperHydratedDom(host: HTMLElement): void {
  const root = host.querySelector('[data-vize-ui="stepper-root"]');
  const list = host.querySelector('[data-vize-ui="stepper-list"]');
  const shipping = host.querySelector('[data-vize-ui="stepper-trigger"][data-value="shipping"]');
  const shippingPanel = host.querySelector(
    '[data-vize-ui="stepper-content"][data-value="shipping"]',
  );
  const billingPanel = host.querySelector('[data-vize-ui="stepper-content"][data-value="billing"]');

  assert.ok(root instanceof HTMLDivElement);
  assert.equal(root.id, "checkout-stepper");
  assert.ok(list instanceof HTMLDivElement);
  assert.equal(list.getAttribute("role"), "list");
  assert.ok(shipping instanceof HTMLButtonElement);
  assert.equal(shipping.getAttribute("aria-current"), "step");
  assert.equal(shipping.getAttribute("aria-controls"), "checkout-stepper-content-value-shipping");
  assert.ok(shippingPanel instanceof HTMLDivElement);
  assert.equal(shippingPanel.hidden, false);
  assert.equal(
    shippingPanel.getAttribute("aria-labelledby"),
    "checkout-stepper-trigger-value-shipping",
  );
  assert.ok(billingPanel instanceof HTMLDivElement);
  assert.equal(billingPanel.hidden, true);
}

export const stepperRuntimeFixtures: readonly RuntimeFixture[] = [
  {
    name: "stepper",
    sourceFile: "families/navigation/stepper/stepper-root.vue",
    render: renderCheckoutStepper,
    assertServerMarkup: assertCheckoutStepperServerMarkup,
    assertHydratedDom: assertCheckoutStepperHydratedDom,
  },
  {
    name: "stepper-content",
    sourceFile: "families/navigation/stepper/stepper-content.vue",
    render: () =>
      h(StepperRoot, { defaultValue: "review", id: "content-stepper" }, () => [
        h(StepperList, null, () =>
          h(StepperItem, { value: "review" }, () => h(StepperTrigger, () => "Review")),
        ),
        h(
          StepperContent,
          { value: "review" },
          {
            default: ({ active, state }: StepperContentSlotState) => `${active}:${state}`,
          },
        ),
      ]),
    assertServerMarkup(html) {
      assert.match(html, /id="content-stepper-content-value-review"/);
      assert.match(html, /role="region"/);
      assert.match(html, /aria-labelledby="content-stepper-trigger-value-review"/);
      assert.match(html, /data-state="active"/);
      assert.match(html, /true:active/);
      assert.doesNotMatch(html, /hidden/);
    },
    assertHydratedDom(host) {
      const content = host.querySelector('[data-vize-ui="stepper-content"]');
      assert.ok(content instanceof HTMLDivElement);
      assert.equal(content.hidden, false);
      assert.equal(content.getAttribute("data-state"), "active");
      assert.equal(content.textContent, "true:active");
    },
  },
  {
    name: "stepper-item",
    sourceFile: "families/navigation/stepper/stepper-item.vue",
    render: () =>
      h(StepperRoot, { defaultValue: "shipping", id: "item-stepper" }, () =>
        h(StepperList, null, () =>
          h(
            StepperItem,
            { completed: true, value: "shipping" },
            {
              default: ({ current, state }: StepperItemSlotState) => `${current}:${state}`,
            },
          ),
        ),
      ),
    assertServerMarkup(html) {
      assert.match(html, /id="item-stepper-item-value-shipping"/);
      assert.match(html, /role="listitem"/);
      assert.match(html, /data-state="current"/);
      assert.match(html, /data-completed="true"/);
      assert.match(html, /true:current/);
    },
    assertHydratedDom(host) {
      const item = host.querySelector('[data-vize-ui="stepper-item"]');
      assert.ok(item instanceof HTMLDivElement);
      assert.equal(item.getAttribute("role"), "listitem");
      assert.equal(item.getAttribute("data-state"), "current");
      assert.equal(item.textContent, "true:current");
    },
  },
  {
    name: "stepper-list",
    sourceFile: "families/navigation/stepper/stepper-list.vue",
    render: () =>
      h(
        StepperRoot,
        {
          defaultValue: "shipping",
          id: "list-stepper",
          navigationMode: "free",
          orientation: "vertical",
        },
        () =>
          h(
            StepperList,
            { ariaLabel: "Vertical steps" },
            {
              default: ({ listId, navigationMode }: StepperListSlotState) =>
                `${listId}:${navigationMode}`,
            },
          ),
      ),
    assertServerMarkup(html) {
      assert.match(html, /id="list-stepper-list"/);
      assert.match(html, /role="list"/);
      assert.match(html, /aria-label="Vertical steps"/);
      assert.match(html, /aria-orientation="vertical"/);
      assert.match(html, /data-navigation-mode="free"/);
      assert.match(html, /list-stepper-list:free/);
    },
    assertHydratedDom(host) {
      const list = host.querySelector('[data-vize-ui="stepper-list"]');
      assert.ok(list instanceof HTMLDivElement);
      assert.equal(list.getAttribute("role"), "list");
      assert.equal(list.getAttribute("aria-orientation"), "vertical");
      assert.equal(list.textContent, "list-stepper-list:free");
    },
  },
  {
    name: "stepper-trigger",
    sourceFile: "families/navigation/stepper/stepper-trigger.vue",
    render: () =>
      h(StepperRoot, { defaultValue: "review", id: "trigger-stepper" }, () =>
        h(StepperList, null, () =>
          h(StepperItem, { value: "review" }, () =>
            h(StepperTrigger, null, {
              default: ({ current, state }: StepperTriggerSlotState) => `${current}:${state}`,
            }),
          ),
        ),
      ),
    assertServerMarkup(html) {
      assert.match(html, /id="trigger-stepper-trigger-value-review"/);
      assert.match(html, /aria-current="step"/);
      assert.match(html, /aria-controls="trigger-stepper-content-value-review"/);
      assert.match(html, /data-state="current"/);
      assert.match(html, /true:current/);
    },
    assertHydratedDom(host) {
      const trigger = host.querySelector('[data-vize-ui="stepper-trigger"]');
      assert.ok(trigger instanceof HTMLButtonElement);
      assert.equal(trigger.getAttribute("aria-current"), "step");
      assert.equal(trigger.getAttribute("data-state"), "current");
      assert.equal(trigger.textContent, "true:current");
    },
  },
];
