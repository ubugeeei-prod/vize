import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h, nextTick, ref } from "vue";

import type { ListboxExpose, ListboxItemExpose, ListboxValue } from "./listbox.ts";
import Listbox from "./listbox.vue";
import ListboxItem from "./listbox-item.vue";
import { mountInteraction } from "./testing/mount.ts";

function mountListbox(
  props: Record<string, unknown> = {},
  firstItemProps: Record<string, unknown> = {},
) {
  return mountInteraction(Listbox, {
    props,
    record: ["update:modelValue", "change"],
    slots: {
      default: (state) => [
        h(
          "output",
          {
            "data-active": state.activeValue ?? "",
            "data-root-state": state.state,
            "data-values": state.selectedValues.join("|"),
          },
          Array.isArray(state.value) ? state.value.join(",") : (state.value ?? ""),
        ),
        h(
          ListboxItem,
          { ariaLabel: "Alpha", textValue: "Alpha", value: "alpha", ...firstItemProps },
          {
            default: (item) => h("span", { "data-item-state": item.state }, "Alpha"),
            indicator: (item) =>
              item.selected ? h("span", { "data-indicator": item.value }, "selected") : null,
          },
        ),
        h(ListboxItem, { textValue: "Bravo", value: "bravo" }, () => "Bravo"),
        h(ListboxItem, { textValue: "Charlie", value: "charlie" }, () => "Charlie"),
      ],
      empty: (state) => h("p", { "data-empty-state": state.state }, "Empty"),
    },
  });
}

function optionIds(root: HTMLElement): Record<string, string> {
  const ids: Record<string, string> = {};
  for (const option of root.querySelectorAll<HTMLElement>("[role='option']")) {
    ids[option.getAttribute("data-value") ?? ""] = option.id;
  }
  return ids;
}

function recordedValues(handle: ReturnType<typeof mountListbox>) {
  return handle.recorded().map(({ event, payload }) => [event, payload[0], payload[1]]);
}

test("renders active-descendant listbox semantics, slots, and data attributes", async () => {
  const handle = mountListbox({
    ariaDescribedby: "letters-help",
    ariaErrormessage: "letters-error",
    ariaInvalid: true,
    ariaLabel: "Letters",
    defaultValue: "bravo",
    direction: "rtl",
    id: "letters",
    orientation: "horizontal",
    required: true,
  });
  await nextTick();
  const root = handle.getByRole("listbox", { name: "Letters" });
  const alpha = handle.getByRole("option", { name: "Alpha" });
  const bravo = handle.getByRole("option", { name: "Bravo" });

  assert.equal(root.id, "letters");
  assert.equal(root.tabIndex, 0);
  assert.equal(root.getAttribute("aria-describedby"), "letters-help");
  assert.equal(root.getAttribute("aria-errormessage"), "letters-error");
  assert.equal(root.getAttribute("aria-invalid"), "true");
  assert.equal(root.getAttribute("aria-orientation"), "horizontal");
  assert.equal(root.getAttribute("aria-required"), "true");
  assert.equal(root.getAttribute("aria-multiselectable"), null);
  assert.equal(root.getAttribute("data-vize-ui"), "listbox");
  assert.equal(root.getAttribute("part"), "root");
  assert.equal(root.getAttribute("data-state"), "selected");
  assert.equal(root.getAttribute("data-selection-mode"), "single");
  assert.equal(root.getAttribute("data-selection-count"), "1");
  assert.equal(root.getAttribute("data-value"), "bravo");
  assert.equal(root.querySelector("[data-root-state]")?.getAttribute("data-active"), "bravo");
  assert.equal(alpha.getAttribute("data-state"), "unselected");
  assert.equal(bravo.getAttribute("aria-selected"), "true");
  assert.equal(bravo.getAttribute("data-state"), "selected");
  assert.equal(root.getAttribute("aria-activedescendant"), bravo.id);
  assert.equal(root.querySelector("[data-indicator='alpha']"), null);
  handle.unmount();
});

test("single selection separates active navigation, typeahead, and selection", async () => {
  const handle = mountListbox({ ariaLabel: "Letters" });
  const root = handle.getByRole("listbox", { name: "Letters" });
  const ids = optionIds(root);

  assert.ok((await handle.tab()) === root);
  assert.equal(root.getAttribute("aria-activedescendant"), ids.alpha);

  const arrow = await handle.press(root, "ArrowDown");
  assert.equal(arrow.keydownPrevented, true);
  assert.equal(root.getAttribute("aria-activedescendant"), ids.bravo);
  assert.equal(root.getAttribute("data-state"), "empty");
  assert.equal(
    handle.getByRole("option", { name: "Bravo" }).getAttribute("aria-selected"),
    "false",
  );

  const enter = await handle.press(root, "Enter");
  assert.equal(enter.keydownPrevented, true);
  assert.equal(root.getAttribute("data-value"), "bravo");
  assert.equal(handle.getByRole("option", { name: "Bravo" }).getAttribute("aria-selected"), "true");

  const typed = new KeyboardEvent("keydown", {
    bubbles: true,
    cancelable: true,
    key: "c",
  });
  root.dispatchEvent(typed);
  await nextTick();
  assert.equal(typed.defaultPrevented, true);
  assert.equal(root.getAttribute("aria-activedescendant"), ids.charlie);
  assert.equal(root.getAttribute("data-value"), "bravo");

  await handle.press(root, " ");
  assert.equal(root.getAttribute("data-value"), "charlie");
  assert.deepEqual(recordedValues(handle), [
    ["update:modelValue", "bravo", undefined],
    ["change", "bravo", null],
    ["update:modelValue", "charlie", undefined],
    ["change", "charlie", "bravo"],
  ]);
  handle.unmount();
});

test("multiple selection toggles independent options and emits readonly arrays", async () => {
  const handle = mountListbox({
    ariaLabel: "Letters",
    defaultValue: ["alpha"],
    selectionMode: "multiple",
  });
  const root = handle.getByRole("listbox", { name: "Letters" });
  const alpha = handle.getByRole("option", { name: "Alpha" });
  const bravo = handle.getByRole("option", { name: "Bravo" });
  const charlie = handle.getByRole("option", { name: "Charlie" });

  assert.equal(root.getAttribute("aria-multiselectable"), "true");
  assert.equal(root.getAttribute("data-selection-count"), "1");
  assert.equal(alpha.getAttribute("aria-selected"), "true");

  await handle.click(bravo);
  assert.equal(root.getAttribute("data-selection-count"), "2");
  assert.equal(bravo.getAttribute("aria-selected"), "true");

  await handle.click(alpha);
  assert.equal(root.getAttribute("data-selection-count"), "1");
  assert.equal(alpha.getAttribute("aria-selected"), "false");

  root.focus();
  const typed = new KeyboardEvent("keydown", {
    bubbles: true,
    cancelable: true,
    key: "c",
  });
  root.dispatchEvent(typed);
  await nextTick();
  await handle.press(root, " ");
  assert.equal(charlie.getAttribute("aria-selected"), "true");
  assert.equal(
    root.querySelector("[data-root-state]")?.getAttribute("data-values"),
    "bravo|charlie",
  );

  const updates = handle.wrapper.emitted("update:modelValue") as ListboxValue[][] | undefined;
  assert.deepEqual(
    updates?.map(([value]) => value),
    [["alpha", "bravo"], ["bravo"], ["bravo", "charlie"]],
  );
  assert.equal(Object.isFrozen(updates?.[0]?.[0]), true);
  handle.unmount();
});

test("controlled selection waits for the parent to accept the requested value", async () => {
  const handle = mountListbox({ ariaLabel: "Letters", modelValue: "alpha" });
  const alpha = handle.getByRole("option", { name: "Alpha" });
  const bravo = handle.getByRole("option", { name: "Bravo" });

  await handle.click(bravo);
  await nextTick();

  assert.deepEqual(handle.wrapper.emitted("update:modelValue"), [["bravo"]]);
  assert.deepEqual(handle.wrapper.emitted("change")?.[0]?.slice(0, 2), ["bravo", "alpha"]);
  assert.equal(alpha.getAttribute("aria-selected"), "true");
  assert.equal(bravo.getAttribute("aria-selected"), "false");

  await handle.wrapper.setProps({ modelValue: "bravo" });
  assert.equal(alpha.getAttribute("aria-selected"), "false");
  assert.equal(bravo.getAttribute("aria-selected"), "true");
  handle.unmount();
});

test("items reregister when their option value changes", async () => {
  const dynamicValue = ref("alpha");
  const Probe = defineComponent({
    name: "ListboxDynamicValueProbe",
    setup: () => () =>
      h(Listbox, { ariaLabel: "Dynamic values", defaultValue: "alpha" }, () => [
        h(ListboxItem, { textValue: "Dynamic", value: dynamicValue.value }, () =>
          dynamicValue.value === "alpha" ? "Alpha" : "Delta",
        ),
        h(ListboxItem, { value: "bravo" }, () => "Bravo"),
      ]),
  });
  const handle = mountInteraction(Probe, { record: ["update:modelValue", "change"] });
  const root = handle.getByRole("listbox", { name: "Dynamic values" });

  assert.equal(handle.getByRole("option", { name: "Alpha" }).getAttribute("aria-selected"), "true");
  dynamicValue.value = "delta";
  await nextTick();

  const delta = handle.getByRole("option", { name: "Delta" });
  assert.equal(delta.getAttribute("data-value"), "delta");
  assert.equal(delta.getAttribute("aria-selected"), "false");
  await handle.click(delta);

  assert.equal(root.getAttribute("data-value"), "delta");
  assert.deepEqual(recordedValues(handle), [
    ["update:modelValue", "delta", undefined],
    ["change", "delta", "alpha"],
  ]);
  handle.unmount();
});

test("disabled roots and items suppress focus, navigation, typeahead, and selection", async () => {
  const disabledRoot = mountListbox({
    ariaLabel: "Letters",
    defaultValue: "alpha",
    disabled: true,
  });
  const root = disabledRoot.getByRole("listbox", { name: "Letters" });
  const bravo = disabledRoot.getByRole("option", { name: "Bravo" });

  assert.equal(root.hasAttribute("tabindex"), false);
  assert.equal(root.getAttribute("aria-disabled"), "true");
  assert.equal(root.getAttribute("data-state"), "disabled");
  assert.equal(await disabledRoot.tab(), null);
  await disabledRoot.click(bravo);
  assert.equal(root.getAttribute("data-value"), "alpha");
  assert.equal(disabledRoot.wrapper.emitted("change"), undefined);
  disabledRoot.unmount();

  const disabledItem = mountListbox({ ariaLabel: "Letters" }, { disabled: true });
  const itemRoot = disabledItem.getByRole("listbox", { name: "Letters" });
  const alpha = disabledItem.getByRole("option", { name: "Alpha" });
  const itemIds = optionIds(itemRoot);

  itemRoot.focus();
  await nextTick();
  assert.equal(alpha.getAttribute("aria-disabled"), "true");
  assert.equal(itemRoot.getAttribute("aria-activedescendant"), itemIds.bravo);
  await disabledItem.click(alpha);
  assert.equal(itemRoot.getAttribute("data-state"), "empty");
  disabledItem.unmount();
});

test("exposed root and item methods share the same selection state", async () => {
  let listbox: ListboxExpose | null = null;
  let charlie: ListboxItemExpose | null = null;
  const Probe = defineComponent({
    setup: () => () =>
      h(
        Listbox,
        {
          ariaLabel: "Letters",
          defaultValue: "alpha",
          ref: (value) => {
            listbox = value as ListboxExpose | null;
          },
        },
        () => [
          h(ListboxItem, { value: "alpha" }, () => "Alpha"),
          h(ListboxItem, { value: "bravo" }, () => "Bravo"),
          h(
            ListboxItem,
            {
              ref: (value) => {
                charlie = value as ListboxItemExpose | null;
              },
              value: "charlie",
            },
            () => "Charlie",
          ),
        ],
      ),
  });
  const handle = mountInteraction(Probe);

  if (listbox === null || charlie === null) assert.fail("Listbox refs must expose state");

  listbox.focus();
  assert.ok(handle.activeElement() === handle.getByRole("listbox", { name: "Letters" }));
  assert.equal(listbox.navigate("next"), "bravo");
  assert.equal(listbox.activeValue, "bravo");

  charlie.focus();
  await nextTick();
  assert.equal(charlie.active, true);
  assert.equal(charlie.select(), true);
  await nextTick();
  assert.equal(listbox.value, "charlie");

  assert.equal(listbox.clear(), true);
  await nextTick();
  assert.equal(listbox.value, null);
  assert.equal(listbox.reset(), true);
  await nextTick();
  assert.equal(listbox.value, "alpha");
  handle.unmount();
});

test("items require a matching Listbox provider", () => {
  assert.throws(
    () =>
      mountInteraction(ListboxItem, {
        props: { value: "orphan" },
        slots: { default: () => "Orphan" },
      }),
    /VIZE_UI_CONTEXT_MISSING/,
  );
});
