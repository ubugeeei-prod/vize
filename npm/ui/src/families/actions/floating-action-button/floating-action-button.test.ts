import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h, nextTick } from "vue";

import type { SpeedDialRootExpose, SpeedDialSelectEvent } from "./floating-action-button.ts";
import FloatingActionButton from "./floating-action-button.vue";
import SpeedDialAction from "./speed-dial-action.vue";
import SpeedDialContent from "./speed-dial-content.vue";
import SpeedDialRoot from "./speed-dial-root.vue";
import SpeedDialTrigger from "./speed-dial-trigger.vue";
import { mountInteraction } from "../../../testing/mount.ts";

async function settle(): Promise<void> {
  await nextTick();
  await new Promise((resolve) => setTimeout(resolve, 0));
  await nextTick();
}

function key(target: Element, name: string): KeyboardEvent {
  const event = new KeyboardEvent("keydown", { bubbles: true, cancelable: true, key: name });
  target.dispatchEvent(event);
  return event;
}

function pointer(target: Element, type: string, pointerType = "mouse"): void {
  target.dispatchEvent(new PointerEvent(type, { bubbles: true, cancelable: true, pointerType }));
}

function mountSpeedDial(props: Record<string, unknown> = {}) {
  return mountInteraction(SpeedDialRoot, {
    props: { id: "create", ...props },
    slots: {
      default: () => [
        h(SpeedDialTrigger, { ariaLabel: "Create" }, () => "+"),
        h(SpeedDialContent, null, () => [
          h(SpeedDialAction, { value: "note", label: "New note" }, () => "N"),
          h(SpeedDialAction, { value: "photo", label: "New photo", disabled: true }, () => "P"),
          h(SpeedDialAction, { value: "task", label: "New task" }, () => "T"),
        ]),
      ],
    },
  });
}

test("floating action button renders a labelled native button with placement data", async () => {
  const handle = mountInteraction(FloatingActionButton, {
    props: { ariaLabel: "Compose", placement: "bottom-start", extended: true },
    slots: { default: ({ placement }: { readonly placement: string }) => `Compose ${placement}` },
  });
  const button = handle.getByRole("button", { name: "Compose" });

  assert.ok(button instanceof HTMLButtonElement);
  assert.equal(button.type, "button");
  assert.equal(button.getAttribute("data-placement"), "bottom-start");
  assert.equal(button.getAttribute("data-extended"), "true");
  assert.equal(button.textContent, "Compose bottom-start");
  await handle.click(button);
  assert.equal(handle.wrapper.emitted("click")?.length, 1);

  handle.unmount();
});

test("speed dial trigger follows the menu button pattern", async () => {
  const handle = mountSpeedDial();
  const trigger = handle.getByRole("button", { name: "Create" });
  const menu = handle.root().querySelector('[data-vize-ui="speed-dial-content"]');
  assert.ok(menu instanceof HTMLElement);

  assert.equal(trigger.getAttribute("aria-haspopup"), "menu");
  assert.equal(trigger.getAttribute("aria-expanded"), "false");
  assert.equal(trigger.getAttribute("aria-controls"), "create-content");
  assert.equal(menu.id, "create-content");
  assert.equal(menu.getAttribute("role"), "menu");
  assert.equal(menu.getAttribute("aria-orientation"), "vertical");
  assert.equal(menu.getAttribute("aria-labelledby"), "create-trigger");
  assert.equal(menu.hidden, true);
  assert.equal(handle.root().getAttribute("data-direction"), "up");
  const actions = [...menu.querySelectorAll('[role="menuitem"]')];
  assert.deepEqual(
    actions.map((action) => action.getAttribute("aria-label")),
    ["New note", "New photo", "New task"],
  );

  pointer(trigger, "pointerdown");
  await handle.click(trigger);
  await settle();
  assert.equal(trigger.getAttribute("aria-expanded"), "true");
  assert.equal(menu.hidden, false);
  assert.ok(handle.activeElement() !== actions[0], "pointer opening keeps focus on the trigger");
  assert.deepEqual(handle.wrapper.emitted("open-change")?.[0]?.slice(0, 2), [true, "pointer"]);

  handle.unmount();
});

test("keyboard opening focuses the first action and arrows rove past disabled actions", async () => {
  const handle = mountSpeedDial();
  const trigger = handle.getByRole("button", { name: "Create" });
  trigger.focus();

  await handle.press(trigger, "Enter");
  await settle();
  const note = handle.getByRole("menuitem", { name: "New note" });
  const task = handle.getByRole("menuitem", { name: "New task" });
  assert.ok(handle.activeElement() === note);
  assert.equal(note.tabIndex, 0);
  assert.equal(task.tabIndex, -1);

  key(note, "ArrowDown");
  await settle();
  assert.ok(handle.activeElement() === task, "disabled actions are skipped");
  key(task, "ArrowDown");
  await settle();
  assert.ok(handle.activeElement() === note, "navigation wraps");
  key(note, "End");
  await settle();
  assert.ok(handle.activeElement() === task);

  key(task, "Escape");
  await settle();
  assert.equal(trigger.getAttribute("aria-expanded"), "false");
  assert.ok(handle.activeElement() === trigger, "Escape returns focus to the trigger");

  handle.unmount();
});

test("the direction arrow on the trigger opens into the menu", async () => {
  const handle = mountSpeedDial({ direction: "right" });
  const trigger = handle.getByRole("button", { name: "Create" });
  trigger.focus();

  assert.equal(key(trigger, "ArrowUp").defaultPrevented, false);
  const event = key(trigger, "ArrowRight");
  await settle();
  assert.equal(event.defaultPrevented, true);
  assert.equal(trigger.getAttribute("aria-expanded"), "true");
  assert.equal(
    handle.root().querySelector('[role="menu"]')?.getAttribute("aria-orientation"),
    "horizontal",
  );
  assert.ok(handle.activeElement() === handle.getByRole("menuitem", { name: "New note" }));

  handle.unmount();
});

test("selecting an action emits select, closes, and restores trigger focus", async () => {
  const selected: string[] = [];
  const handle = mountSpeedDial({
    defaultOpen: true,
    onSelect: (event: SpeedDialSelectEvent) => selected.push(event.value),
  });
  await settle();
  const task = handle.getByRole("menuitem", { name: "New task" });

  await handle.click(task);
  await settle();
  assert.deepEqual(selected, ["task"]);
  assert.equal(
    handle.getByRole("button", { name: "Create" }).getAttribute("aria-expanded"),
    "false",
  );
  assert.ok(handle.activeElement() === handle.getByRole("button", { name: "Create" }));
  assert.deepEqual(handle.wrapper.emitted("open-change")?.at(-1)?.slice(0, 2), [false, "action"]);

  handle.unmount();
});

test("preventDefault in select keeps the speed dial open", async () => {
  const handle = mountSpeedDial({
    defaultOpen: true,
    onSelect: (event: SpeedDialSelectEvent) => event.preventDefault(),
  });
  await settle();

  await handle.click(handle.getByRole("menuitem", { name: "New note" }));
  await settle();
  assert.equal(
    handle.getByRole("button", { name: "Create" }).getAttribute("aria-expanded"),
    "true",
  );

  handle.unmount();
});

test("outside pointer-down and Tab close the speed dial", async () => {
  const outside = document.createElement("button");
  document.body.append(outside);
  const handle = mountSpeedDial({ defaultOpen: true });
  await settle();
  const trigger = handle.getByRole("button", { name: "Create" });

  pointer(outside, "pointerdown");
  await settle();
  assert.equal(trigger.getAttribute("aria-expanded"), "false");

  await handle.click(trigger);
  await settle();
  key(handle.getByRole("menuitem", { name: "New note" }), "Tab");
  await settle();
  assert.equal(trigger.getAttribute("aria-expanded"), "false");

  handle.unmount();
  outside.remove();
});

test("hover opening is opt-in and ignores touch", async () => {
  const handle = mountSpeedDial({ openOnHover: true });
  const root = handle.root();
  const trigger = handle.getByRole("button", { name: "Create" });

  root.dispatchEvent(new PointerEvent("pointerenter", { pointerType: "touch" }));
  await settle();
  assert.equal(trigger.getAttribute("aria-expanded"), "false");
  root.dispatchEvent(new PointerEvent("pointerenter", { pointerType: "mouse" }));
  await settle();
  assert.equal(trigger.getAttribute("aria-expanded"), "true");
  root.dispatchEvent(new PointerEvent("pointerleave", { pointerType: "mouse" }));
  await settle();
  assert.equal(trigger.getAttribute("aria-expanded"), "false");
  assert.equal(handle.wrapper.emitted("open-change")?.[0]?.[1], "hover");

  handle.unmount();
});

test("disabled and controlled speed dials respect their owner", async () => {
  const disabled = mountSpeedDial({ disabled: true, defaultOpen: true });
  const disabledTrigger = disabled.getByRole("button", { name: "Create" });
  assert.equal((disabledTrigger as HTMLButtonElement).disabled, true);
  assert.equal(disabledTrigger.getAttribute("aria-expanded"), "false");
  disabled.unmount();

  const controlled = mountSpeedDial({ open: false });
  const trigger = controlled.getByRole("button", { name: "Create" });
  pointer(trigger, "pointerdown");
  await controlled.click(trigger);
  assert.equal(trigger.getAttribute("aria-expanded"), "false");
  assert.deepEqual(controlled.wrapper.emitted("update:open"), [[true]]);
  await controlled.wrapper.setProps({ open: true });
  assert.equal(trigger.getAttribute("aria-expanded"), "true");
  controlled.unmount();
});

test("root exposes ids and programmatic controls", async () => {
  let root: SpeedDialRootExpose | null = null;
  const Probe = defineComponent({
    name: "SpeedDialExposeProbe",
    setup: () => () =>
      h(
        SpeedDialRoot,
        {
          ref: (value) => {
            root = value as SpeedDialRootExpose | null;
          },
        },
        () => [
          h(SpeedDialTrigger, { ariaLabel: "Share" }, () => "S"),
          h(SpeedDialContent, null, () => h(SpeedDialAction, { value: "copy", label: "Copy" })),
        ],
      ),
  });
  const handle = mountInteraction(Probe);
  if (root === null) assert.fail("SpeedDialRoot must expose its API");
  const api: SpeedDialRootExpose = root;

  assert.match(api.triggerId, /-speed-dial-trigger$/);
  assert.equal(api.direction, "up");
  assert.equal(api.openAndFocus(), true);
  await settle();
  assert.equal(api.open, true);
  assert.ok(handle.activeElement() === handle.getByRole("menuitem", { name: "Copy" }));
  assert.equal(api.close({ focusTrigger: true }), true);
  await settle();
  assert.equal(api.state, "closed");
  assert.ok(handle.activeElement() === handle.getByRole("button", { name: "Share" }));
  assert.equal(api.setOpen(true), true);

  handle.unmount();
});

test("speed-dial parts require a root", () => {
  assert.throws(() => mountInteraction(SpeedDialTrigger), /VIZE_UI_CONTEXT_MISSING/);
  assert.throws(() => mountInteraction(SpeedDialContent), /VIZE_UI_CONTEXT_MISSING/);
  assert.throws(
    () => mountInteraction(SpeedDialAction, { props: { value: "a", label: "A" } }),
    /VIZE_UI_CONTEXT_MISSING/,
  );
});
