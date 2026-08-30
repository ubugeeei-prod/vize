import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h, nextTick, ref } from "vue";

import type { StepperRootExpose } from "./stepper.ts";
import StepperContent from "./stepper-content.vue";
import StepperItem from "./stepper-item.vue";
import StepperList from "./stepper-list.vue";
import StepperRoot from "./stepper-root.vue";
import StepperTrigger from "./stepper-trigger.vue";
import { mountInteraction } from "../../../testing/mount.ts";

function mountStepper(
  props: Record<string, unknown> = {},
  shippingProps: Record<string, unknown> = {},
  billingProps: Record<string, unknown> = {},
  reviewProps: Record<string, unknown> = {},
) {
  return mountInteraction(StepperRoot, {
    props,
    record: ["update:modelValue", "change"],
    slots: {
      default: (state) => [
        h(
          "output",
          {
            "data-completed-values": state.completedValues.join("|"),
            "data-current-index": String(state.currentIndex),
            "data-root-state": state.state,
          },
          String(state.value ?? ""),
        ),
        h(StepperList, { ariaLabel: "Checkout steps" }, () => [
          h(
            StepperItem,
            { completed: true, textValue: "Shipping", value: "shipping", ...shippingProps },
            () =>
              h(
                StepperTrigger,
                {},
                {
                  default: ({ state }) => h("span", { "data-shipping-state": state }, "Shipping"),
                  indicator: ({ completed }) =>
                    h("span", { "data-shipping-indicator": completed ? "true" : "false" }),
                },
              ),
          ),
          h(StepperItem, { textValue: "Billing", value: "billing", ...billingProps }, () =>
            h(StepperTrigger, () => "Billing"),
          ),
          h(StepperItem, { textValue: "Review", value: "review", ...reviewProps }, () =>
            h(StepperTrigger, () => "Review"),
          ),
        ]),
        h(StepperContent, { value: "shipping" }, ({ state }) =>
          h("p", { "data-shipping-panel": state }, "Shipping panel"),
        ),
        h(StepperContent, { value: "billing" }, () => "Billing panel"),
        h(StepperContent, { value: "review" }, () => "Review panel"),
      ],
    },
  });
}

test("renders accessible stepper semantics with deterministic ids, slots, and data", async () => {
  const handle = mountStepper({ defaultValue: "shipping", id: "checkout" });
  await nextTick();
  const root = handle.root();
  const list = handle.getByRole("list", { name: "Checkout steps" });
  const shipping = handle.getByRole("button", { name: "Shipping" }) as HTMLButtonElement;
  const billing = handle.getByRole("button", { name: "Billing" }) as HTMLButtonElement;
  const shippingItem = shipping.closest("[data-vize-ui='stepper-item']");
  const shippingPanel = handle.getByRole("region", { name: "Shipping" }) as HTMLDivElement;
  const billingPanel = root.querySelector<HTMLDivElement>("#checkout-content-value-billing");

  assert.equal(root.id, "checkout");
  assert.equal(root.getAttribute("data-vize-ui"), "stepper-root");
  assert.equal(root.getAttribute("part"), "root");
  assert.equal(root.getAttribute("data-state"), "active");
  assert.equal(root.getAttribute("data-navigation-mode"), "linear");
  assert.equal(root.getAttribute("data-linear"), "true");
  assert.equal(root.getAttribute("data-value"), "shipping");
  assert.equal(root.getAttribute("data-current-index"), "0");
  assert.equal(root.getAttribute("data-completed-count"), "1");
  assert.equal(list.id, "checkout-list");
  assert.equal(list.getAttribute("aria-orientation"), "horizontal");
  assert.equal(list.getAttribute("part"), "list");
  assert.equal(shippingItem?.id, "checkout-item-value-shipping");
  assert.equal(shippingItem?.getAttribute("role"), "listitem");
  assert.equal(shippingItem?.getAttribute("data-current"), "true");
  assert.equal(shippingItem?.getAttribute("data-completed"), "true");
  assert.equal(shipping.id, "checkout-trigger-value-shipping");
  assert.equal(shipping.type, "button");
  assert.equal(shipping.getAttribute("aria-current"), "step");
  assert.equal(shipping.getAttribute("aria-controls"), "checkout-content-value-shipping");
  assert.equal(shipping.getAttribute("tabindex"), "0");
  assert.equal(shipping.getAttribute("data-state"), "current");
  assert.equal(billing.getAttribute("aria-current"), null);
  assert.equal(billing.getAttribute("aria-disabled"), null);
  assert.equal(billing.getAttribute("data-selectable"), "true");
  assert.equal(
    shipping.querySelector("[data-shipping-state]")?.getAttribute("data-shipping-state"),
    "current",
  );
  assert.equal(
    shipping.querySelector("[data-shipping-indicator]")?.getAttribute("data-shipping-indicator"),
    "true",
  );
  assert.equal(shippingPanel.id, "checkout-content-value-shipping");
  assert.equal(shippingPanel.hidden, false);
  assert.equal(shippingPanel.tabIndex, 0);
  assert.equal(shippingPanel.getAttribute("aria-labelledby"), "checkout-trigger-value-shipping");
  assert.equal(shippingPanel.getAttribute("data-state"), "active");
  assert.equal(billingPanel?.hidden, true);
  assert.equal(billingPanel?.getAttribute("data-state"), "inactive");
  assert.equal(
    root.querySelector("[data-root-state]")?.getAttribute("data-completed-values"),
    "shipping",
  );
  handle.unmount();
});

test("linear navigation prevents future activation until prior enabled steps are complete", async () => {
  const locked = mountStepper({}, { completed: false });
  await nextTick();
  const root = locked.root();
  const billing = locked.getByRole("button", { name: "Billing" }) as HTMLButtonElement;

  assert.equal(root.getAttribute("data-value"), "shipping");
  assert.equal(billing.disabled, false);
  assert.equal(billing.getAttribute("aria-disabled"), "true");
  assert.equal(billing.getAttribute("data-locked"), "true");
  await locked.click(billing);
  assert.equal(root.getAttribute("data-value"), "shipping");
  assert.equal(locked.wrapper.emitted("update:modelValue"), undefined);
  locked.unmount();

  const unlocked = mountStepper();
  await nextTick();
  const unlockedBilling = unlocked.getByRole("button", {
    name: "Billing",
  }) as HTMLButtonElement;

  assert.equal(unlockedBilling.getAttribute("aria-disabled"), null);
  await unlocked.click(unlockedBilling);
  assert.equal(unlocked.root().getAttribute("data-value"), "billing");
  assert.deepEqual(unlocked.wrapper.emitted("update:modelValue"), [["billing"]]);
  assert.deepEqual(unlocked.wrapper.emitted("change")?.[0]?.slice(0, 2), ["billing", "shipping"]);
  assert.ok(unlocked.wrapper.emitted("change")?.[0]?.[2] instanceof MouseEvent);
  unlocked.unmount();
});

test("reset restores the configured default value even when linear activation is locked", async () => {
  let rootExpose: StepperRootExpose | null = null;
  const Probe = defineComponent({
    name: "StepperResetProbe",
    setup: () => () =>
      h(
        StepperRoot,
        {
          defaultValue: "review",
          id: "reset-stepper",
          ref: (value) => {
            rootExpose = value as StepperRootExpose | null;
          },
        },
        () => [
          h(StepperList, { ariaLabel: "Reset steps" }, () => [
            h(StepperItem, { completed: true, value: "shipping" }, () =>
              h(StepperTrigger, () => "Shipping"),
            ),
            h(StepperItem, { value: "billing" }, () => h(StepperTrigger, () => "Billing")),
            h(StepperItem, { value: "review" }, () => h(StepperTrigger, () => "Review")),
          ]),
          h(StepperContent, { value: "shipping" }, () => "Shipping panel"),
          h(StepperContent, { value: "billing" }, () => "Billing panel"),
          h(StepperContent, { value: "review" }, () => "Review panel"),
        ],
      ),
  });
  const handle = mountInteraction(Probe);
  await nextTick();

  if (rootExpose === null) assert.fail("Stepper root ref must expose reset");
  assert.equal(rootExpose.value, "review");
  assert.equal(rootExpose.setValue("billing"), true);
  await nextTick();
  assert.equal(rootExpose.value, "billing");
  assert.equal(rootExpose.setValue("review"), false);
  assert.equal(rootExpose.reset(), true);
  await nextTick();
  assert.equal(rootExpose.value, "review");
  assert.equal(handle.root().getAttribute("data-value"), "review");
  handle.unmount();
});

test("items reregister when their step value changes", async () => {
  let renameStep: (value: string) => void = () => assert.fail("renameStep must be assigned");
  const Probe = defineComponent({
    name: "StepperDynamicValueProbe",
    setup() {
      const dynamicValue = ref("billing");
      renameStep = (value) => {
        dynamicValue.value = value;
      };
      return () =>
        h(StepperRoot, { id: "dynamic-stepper", navigationMode: "free" }, () => [
          h(StepperList, { ariaLabel: "Dynamic steps" }, () => [
            h(StepperItem, { completed: true, value: "shipping" }, () =>
              h(StepperTrigger, () => "Shipping"),
            ),
            h(StepperItem, { textValue: "Variable", value: dynamicValue.value }, () =>
              h(StepperTrigger, () => "Variable"),
            ),
          ]),
          h(StepperContent, { value: "shipping" }, () => "Shipping panel"),
          h(StepperContent, { value: dynamicValue.value }, () => "Variable panel"),
        ]);
    },
  });
  const handle = mountInteraction(Probe);
  await nextTick();
  const root = handle.root();
  const variable = handle.getByRole("button", { name: "Variable" });

  assert.equal(variable.id, "dynamic-stepper-trigger-value-billing");
  await handle.click(variable);
  assert.equal(root.getAttribute("data-value"), "billing");

  renameStep("invoice");
  await nextTick();
  await nextTick();
  const renamed = handle.getByRole("button", { name: "Variable" });

  assert.equal(root.querySelector("[data-value='billing']"), null);
  assert.equal(renamed.id, "dynamic-stepper-trigger-value-invoice");
  assert.equal(root.getAttribute("data-value"), "shipping");
  await handle.click(renamed);
  assert.equal(root.getAttribute("data-value"), "invoice");
  handle.unmount();
});

test("free navigation and roving focus can activate any enabled step while skipping disabled", async () => {
  const handle = mountStepper({ navigationMode: "free" }, { completed: false }, { disabled: true });
  const shipping = handle.getByRole("button", { name: "Shipping" });
  const billing = handle.getByRole("button", { name: "Billing" }) as HTMLButtonElement;
  const review = handle.getByRole("button", { name: "Review" });

  assert.ok((await handle.tab()) === shipping);
  assert.equal(billing.disabled, true);
  assert.equal(billing.getAttribute("data-disabled"), "true");
  const arrow = await handle.press(shipping, "ArrowRight");
  assert.equal(arrow.keydownPrevented, true);
  assert.ok(handle.activeElement() === review);
  assert.equal(handle.root().getAttribute("data-value"), "shipping");

  const enter = await handle.press(review, "Enter");
  assert.equal(enter.activated, true);
  assert.equal(handle.root().getAttribute("data-value"), "review");
  assert.deepEqual(handle.wrapper.emitted("update:modelValue"), [["review"]]);
  handle.unmount();
});

test("controlled current value wins until the parent accepts the request", async () => {
  const handle = mountStepper({ modelValue: "shipping" });
  const shipping = handle.getByRole("button", { name: "Shipping" });
  const billing = handle.getByRole("button", { name: "Billing" });

  await handle.click(billing);

  assert.deepEqual(handle.wrapper.emitted("update:modelValue"), [["billing"]]);
  assert.deepEqual(handle.wrapper.emitted("change")?.[0]?.slice(0, 2), ["billing", "shipping"]);
  assert.equal(shipping.getAttribute("aria-current"), "step");
  assert.equal(billing.getAttribute("aria-current"), null);

  await handle.wrapper.setProps({ modelValue: "billing" });
  assert.equal(shipping.getAttribute("aria-current"), null);
  assert.equal(billing.getAttribute("aria-current"), "step");
  handle.unmount();
});

test("disabled roots and items suppress user activation and sequential focus", async () => {
  const rootDisabled = mountStepper({ defaultValue: "shipping", disabled: true });
  const root = rootDisabled.root();
  const shipping = rootDisabled.getByRole("button", { name: "Shipping" }) as HTMLButtonElement;
  const billing = rootDisabled.getByRole("button", { name: "Billing" }) as HTMLButtonElement;

  assert.equal(root.getAttribute("data-state"), "disabled");
  assert.equal(shipping.disabled, true);
  assert.equal(billing.disabled, true);
  assert.notEqual(await rootDisabled.tab(), shipping);
  await rootDisabled.click(billing);
  assert.equal(root.getAttribute("data-value"), "shipping");
  assert.equal(rootDisabled.wrapper.emitted("update:modelValue"), undefined);
  rootDisabled.unmount();

  const itemDisabled = mountStepper({}, {}, { disabled: true });
  await nextTick();
  const disabledBilling = itemDisabled.getByRole("button", {
    name: "Billing",
  }) as HTMLButtonElement;
  const review = itemDisabled.getByRole("button", { name: "Review" }) as HTMLButtonElement;

  assert.equal(disabledBilling.disabled, true);
  assert.equal(disabledBilling.getAttribute("data-state"), "disabled");
  await itemDisabled.click(disabledBilling);
  assert.equal(itemDisabled.wrapper.emitted("update:modelValue"), undefined);
  assert.equal(review.getAttribute("aria-disabled"), null);
  itemDisabled.unmount();
});
