import assert from "node:assert/strict";

import { mount } from "@vue/test-utils";
import { test } from "vite-plus/test";
import { defineComponent, h, nextTick } from "vue";

import { mountInteraction } from "../../../testing/mount.ts";
import ToolbarItem from "./toolbar-item.vue";
import Toolbar from "./toolbar.vue";
import type { ToolbarExpose, ToolbarItemExpose, ToolbarSlotState } from "./toolbar.ts";

function mountToolbar(
  props: Record<string, unknown> = {},
  itemProps: Record<string, unknown> = {},
) {
  return mountInteraction(Toolbar, {
    props: { ariaLabel: "Editor actions", ...props },
    slots: {
      default: (state: ToolbarSlotState) => [
        h("output", { "data-toolbar-state": state.state }, `${state.orientation}:${state.dir}`),
        h(ToolbarItem, { value: "save", ...itemProps }, () => "Save"),
        h(ToolbarItem, { value: "publish" }, () => "Publish"),
      ],
    },
  });
}

test("renders accessible toolbar semantics without adding visual CSS", () => {
  const handle = mountToolbar();
  const toolbar = handle.getByRole("toolbar", { name: "Editor actions" });
  const save = handle.getByRole("button", { name: "Save" }) as HTMLButtonElement;
  const publish = handle.getByRole("button", { name: "Publish" }) as HTMLButtonElement;

  assert.equal(toolbar.getAttribute("data-vize-ui"), "toolbar");
  assert.equal(toolbar.getAttribute("part"), "root");
  assert.equal(toolbar.getAttribute("aria-orientation"), "horizontal");
  assert.equal(toolbar.getAttribute("dir"), "ltr");
  assert.equal(toolbar.getAttribute("data-state"), "idle");
  assert.equal(toolbar.getAttribute("data-orientation"), "horizontal");
  assert.equal(toolbar.getAttribute("data-roving-focus"), "true");
  assert.equal(toolbar.getAttribute("class"), null);
  assert.equal(toolbar.style.getPropertyValue("--vize-ui-toolbar-orientation"), "horizontal");
  assert.equal(toolbar.querySelector("[data-toolbar-state]")?.textContent, "horizontal:ltr");
  assert.equal(save.type, "button");
  assert.equal(save.getAttribute("data-vize-ui"), "toolbar-item");
  assert.equal(save.getAttribute("part"), "item");
  assert.equal(save.getAttribute("data-state"), "idle");
  assert.equal(save.getAttribute("tabindex"), "0");
  assert.equal(publish.getAttribute("tabindex"), "-1");
  handle.unmount();
});

test("roving focus follows vertical orientation and skips disabled items", async () => {
  const handle = mountInteraction(Toolbar, {
    props: { ariaLabel: "Editor actions", orientation: "vertical" },
    slots: {
      default: () => [
        h(ToolbarItem, { value: "cut" }, () => "Cut"),
        h(ToolbarItem, { disabled: true, value: "copy" }, () => "Copy"),
        h(ToolbarItem, { value: "paste" }, () => "Paste"),
      ],
    },
  });
  const toolbar = handle.getByRole("toolbar", { name: "Editor actions" });
  const cut = handle.getByRole("button", { name: "Cut" });
  const copy = handle.getByRole("button", { name: "Copy" });
  const paste = handle.getByRole("button", { name: "Paste" });

  assert.equal(toolbar.getAttribute("aria-orientation"), "vertical");
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

test("roving focus can be disabled to preserve the native tab order", async () => {
  const handle = mountToolbar({ rovingFocus: false });
  const toolbar = handle.getByRole("toolbar", { name: "Editor actions" });
  const save = handle.getByRole("button", { name: "Save" });
  const publish = handle.getByRole("button", { name: "Publish" });

  assert.equal(toolbar.getAttribute("data-roving-focus"), null);
  assert.equal(save.getAttribute("tabindex"), null);
  assert.equal(publish.getAttribute("tabindex"), null);
  assert.ok((await handle.tab()) === save);
  assert.ok((await handle.tab()) === publish);
  const right = await handle.press(save, "ArrowRight");
  assert.equal(right.keydownPrevented, false);
  assert.ok(handle.activeElement() === publish);
  handle.unmount();
});

test("horizontal roving focus respects rtl direction", async () => {
  const handle = mountToolbar({ dir: "rtl" });
  const toolbar = handle.getByRole("toolbar", { name: "Editor actions" });
  const save = handle.getByRole("button", { name: "Save" });
  const publish = handle.getByRole("button", { name: "Publish" });

  assert.equal(toolbar.getAttribute("dir"), "rtl");
  assert.ok((await handle.tab()) === save);
  const left = await handle.press(save, "ArrowLeft");
  assert.equal(left.keydownPrevented, true);
  assert.ok(handle.activeElement() === publish);
  const right = await handle.press(publish, "ArrowRight");
  assert.equal(right.keydownPrevented, true);
  assert.ok(handle.activeElement() === save);
  handle.unmount();
});

test("pointer and keyboard activation emit value-carrying events", async () => {
  const itemPresses: unknown[][] = [];
  const handle = mountInteraction(Toolbar, {
    props: { ariaLabel: "Editor actions" },
    record: ["press"],
    slots: {
      default: () => [
        h(
          ToolbarItem,
          { onPress: (...args: unknown[]) => itemPresses.push(args), value: "save" },
          () => "Save",
        ),
        h(ToolbarItem, { as: "span", value: "publish" }, () => "Publish"),
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

test("disabled toolbars and items suppress activation", async () => {
  const toolbarDisabled = mountToolbar({ disabled: true });
  const toolbar = toolbarDisabled.getByRole("toolbar", { name: "Editor actions" });
  const save = toolbarDisabled.getByRole("button", { name: "Save" }) as HTMLButtonElement;

  assert.equal(toolbar.getAttribute("aria-disabled"), "true");
  assert.equal(toolbar.getAttribute("data-state"), "disabled");
  assert.equal(save.disabled, true);
  assert.equal(save.getAttribute("data-state"), "disabled");
  await toolbarDisabled.click(save);
  assert.equal(toolbarDisabled.wrapper.emitted("press"), undefined);
  assert.ok((await toolbarDisabled.tab()) === null);
  toolbarDisabled.unmount();

  const itemDisabled = mountToolbar({}, { disabled: true });
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
  const handle = mountInteraction(Toolbar, {
    props: { ariaLabel: "Editor actions" },
    slots: {
      default: () => h(ToolbarItem, { as: "span", value: "save" }, () => "Save"),
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

test("exposes focus, focusValue, activeValue, and live state", async () => {
  let toolbarExpose: ToolbarExpose | null = null;
  let itemExpose: ToolbarItemExpose | null = null;
  const Probe = defineComponent({
    setup: () => () =>
      h(
        Toolbar,
        {
          ariaLabel: "Editor actions",
          ref: (value) => (toolbarExpose = value as ToolbarExpose | null),
        },
        () => [
          h(ToolbarItem, { value: "save" }, () => "Save"),
          h(
            ToolbarItem,
            {
              ref: (value) => (itemExpose = value as ToolbarItemExpose | null),
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

  if (toolbarExpose === null || itemExpose === null) {
    assert.fail("Toolbar refs must expose state");
  }
  await nextTick();
  toolbarExpose.focus();
  assert.ok(handle.activeElement() === save);
  assert.equal(toolbarExpose.activeValue, "save");
  assert.equal(toolbarExpose.focusValue("publish"), true);
  assert.ok(handle.activeElement() === publish);
  assert.equal(toolbarExpose.style["--vize-ui-toolbar-orientation"], "horizontal");
  assert.equal(itemExpose.state, "idle");
  assert.equal(itemExpose.dir, "ltr");
  assert.equal(toolbarExpose.focusValue("missing"), false);
  itemExpose.focus();
  assert.ok(handle.activeElement() === publish);
  handle.unmount();
});

test("rejects duplicate item values before roving focus becomes ambiguous", async () => {
  assert.throws(
    () =>
      mountInteraction(Toolbar, {
        props: { ariaLabel: "Editor actions" },
        slots: {
          default: () => [
            h(ToolbarItem, { value: "save" }, () => "Save"),
            h(ToolbarItem, { value: "save" }, () => "Duplicate save"),
          ],
        },
      }),
    /VIZE_UI_TOOLBAR_VALUE_DUPLICATE/,
  );

  const Probe = defineComponent({
    props: {
      secondaryValue: { type: String, default: "publish" },
    },
    setup: (props) => () =>
      h(Toolbar, { ariaLabel: "Editor actions" }, () => [
        h(ToolbarItem, { value: "save" }, () => "Save"),
        h(ToolbarItem, { value: props.secondaryValue }, () => "Publish"),
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
    /VIZE_UI_TOOLBAR_VALUE_DUPLICATE/,
  );
  wrapper.unmount();
  container.remove();
});

test("items require a matching toolbar provider", () => {
  assert.throws(
    () => mountInteraction(ToolbarItem, { props: { value: "save" } }),
    /VIZE_UI_CONTEXT_MISSING/,
  );
});
