import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h, nextTick } from "vue";

import DirectionProvider from "./direction-provider.vue";
import { toDirection, useResolvedDirection } from "./direction-runtime.ts";
import AccordionContent from "../../disclosure/accordion/accordion-content.vue";
import AccordionHeader from "../../disclosure/accordion/accordion-header.vue";
import AccordionItem from "../../disclosure/accordion/accordion-item.vue";
import AccordionRoot from "../../disclosure/accordion/accordion-root.vue";
import AccordionTrigger from "../../disclosure/accordion/accordion-trigger.vue";
import LocaleProvider from "../locale/locale-provider.vue";
import TabsList from "../../navigation/tabs/tabs-list.vue";
import TabsRoot from "../../navigation/tabs/tabs-root.vue";
import TabsTrigger from "../../navigation/tabs/tabs-trigger.vue";
import { mountInteraction } from "../../../testing/mount.ts";

const DirectionProbe = defineComponent({
  name: "DirectionProbe",
  props: { local: { type: String, default: undefined } },
  setup(props) {
    const resolved = useResolvedDirection(() => toDirection(props.local));
    return () => h("output", { "data-dir": resolved.value }, resolved.value);
  },
});

function horizontalAccordion(props: Record<string, unknown> = {}) {
  return h(AccordionRoot, { type: "single", orientation: "horizontal", ...props }, () =>
    ["one", "two"].map((value) =>
      h(AccordionItem, { key: value, value }, () => [
        h(AccordionHeader, null, () => h(AccordionTrigger, null, () => value)),
        h(AccordionContent, null, () => `${value} details`),
      ]),
    ),
  );
}

test("provider renders a dir wrapper and publishes the direction to its slot", () => {
  const handle = mountInteraction(DirectionProvider, {
    props: { dir: "rtl" },
    slots: { default: ({ dir }: { readonly dir: string }) => h("span", dir) },
  });
  const root = handle.root();

  assert.equal(root.tagName, "DIV");
  assert.equal(root.getAttribute("dir"), "rtl");
  assert.equal(root.getAttribute("data-vize-ui"), "direction-provider");
  assert.equal(root.textContent, "rtl");

  handle.unmount();
});

test("useResolvedDirection prefers the local value, then the provider, then ltr", () => {
  const standalone = mountInteraction(DirectionProbe);
  assert.equal(standalone.root().getAttribute("data-dir"), "ltr");
  standalone.unmount();

  const Tree = defineComponent({
    name: "DirectionPrecedence",
    setup: () => () =>
      h(DirectionProvider, { dir: "rtl" }, () => [
        h(DirectionProbe),
        h(DirectionProbe, { local: "ltr" }),
        h(DirectionProvider, { as: null }, () => h(DirectionProbe, { "data-nested": "" })),
      ]),
  });
  const handle = mountInteraction(Tree);
  const outputs = [...handle.root().querySelectorAll("output")].map((node) => node.textContent);

  assert.deepEqual(outputs, ["rtl", "ltr", "rtl"]);
  assert.equal(
    handle.root().querySelectorAll('[data-vize-ui="direction-provider"]').length,
    0,
    "as=null renders the nested provider without a wrapper",
  );
  assert.equal(toDirection("sideways"), undefined);

  handle.unmount();
});

test("direction-aware families inherit the provider and flip horizontal arrow keys", async () => {
  const Tree = defineComponent({
    name: "DirectionInheritance",
    setup: () => () => h(DirectionProvider, { dir: "rtl" }, () => horizontalAccordion()),
  });
  const handle = mountInteraction(Tree);
  await nextTick();
  const root = handle.root().querySelector('[data-vize-ui="accordion-root"]');
  const one = handle.getByRole("button", { name: "one" });
  const two = handle.getByRole("button", { name: "two" });

  assert.equal(root?.getAttribute("dir"), "rtl");
  one.focus();
  await handle.press(one, "ArrowLeft");
  assert.ok(handle.activeElement() === two, "ArrowLeft moves forward in an inherited rtl context");

  handle.unmount();
});

test("an explicit dir prop overrides the provided direction", async () => {
  const Tree = defineComponent({
    name: "DirectionOverride",
    setup: () => () =>
      h(DirectionProvider, { dir: "rtl" }, () =>
        h(TabsRoot, { dir: "ltr", defaultValue: "a" }, () =>
          h(TabsList, null, () => [
            h(TabsTrigger, { value: "a" }, () => "A"),
            h(TabsTrigger, { value: "b" }, () => "B"),
          ]),
        ),
      ),
  });
  const handle = mountInteraction(Tree);
  await nextTick();

  assert.equal(
    handle.root().querySelector('[data-vize-ui="tabs-root"]')?.getAttribute("dir"),
    "ltr",
  );

  handle.unmount();
});

test("LocaleProvider publishes its resolved direction to direction-aware families", async () => {
  const Tree = defineComponent({
    name: "LocaleDirection",
    setup: () => () =>
      h(LocaleProvider, { locale: "ar", direction: "rtl" }, () => horizontalAccordion()),
  });
  const handle = mountInteraction(Tree);
  await nextTick();

  assert.equal(
    handle.root().querySelector('[data-vize-ui="accordion-root"]')?.getAttribute("dir"),
    "rtl",
  );

  handle.unmount();
});
