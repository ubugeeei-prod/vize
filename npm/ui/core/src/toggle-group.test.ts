import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h, nextTick } from "vue";

import type {
  ToggleGroupExpose,
  ToggleGroupItemExpose,
  ToggleGroupSlotState,
} from "./toggle-group.ts";
import ToggleGroup from "./toggle-group.vue";
import ToggleGroupItem from "./toggle-group-item.vue";
import { mountInteraction } from "./testing/mount.ts";

function mountToggleGroup(
  props: Record<string, unknown> = {},
  itemProps: Record<string, unknown> = {},
) {
  return mountInteraction(ToggleGroup, {
    props: { ariaLabel: "Formatting", ...props },
    slots: {
      default: (state: ToggleGroupSlotState) => [
        h("output", { "data-group-state": state.state }, state.pressedValues.join(",")),
        h(ToggleGroupItem, { value: "bold", ...itemProps }, () => "Bold"),
        h(ToggleGroupItem, { value: "italic" }, () => "Italic"),
      ],
    },
  });
}

test("renders grouped toggle button semantics", () => {
  const handle = mountToggleGroup({ defaultValue: "bold" });
  const group = handle.getByRole("group", { name: "Formatting" });
  const bold = handle.getByRole("button", { name: "Bold" }) as HTMLButtonElement;
  const italic = handle.getByRole("button", { name: "Italic" }) as HTMLButtonElement;

  assert.equal(group.getAttribute("data-vize-ui"), "toggle-group");
  assert.equal(group.getAttribute("data-state"), "selected");
  assert.equal(group.getAttribute("data-type"), "single");
  assert.equal(group.getAttribute("data-orientation"), "horizontal");
  assert.equal(group.getAttribute("data-value"), "bold");
  assert.equal(group.querySelector("[data-group-state]")?.textContent, "bold");
  assert.equal(bold.type, "button");
  assert.equal(bold.getAttribute("aria-pressed"), "true");
  assert.equal(bold.getAttribute("data-state"), "pressed");
  assert.equal(bold.getAttribute("data-pressed"), "true");
  assert.equal(bold.getAttribute("tabindex"), "0");
  assert.equal(italic.getAttribute("aria-pressed"), "false");
  assert.equal(italic.getAttribute("data-state"), "unpressed");
  assert.equal(italic.getAttribute("tabindex"), "-1");
  handle.unmount();
});

test("uncontrolled single mode toggles one value and emits changes", async () => {
  const handle = mountInteraction(ToggleGroup, {
    props: { ariaLabel: "Formatting" },
    record: ["update:modelValue", "change"],
    slots: {
      default: () => [
        h(ToggleGroupItem, { value: "bold" }, () => "Bold"),
        h(ToggleGroupItem, { value: "italic" }, () => "Italic"),
      ],
    },
  });
  const bold = handle.getByRole("button", { name: "Bold" });
  const italic = handle.getByRole("button", { name: "Italic" });

  await handle.click(bold);
  assert.equal(bold.getAttribute("aria-pressed"), "true");
  assert.equal(italic.getAttribute("aria-pressed"), "false");
  await handle.click(italic);
  assert.equal(bold.getAttribute("aria-pressed"), "false");
  assert.equal(italic.getAttribute("aria-pressed"), "true");
  await handle.click(italic);
  assert.equal(italic.getAttribute("aria-pressed"), "false");

  assert.deepEqual(
    handle.recorded().map((emit) => [emit.event, emit.payload[0], emit.payload[1]]),
    [
      ["update:modelValue", "bold", undefined],
      ["change", "bold", null],
      ["update:modelValue", "italic", undefined],
      ["change", "italic", "bold"],
      ["update:modelValue", null, undefined],
      ["change", null, "italic"],
    ],
  );
  assert.ok(handle.recorded()[1]?.payload[2] instanceof MouseEvent);
  handle.unmount();
});

test("controlled single value wins until the parent accepts the request", async () => {
  const handle = mountToggleGroup({ modelValue: "bold" });
  const bold = handle.getByRole("button", { name: "Bold" });
  const italic = handle.getByRole("button", { name: "Italic" });

  await handle.click(italic);
  await nextTick();
  assert.deepEqual(handle.wrapper.emitted("update:modelValue"), [["italic"]]);
  assert.deepEqual(handle.wrapper.emitted("change")?.[0]?.slice(0, 2), ["italic", "bold"]);
  assert.equal(bold.getAttribute("aria-pressed"), "true");
  assert.equal(italic.getAttribute("aria-pressed"), "false");

  await handle.wrapper.setProps({ modelValue: "italic" });
  assert.equal(bold.getAttribute("aria-pressed"), "false");
  assert.equal(italic.getAttribute("aria-pressed"), "true");
  handle.unmount();
});

test("multiple mode adds and removes item values", async () => {
  const itemPresses: unknown[][] = [];
  const handle = mountInteraction(ToggleGroup, {
    props: { ariaLabel: "Formatting", defaultValue: ["bold"], type: "multiple" },
    record: ["update:modelValue", "change"],
    slots: {
      default: () => [
        h(
          ToggleGroupItem,
          { value: "bold", onPress: (...args: unknown[]) => itemPresses.push(args) },
          () => "Bold",
        ),
        h(ToggleGroupItem, { value: "italic" }, () => "Italic"),
      ],
    },
  });
  const bold = handle.getByRole("button", { name: "Bold" });
  const italic = handle.getByRole("button", { name: "Italic" });

  await handle.click(italic);
  assert.deepEqual(handle.wrapper.emitted("update:modelValue")?.[0], [["bold", "italic"]]);
  assert.equal(bold.getAttribute("aria-pressed"), "true");
  assert.equal(italic.getAttribute("aria-pressed"), "true");

  await handle.click(bold);
  assert.deepEqual(handle.wrapper.emitted("update:modelValue")?.[1], [["italic"]]);
  assert.equal(bold.getAttribute("aria-pressed"), "false");
  assert.equal(italic.getAttribute("aria-pressed"), "true");
  assert.deepEqual(
    itemPresses.map((args) => args.slice(0, 2)),
    [["bold", false]],
  );
  handle.unmount();
});

test("defaultValue seeds state and native form reset restores it", async () => {
  const FormProbe = defineComponent({
    setup: () => () =>
      h("form", [
        h(ToggleGroup, { ariaLabel: "Formatting", defaultValue: "bold" }, () => [
          h(ToggleGroupItem, { value: "bold" }, () => "Bold"),
          h(ToggleGroupItem, { value: "italic" }, () => "Italic"),
        ]),
      ]),
  });
  const handle = mountInteraction(FormProbe);
  const form = handle.root() as HTMLFormElement;
  const bold = handle.getByRole("button", { name: "Bold" });
  const italic = handle.getByRole("button", { name: "Italic" });

  await handle.click(italic);
  assert.equal(bold.getAttribute("aria-pressed"), "false");
  assert.equal(italic.getAttribute("aria-pressed"), "true");
  form.reset();
  await nextTick();
  assert.equal(bold.getAttribute("aria-pressed"), "true");
  assert.equal(italic.getAttribute("aria-pressed"), "false");
  handle.unmount();
});

test("roving focus follows orientation and skips disabled items", async () => {
  const handle = mountInteraction(ToggleGroup, {
    props: { ariaLabel: "Formatting", orientation: "vertical" },
    slots: {
      default: () => [
        h(ToggleGroupItem, { value: "bold" }, () => "Bold"),
        h(ToggleGroupItem, { disabled: true, value: "italic" }, () => "Italic"),
        h(ToggleGroupItem, { value: "underline" }, () => "Underline"),
      ],
    },
  });
  const bold = handle.getByRole("button", { name: "Bold" });
  const italic = handle.getByRole("button", { name: "Italic" });
  const underline = handle.getByRole("button", { name: "Underline" });

  assert.ok((await handle.tab()) === bold);
  const down = await handle.press(bold, "ArrowDown");
  assert.equal(down.keydownPrevented, true);
  assert.ok(handle.activeElement() === underline);
  assert.equal(italic.getAttribute("tabindex"), null);
  assert.equal(underline.getAttribute("tabindex"), "0");

  await handle.press(underline, "Home");
  assert.ok(handle.activeElement() === bold);
  await handle.press(bold, "ArrowUp");
  assert.ok(handle.activeElement() === underline);
  handle.unmount();
});

test("roving focus uses the focused item when selected state changes externally", async () => {
  let groupExpose: ToggleGroupExpose | null = null;
  const Probe = defineComponent({
    setup: () => () =>
      h(
        ToggleGroup,
        {
          ariaLabel: "Formatting",
          ref: (value) => (groupExpose = value as ToggleGroupExpose | null),
        },
        () => [
          h(ToggleGroupItem, { value: "bold" }, () => "Bold"),
          h(ToggleGroupItem, { value: "italic" }, () => "Italic"),
          h(ToggleGroupItem, { value: "underline" }, () => "Underline"),
        ],
      ),
  });
  const handle = mountInteraction(Probe);
  const bold = handle.getByRole("button", { name: "Bold" });
  const italic = handle.getByRole("button", { name: "Italic" });

  if (groupExpose === null) assert.fail("ToggleGroup ref must expose state");
  assert.ok((await handle.tab()) === bold);
  groupExpose.setValue("underline");
  await nextTick();

  await handle.press(bold, "ArrowRight");
  assert.ok(handle.activeElement() === italic);
  handle.unmount();
});

test("disabled groups and items suppress activation", async () => {
  const groupDisabled = mountToggleGroup({ disabled: true, defaultValue: "bold" });
  const group = groupDisabled.getByRole("group", { name: "Formatting" });
  const bold = groupDisabled.getByRole("button", { name: "Bold" }) as HTMLButtonElement;

  assert.equal(group.getAttribute("aria-disabled"), "true");
  assert.equal(group.getAttribute("data-state"), "disabled");
  assert.equal(bold.disabled, true);
  assert.equal(bold.getAttribute("data-state"), "disabled");
  await groupDisabled.click(bold);
  assert.equal(bold.getAttribute("aria-pressed"), "true");
  assert.equal(groupDisabled.wrapper.emitted("change"), undefined);
  assert.ok((await groupDisabled.tab()) === null);
  groupDisabled.unmount();

  const customDisabled = mountInteraction(ToggleGroup, {
    props: { ariaLabel: "Formatting" },
    slots: {
      default: () =>
        h(ToggleGroupItem, { as: "span", disabled: true, value: "bold" }, () => "Bold"),
    },
  });
  const customItem = customDisabled.getByRole("button", { name: "Bold" });
  assert.equal(customItem.tagName, "SPAN");
  assert.equal(customItem.getAttribute("aria-disabled"), "true");
  assert.equal(customItem.getAttribute("tabindex"), "-1");
  await customDisabled.press(customItem, " ");
  assert.equal(customItem.getAttribute("aria-pressed"), "false");
  customDisabled.unmount();
});

test("exposes focus, setValue, toggleValue, reset, and item state", async () => {
  let groupExpose: ToggleGroupExpose | null = null;
  let itemExpose: ToggleGroupItemExpose | null = null;
  const Probe = defineComponent({
    setup: () => () =>
      h(
        ToggleGroup,
        {
          ariaLabel: "Formatting",
          ref: (value) => (groupExpose = value as ToggleGroupExpose | null),
        },
        () => [
          h(ToggleGroupItem, { value: "bold" }, () => "Bold"),
          h(
            ToggleGroupItem,
            {
              ref: (value) => (itemExpose = value as ToggleGroupItemExpose | null),
              value: "italic",
            },
            () => "Italic",
          ),
        ],
      ),
  });
  const handle = mountInteraction(Probe);
  const bold = handle.getByRole("button", { name: "Bold" });
  const italic = handle.getByRole("button", { name: "Italic" });

  if (groupExpose === null || itemExpose === null)
    assert.fail("ToggleGroup refs must expose state");
  groupExpose.setValue("italic");
  await nextTick();
  assert.deepEqual(groupExpose.pressedValues, ["italic"]);
  assert.equal(itemExpose.pressed, true);
  groupExpose.focus();
  assert.ok(handle.activeElement() === italic);

  itemExpose.focus();
  assert.ok(handle.activeElement() === italic);
  groupExpose.toggleValue("italic", false);
  await nextTick();
  assert.equal(italic.getAttribute("aria-pressed"), "false");
  assert.equal(groupExpose.reset(), false);
  groupExpose.setValue("bold");
  await nextTick();
  assert.equal(bold.getAttribute("aria-pressed"), "true");
  handle.unmount();
});

test("items require a matching group provider", () => {
  assert.throws(
    () => mountInteraction(ToggleGroupItem, { props: { value: "bold" } }),
    /VIZE_UI_CONTEXT_MISSING/,
  );
});
