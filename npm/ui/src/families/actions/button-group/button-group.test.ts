import assert from "node:assert/strict";

import { mount } from "@vue/test-utils";
import { test } from "vite-plus/test";
import { defineComponent, h, nextTick } from "vue";

import type {
  ButtonGroupExpose,
  ButtonGroupItemExpose,
  ButtonGroupSlotState,
} from "./button-group.ts";
import ButtonGroup from "./button-group.vue";
import ButtonGroupItem from "./button-group-item.vue";
import { mountInteraction } from "../../../testing/mount.ts";

function mountButtonGroup(
  props: Record<string, unknown> = {},
  itemProps: Record<string, unknown> = {},
) {
  return mountInteraction(ButtonGroup, {
    props: { ariaLabel: "Actions", ...props },
    slots: {
      default: (state: ButtonGroupSlotState) => [
        h("output", { "data-group-state": state.state }, state.orientation),
        h(ButtonGroupItem, { value: "save", ...itemProps }, () => "Save"),
        h(ButtonGroupItem, { value: "publish" }, () => "Publish"),
      ],
    },
  });
}

test("renders grouped button semantics without adding visual CSS", () => {
  const handle = mountButtonGroup();
  const group = handle.getByRole("group", { name: "Actions" });
  const save = handle.getByRole("button", { name: "Save" }) as HTMLButtonElement;
  const publish = handle.getByRole("button", { name: "Publish" }) as HTMLButtonElement;

  assert.equal(group.getAttribute("data-vize-ui"), "button-group");
  assert.equal(group.getAttribute("data-state"), "idle");
  assert.equal(group.getAttribute("data-orientation"), "horizontal");
  assert.equal(group.getAttribute("data-role"), "group");
  assert.equal(group.getAttribute("data-roving-focus"), null);
  assert.equal(group.getAttribute("aria-orientation"), null);
  assert.equal(group.querySelector("[data-group-state]")?.textContent, "horizontal");
  assert.equal(save.type, "button");
  assert.equal(save.getAttribute("data-vize-ui"), "button-group-item");
  assert.equal(save.getAttribute("part"), "item");
  assert.equal(save.getAttribute("tabindex"), null);
  assert.equal(publish.getAttribute("data-state"), "idle");
  handle.unmount();
});

test("toolbar roving focus follows orientation and skips disabled", async () => {
  const handle = mountInteraction(ButtonGroup, {
    props: { ariaLabel: "Editor actions", orientation: "vertical", role: "toolbar" },
    slots: {
      default: () => [
        h(ButtonGroupItem, { value: "cut" }, () => "Cut"),
        h(ButtonGroupItem, { disabled: true, value: "copy" }, () => "Copy"),
        h(ButtonGroupItem, { value: "paste" }, () => "Paste"),
      ],
    },
  });
  const toolbar = handle.getByRole("toolbar", { name: "Editor actions" });
  const cut = handle.getByRole("button", { name: "Cut" });
  const copy = handle.getByRole("button", { name: "Copy" });
  const paste = handle.getByRole("button", { name: "Paste" });

  assert.equal(toolbar.getAttribute("aria-orientation"), "vertical");
  assert.equal(toolbar.getAttribute("data-roving-focus"), "true");
  assert.equal(cut.getAttribute("tabindex"), "0");
  assert.equal(copy.getAttribute("tabindex"), null);
  assert.equal(paste.getAttribute("tabindex"), "-1");

  assert.ok((await handle.tab()) === cut);
  const down = await handle.press(cut, "ArrowDown");
  assert.equal(down.keydownPrevented, true);
  assert.ok(handle.activeElement() === paste);
  assert.equal(cut.getAttribute("tabindex"), "-1");
  assert.equal(paste.getAttribute("tabindex"), "0");

  await handle.press(paste, "Home");
  assert.ok(handle.activeElement() === cut);
  await handle.press(cut, "ArrowUp");
  assert.ok(handle.activeElement() === paste);
  handle.unmount();
});

test("plain groups can opt into roving focus without toolbar role", async () => {
  const handle = mountButtonGroup({ rovingFocus: true });
  const group = handle.getByRole("group", { name: "Actions" });
  const save = handle.getByRole("button", { name: "Save" });
  const publish = handle.getByRole("button", { name: "Publish" });

  assert.equal(group.getAttribute("data-roving-focus"), "true");
  assert.equal(group.getAttribute("aria-orientation"), null);
  assert.equal(save.getAttribute("tabindex"), "0");
  await handle.press(save, "ArrowRight");
  assert.ok(handle.activeElement() === publish);
  handle.unmount();
});

test("pointer and keyboard activation emit value-carrying events", async () => {
  const itemPresses: unknown[][] = [];
  const handle = mountInteraction(ButtonGroup, {
    props: { ariaLabel: "Actions" },
    record: ["press"],
    slots: {
      default: () => [
        h(
          ButtonGroupItem,
          { onPress: (...args: unknown[]) => itemPresses.push(args), value: "save" },
          () => "Save",
        ),
        h(ButtonGroupItem, { as: "span", value: "publish" }, () => "Publish"),
      ],
    },
  });
  const save = handle.getByRole("button", { name: "Save" });
  const publish = handle.getByRole("button", { name: "Publish" });

  await handle.click(save);
  const enter = await handle.press(publish, "Enter");
  const space = await handle.press(publish, " ");

  assert.equal(enter.activated, false);
  assert.equal(space.keydownPrevented, true);
  assert.deepEqual(
    handle.recorded().map((emit) => [emit.event, emit.payload[0]]),
    [
      ["press", "save"],
      ["press", "publish"],
      ["press", "publish"],
    ],
  );
  assert.deepEqual(
    itemPresses.map((args) => args[0]),
    ["save"],
  );
  for (const emit of handle.recorded()) assert.ok(emit.payload[1] instanceof MouseEvent);
  handle.unmount();
});

test("disabled groups and items suppress activation", async () => {
  const groupDisabled = mountButtonGroup({ disabled: true });
  const group = groupDisabled.getByRole("group", { name: "Actions" });
  const save = groupDisabled.getByRole("button", { name: "Save" }) as HTMLButtonElement;

  assert.equal(group.getAttribute("aria-disabled"), "true");
  assert.equal(group.getAttribute("data-state"), "disabled");
  assert.equal(save.disabled, true);
  assert.equal(save.getAttribute("data-state"), "disabled");
  await groupDisabled.click(save);
  assert.equal(groupDisabled.wrapper.emitted("press"), undefined);
  assert.ok((await groupDisabled.tab()) === null);
  groupDisabled.unmount();

  const itemDisabled = mountButtonGroup({}, { disabled: true });
  const disabled = itemDisabled.getByRole("button", { name: "Save" }) as HTMLButtonElement;
  const enabled = itemDisabled.getByRole("button", { name: "Publish" });
  assert.equal(disabled.disabled, true);
  await itemDisabled.click(disabled);
  await itemDisabled.click(enabled);
  assert.deepEqual(
    itemDisabled.wrapper.emitted("press")?.map((emit) => emit[0]),
    ["publish"],
  );
  itemDisabled.unmount();
});

test("custom items expose button semantics and keyboard activation", async () => {
  const handle = mountInteraction(ButtonGroup, {
    props: { ariaLabel: "Actions" },
    slots: {
      default: () => h(ButtonGroupItem, { as: "span", value: "save" }, () => "Save"),
    },
  });
  const item = handle.getByRole("button", { name: "Save" });

  assert.equal(item.tagName, "SPAN");
  assert.equal(item.getAttribute("role"), "button");
  assert.equal(item.getAttribute("tabindex"), "0");
  await handle.press(item, " ");
  assert.deepEqual(
    handle.wrapper.emitted("press")?.map((emit) => emit[0]),
    ["save"],
  );
  handle.unmount();
});

test("exposes focus, focusValue, activeValue, and item state", async () => {
  let groupExpose: ButtonGroupExpose | null = null;
  let itemExpose: ButtonGroupItemExpose | null = null;
  const Probe = defineComponent({
    setup: () => () =>
      h(
        ButtonGroup,
        {
          ariaLabel: "Actions",
          ref: (value) => (groupExpose = value as ButtonGroupExpose | null),
          role: "toolbar",
        },
        () => [
          h(ButtonGroupItem, { value: "save" }, () => "Save"),
          h(
            ButtonGroupItem,
            {
              ref: (value) => (itemExpose = value as ButtonGroupItemExpose | null),
              value: "publish",
            },
            () => "Publish",
          ),
        ],
      ),
  });
  const handle = mountInteraction(Probe);
  const save = handle.getByRole("button", { name: "Save" });
  const publish = handle.getByRole("button", { name: "Publish" });

  if (groupExpose === null || itemExpose === null)
    assert.fail("ButtonGroup refs must expose state");
  await nextTick();
  groupExpose.focus();
  assert.ok(handle.activeElement() === save);
  assert.equal(groupExpose.activeValue, "save");
  assert.equal(groupExpose.focusValue("publish"), true);
  assert.ok(handle.activeElement() === publish);
  assert.equal(itemExpose.state, "idle");
  assert.equal(groupExpose.focusValue("missing"), false);
  itemExpose.focus();
  assert.ok(handle.activeElement() === publish);
  handle.unmount();
});

test("rejects duplicate item values before roving focus becomes ambiguous", async () => {
  assert.throws(
    () =>
      mountInteraction(ButtonGroup, {
        props: { ariaLabel: "Actions", role: "toolbar" },
        slots: {
          default: () => [
            h(ButtonGroupItem, { value: "save" }, () => "Save"),
            h(ButtonGroupItem, { value: "save" }, () => "Duplicate save"),
          ],
        },
      }),
    /VIZE_UI_BUTTON_GROUP_VALUE_DUPLICATE/,
  );

  const Probe = defineComponent({
    props: {
      secondaryValue: { type: String, default: "publish" },
    },
    setup: (props) => () =>
      h(ButtonGroup, { ariaLabel: "Actions", role: "toolbar" }, () => [
        h(ButtonGroupItem, { value: "save" }, () => "Save"),
        h(ButtonGroupItem, { value: props.secondaryValue }, () => "Publish"),
      ]),
  });
  const errors: unknown[] = [];
  const container = document.createElement("div");
  document.body.append(container);
  const wrapper = mount(Probe, {
    attachTo: container,
    global: {
      config: {
        errorHandler(error) {
          errors.push(error);
        },
      },
    },
  });

  await nextTick();
  await wrapper.setProps({ secondaryValue: "save" });
  assert.equal(errors.length, 1);
  assert.match(
    errors[0] instanceof Error ? errors[0].message : String(errors[0]),
    /VIZE_UI_BUTTON_GROUP_VALUE_DUPLICATE/,
  );
  wrapper.unmount();
  container.remove();
});

test("items require a matching group provider", () => {
  assert.throws(
    () => mountInteraction(ButtonGroupItem, { props: { value: "save" } }),
    /VIZE_UI_CONTEXT_MISSING/,
  );
});
