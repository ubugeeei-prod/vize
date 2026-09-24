import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h, nextTick } from "vue";

import type { AccordionItemExpose, AccordionRootExpose } from "./accordion.ts";
import AccordionContent from "./accordion-content.vue";
import AccordionHeader from "./accordion-header.vue";
import AccordionItem from "./accordion-item.vue";
import AccordionRoot from "./accordion-root.vue";
import AccordionTrigger from "./accordion-trigger.vue";
import CollapsibleContent from "../collapsible/collapsible-content.vue";
import CollapsibleTrigger from "../collapsible/collapsible-trigger.vue";
import { mountInteraction } from "../../../testing/mount.ts";

interface ItemSpec {
  readonly value: string | number;
  readonly label: string;
  readonly disabled?: boolean;
}

const defaultItems: readonly ItemSpec[] = [
  { value: "shipping", label: "Shipping" },
  { value: "returns", label: "Returns" },
  { value: "warranty", label: "Warranty" },
];

function renderItems(items: readonly ItemSpec[], contentProps: Record<string, unknown> = {}) {
  return items.map((item) =>
    h(
      AccordionItem,
      { key: item.value, value: item.value, disabled: item.disabled },
      {
        default: () => [
          h(AccordionHeader, null, () =>
            h(AccordionTrigger, null, {
              default: ({ state }: { readonly state: string }) =>
                h("span", { "data-trigger-state": state }, item.label),
            }),
          ),
          h(AccordionContent, contentProps, () => h("p", `${item.label} details`)),
        ],
      },
    ),
  );
}

function mountAccordion(
  props: Record<string, unknown>,
  items: readonly ItemSpec[] = defaultItems,
  contentProps: Record<string, unknown> = {},
) {
  return mountInteraction(AccordionRoot, {
    props,
    slots: { default: () => renderItems(items, contentProps) },
  });
}

function trigger(handle: ReturnType<typeof mountAccordion>, name: string): HTMLButtonElement {
  const element = handle.getByRole("button", { name });
  assert.ok(element instanceof HTMLButtonElement);
  return element;
}

function panel(handle: ReturnType<typeof mountAccordion>, value: string): HTMLDivElement {
  const element = handle
    .root()
    .querySelector(`[data-vize-ui="accordion-content"][id$="item-${value}-content"]`);
  assert.ok(element instanceof HTMLDivElement);
  return element;
}

test("renders APG accordion semantics with deterministic ids and native headings", () => {
  const handle = mountAccordion({ type: "single", id: "faq", defaultValue: "shipping" });
  const root = handle.root();
  const shipping = trigger(handle, "Shipping");
  const returns = trigger(handle, "Returns");
  const shippingPanel = handle.getByRole("region", { name: "Shipping" });

  assert.equal(root.id, "faq");
  assert.equal(root.getAttribute("data-vize-ui"), "accordion-root");
  assert.equal(root.getAttribute("data-type"), "single");
  assert.equal(root.getAttribute("data-orientation"), "vertical");
  assert.equal(shipping.id, "faq-item-shipping-trigger");
  assert.equal(shipping.type, "button");
  assert.equal(shipping.getAttribute("aria-expanded"), "true");
  assert.equal(shipping.getAttribute("aria-controls"), "faq-item-shipping-content");
  assert.equal(shipping.parentElement?.tagName, "H3");
  assert.equal(shipping.parentElement?.getAttribute("data-vize-ui"), "accordion-header");
  assert.equal(returns.getAttribute("aria-expanded"), "false");
  assert.equal(shippingPanel.id, "faq-item-shipping-content");
  assert.equal(shippingPanel.getAttribute("aria-labelledby"), "faq-item-shipping-trigger");
  assert.equal(shippingPanel.hidden, false);
  assert.equal(panel(handle, "returns").hidden, true);
  assert.equal(
    shipping.querySelector("[data-trigger-state]")?.getAttribute("data-trigger-state"),
    "open",
  );
  assert.equal(root.querySelector('[data-vize-ui="accordion-item"]')?.id, "faq-item-shipping");

  handle.unmount();
});

test("single accordion switches items and locks the open item unless collapsible", async () => {
  const handle = mountAccordion({ type: "single", defaultValue: "shipping" });
  const shipping = trigger(handle, "Shipping");
  const returns = trigger(handle, "Returns");

  assert.equal(shipping.getAttribute("aria-disabled"), "true");
  assert.equal(shipping.getAttribute("data-locked"), "true");
  await handle.click(shipping);
  assert.equal(shipping.getAttribute("aria-expanded"), "true");
  assert.equal(handle.wrapper.emitted("update:modelValue"), undefined);

  await handle.click(returns);
  assert.equal(shipping.getAttribute("aria-expanded"), "false");
  assert.equal(shipping.getAttribute("aria-disabled"), null);
  assert.equal(returns.getAttribute("aria-expanded"), "true");
  assert.deepEqual(handle.wrapper.emitted("update:modelValue"), [["returns"]]);
  const change = handle.wrapper.emitted("value-change")?.[0];
  assert.deepEqual(change?.slice(0, 2), ["returns", "shipping"]);
  assert.ok(change?.[2] instanceof MouseEvent);

  handle.unmount();
});

test("collapsible single accordion collapses the open item to null", async () => {
  const handle = mountAccordion({ type: "single", collapsible: true, defaultValue: "returns" });
  const returns = trigger(handle, "Returns");

  assert.equal(returns.getAttribute("aria-disabled"), null);
  await handle.click(returns);
  assert.equal(returns.getAttribute("aria-expanded"), "false");
  assert.equal(panel(handle, "returns").hidden, true);
  assert.deepEqual(handle.wrapper.emitted("update:modelValue"), [[null]]);

  handle.unmount();
});

test("multiple accordion keeps independent items open and emits readonly lists", async () => {
  const handle = mountAccordion({ type: "multiple", defaultValue: ["shipping"] });
  const shipping = trigger(handle, "Shipping");
  const warranty = trigger(handle, "Warranty");

  assert.equal(shipping.getAttribute("aria-disabled"), null);
  await handle.click(warranty);
  assert.equal(shipping.getAttribute("aria-expanded"), "true");
  assert.equal(warranty.getAttribute("aria-expanded"), "true");
  await handle.click(shipping);
  assert.equal(shipping.getAttribute("aria-expanded"), "false");
  assert.deepEqual(handle.wrapper.emitted("update:modelValue"), [
    [["shipping", "warranty"]],
    [["warranty"]],
  ]);

  handle.unmount();
});

test("controlled accordion waits for the parent to accept each request", async () => {
  const handle = mountAccordion({ type: "multiple", modelValue: [] });
  const returns = trigger(handle, "Returns");

  await handle.click(returns);
  assert.equal(returns.getAttribute("aria-expanded"), "false");
  assert.deepEqual(handle.wrapper.emitted("update:modelValue"), [[["returns"]]]);

  await handle.wrapper.setProps({ modelValue: ["returns"] });
  assert.equal(returns.getAttribute("aria-expanded"), "true");
  assert.equal(panel(handle, "returns").hidden, false);

  handle.unmount();
});

test("native Enter and Space activation toggle the focused trigger once", async () => {
  const handle = mountAccordion({ type: "single", collapsible: true });
  const returns = trigger(handle, "Returns");

  const enter = await handle.press(returns, "Enter");
  assert.equal(enter.activated, true);
  assert.equal(returns.getAttribute("aria-expanded"), "true");
  const space = await handle.press(returns, " ");
  assert.equal(space.activated, true);
  assert.equal(returns.getAttribute("aria-expanded"), "false");

  handle.unmount();
});

test("arrow keys, Home, and End move focus between enabled triggers with wrapping", async () => {
  const handle = mountAccordion({ type: "single" }, [
    { value: "one", label: "One" },
    { value: "two", label: "Two", disabled: true },
    { value: "three", label: "Three" },
  ]);
  await nextTick();
  const one = trigger(handle, "One");
  const three = trigger(handle, "Three");

  one.focus();
  const down = await handle.press(one, "ArrowDown");
  assert.equal(down.keydownPrevented, true);
  assert.ok(handle.activeElement() === three, "ArrowDown skips the disabled trigger");
  await handle.press(three, "ArrowDown");
  assert.ok(handle.activeElement() === one, "ArrowDown wraps to the first trigger");
  await handle.press(one, "ArrowUp");
  assert.ok(handle.activeElement() === three, "ArrowUp wraps to the last trigger");
  await handle.press(three, "Home");
  assert.ok(handle.activeElement() === one);
  await handle.press(one, "End");
  assert.ok(handle.activeElement() === three);
  const sideways = await handle.press(three, "ArrowRight");
  assert.equal(sideways.keydownPrevented, false, "vertical accordions ignore horizontal arrows");

  handle.unmount();
});

test("horizontal navigation follows reading direction and loop can stop at the edges", async () => {
  const handle = mountAccordion({ type: "single", orientation: "horizontal", dir: "rtl" });
  await nextTick();
  const shipping = trigger(handle, "Shipping");
  const returns = trigger(handle, "Returns");

  shipping.focus();
  await handle.press(shipping, "ArrowLeft");
  assert.ok(handle.activeElement() === returns, "ArrowLeft moves forward in rtl");
  await handle.press(returns, "ArrowRight");
  assert.ok(handle.activeElement() === shipping);
  handle.unmount();

  const bounded = mountAccordion({ type: "single", loop: false });
  await nextTick();
  const first = trigger(bounded, "Shipping");
  first.focus();
  const up = await bounded.press(first, "ArrowUp");
  assert.equal(up.keydownPrevented, false);
  assert.ok(bounded.activeElement() === first);
  bounded.unmount();
});

test("disabled roots and items block user activation with native disabled buttons", async () => {
  const rootDisabled = mountAccordion({ type: "multiple", disabled: true });
  const shipping = trigger(rootDisabled, "Shipping");
  assert.equal(shipping.disabled, true);
  assert.equal(rootDisabled.root().getAttribute("data-disabled"), "true");
  await rootDisabled.click(shipping);
  assert.equal(shipping.getAttribute("aria-expanded"), "false");
  rootDisabled.unmount();

  const itemDisabled = mountAccordion({ type: "multiple" }, [
    { value: "a", label: "Alpha", disabled: true },
    { value: "b", label: "Bravo" },
  ]);
  const alpha = trigger(itemDisabled, "Alpha");
  assert.equal(alpha.disabled, true);
  assert.equal(alpha.getAttribute("data-disabled"), "true");
  await itemDisabled.click(alpha);
  assert.equal(itemDisabled.wrapper.emitted("update:modelValue"), undefined);
  itemDisabled.unmount();
});

test("trigger click is preventable before the item toggles", async () => {
  const handle = mountInteraction(AccordionRoot, {
    props: { type: "multiple" },
    slots: {
      default: () =>
        h(AccordionItem, { value: "a" }, () => [
          h(AccordionHeader, null, () =>
            h(
              AccordionTrigger,
              { onClick: (event: MouseEvent) => event.preventDefault() },
              () => "Alpha",
            ),
          ),
          h(AccordionContent, null, () => "Alpha details"),
        ]),
    },
  });

  await handle.click(handle.getByRole("button", { name: "Alpha" }));
  assert.equal(handle.wrapper.emitted("update:modelValue"), undefined);

  handle.unmount();
});

test("hidden until-found keeps closed panels searchable and beforematch expands them", async () => {
  const handle = mountAccordion({ type: "single", hiddenUntilFound: true });
  await nextTick();
  const returns = panel(handle, "returns");

  assert.equal(returns.getAttribute("hidden"), "until-found");
  assert.equal(returns.getAttribute("data-hidden-until-found"), "true");
  returns.dispatchEvent(new Event("beforematch", { bubbles: true }));
  await nextTick();
  await nextTick();
  assert.equal(returns.hasAttribute("hidden"), false);
  assert.equal(trigger(handle, "Returns").getAttribute("aria-expanded"), "true");
  const change = handle.wrapper.emitted("value-change")?.[0];
  assert.equal(change?.[0], "returns");
  assert.ok(change?.[2] instanceof Event);

  await handle.click(trigger(handle, "Shipping"));
  await nextTick();
  assert.equal(returns.getAttribute("hidden"), "until-found", "re-closed panels stay findable");
  handle.unmount();

  const plain = mountAccordion({ type: "single" }, defaultItems, { hiddenUntilFound: false });
  await nextTick();
  assert.equal(panel(plain, "returns").getAttribute("hidden"), "");
  plain.unmount();
});

test("content publishes measured size variables and supports role opt-out", async () => {
  const handle = mountAccordion({ type: "single", defaultValue: "shipping" }, defaultItems, {
    role: null,
  });
  await nextTick();
  const shipping = panel(handle, "shipping");

  assert.equal(shipping.getAttribute("role"), null);
  assert.equal(shipping.getAttribute("aria-labelledby"), null);
  assert.match(shipping.style.getPropertyValue("--vize-accordion-content-height"), /^\d+px$/);
  assert.equal(handle.queryByRole("region"), null);

  handle.unmount();
});

test("heading level follows the root default and per-header overrides", () => {
  const handle = mountInteraction(AccordionRoot, {
    props: { type: "single", headingLevel: 2 },
    slots: {
      default: () => [
        h(AccordionItem, { value: 1 }, () => [
          h(AccordionHeader, null, () => h(AccordionTrigger, null, () => "Numeric")),
          h(AccordionContent, null, () => "Numeric details"),
        ]),
        h(AccordionItem, { value: "custom id" }, () => [
          h(AccordionHeader, { level: 4 }, () => h(AccordionTrigger, null, () => "Spaced")),
          h(AccordionContent, null, () => "Spaced details"),
        ]),
      ],
    },
  });
  const numeric = handle.getByRole("button", { name: "Numeric" });
  const spaced = handle.getByRole("button", { name: "Spaced" });

  assert.equal(numeric.parentElement?.tagName, "H2");
  assert.equal(numeric.parentElement?.getAttribute("data-level"), "2");
  assert.equal(spaced.parentElement?.tagName, "H4");
  assert.match(numeric.id, /-item-n1-trigger$/);
  assert.match(spaced.id, /-item-custom-id-[a-z0-9]+-trigger$/);

  handle.unmount();
});

test("root and item expose typed programmatic controls", async () => {
  let rootExpose: AccordionRootExpose<string, "multiple"> | null = null;
  let itemExpose: AccordionItemExpose | null = null;
  const Probe = defineComponent({
    name: "AccordionExposeProbe",
    setup: () => () =>
      h(
        AccordionRoot,
        {
          type: "multiple",
          ref: (value) => {
            rootExpose = value as AccordionRootExpose<string, "multiple"> | null;
          },
        },
        () => [
          h(
            AccordionItem,
            {
              value: "a",
              ref: (value) => {
                itemExpose = value as AccordionItemExpose | null;
              },
            },
            () => [
              h(AccordionHeader, null, () => h(AccordionTrigger, null, () => "Alpha")),
              h(AccordionContent, null, () => "Alpha details"),
            ],
          ),
          h(AccordionItem, { value: "b", disabled: true }, () => [
            h(AccordionHeader, null, () => h(AccordionTrigger, null, () => "Bravo")),
            h(AccordionContent, null, () => "Bravo details"),
          ]),
        ],
      ),
  });
  const handle = mountInteraction(Probe);
  await nextTick();

  if (rootExpose === null || itemExpose === null) assert.fail("Accordion refs must expose state");
  const root: AccordionRootExpose<string, "multiple"> = rootExpose;
  const item: AccordionItemExpose = itemExpose;

  assert.equal(root.type, "multiple");
  assert.deepEqual(root.value, []);
  assert.equal(item.open, false);
  assert.match(item.triggerId, /-item-a-trigger$/);
  assert.equal(root.expand("a"), true);
  await nextTick();
  assert.equal(item.open, true);
  assert.equal(root.isOpen("a"), true);
  assert.equal(item.toggle(), true);
  await nextTick();
  assert.equal(root.isOpen("a"), false);
  assert.equal(root.expandAll(), true, "expandAll opens enabled items");
  await nextTick();
  assert.deepEqual(root.openValues, ["a"]);
  assert.equal(root.setValue(["a", "b"]), true);
  await nextTick();
  assert.deepEqual(root.value, ["a", "b"]);
  assert.equal(root.collapseAll(), true);
  await nextTick();
  assert.deepEqual(root.openValues, []);
  assert.equal(root.collapse("a"), false);
  assert.equal(root.focus(), true);
  assert.equal(handle.activeElement()?.textContent, "Alpha");

  handle.unmount();
});

test("single roots refuse expandAll and keep collapseAll behind collapsible", async () => {
  let rootExpose: AccordionRootExpose<string, "single"> | null = null;
  const Probe = defineComponent({
    name: "AccordionSingleExposeProbe",
    setup: () => () =>
      h(
        AccordionRoot,
        {
          type: "single",
          defaultValue: "shipping",
          ref: (value) => {
            rootExpose = value as AccordionRootExpose<string, "single"> | null;
          },
        },
        () => renderItems(defaultItems),
      ),
  });
  const handle = mountInteraction(Probe);

  if (rootExpose === null) assert.fail("Accordion root must expose state");
  const root: AccordionRootExpose<string, "single"> = rootExpose;
  assert.equal(root.value, "shipping");
  assert.equal(root.expandAll(), false);
  assert.equal(root.collapseAll(), false);
  assert.equal(root.setValue(null), true, "setValue bypasses the collapsible guard");
  await nextTick();
  assert.equal(root.value, null);

  handle.unmount();
});

test("items publish the Collapsible contract to Collapsible parts", async () => {
  const handle = mountInteraction(AccordionRoot, {
    props: { type: "multiple" },
    slots: {
      default: () =>
        h(AccordionItem, { value: "legacy" }, () => [
          h(CollapsibleTrigger, null, () => "Legacy"),
          h(CollapsibleContent, null, () => "Legacy details"),
        ]),
    },
  });
  const legacy = handle.getByRole("button", { name: "Legacy" });

  await handle.click(legacy);
  assert.equal(legacy.getAttribute("aria-expanded"), "true");
  assert.deepEqual(handle.wrapper.emitted("update:modelValue"), [[["legacy"]]]);
  assert.match(legacy.id, /-item-legacy-trigger$/);

  handle.unmount();
});

test("accordion parts require their providers", () => {
  assert.throws(() => mountInteraction(AccordionItem, { props: { value: "a" } }), /Accordion/);
  assert.throws(() => mountInteraction(AccordionTrigger), /VIZE_UI_CONTEXT_MISSING/);
  assert.throws(() => mountInteraction(AccordionContent), /VIZE_UI_CONTEXT_MISSING/);
  assert.throws(() => mountInteraction(AccordionHeader), /VIZE_UI_CONTEXT_MISSING/);
});
