import assert from "node:assert/strict";

import { afterEach, beforeEach, test, vi } from "vite-plus/test";
import { h, nextTick } from "vue";

import {
  ActionSheet,
  ActionSheetCancel,
  ActionSheetContent,
  ActionSheetTitle,
  ActionSheetTrigger,
} from "./action-sheet.ts";
import ActionSheetItem from "./action-sheet-item.vue";
import ActionSheetMenu from "./action-sheet-menu.vue";
import type { ActionSheetSelectEvent } from "./action-sheet-types.ts";
import { mountInteraction } from "../../../testing/mount.ts";

beforeEach(() => {
  vi.spyOn(HTMLDialogElement.prototype, "getBoundingClientRect").mockImplementation(() =>
    DOMRect.fromRect({ height: 300, width: 300, x: 0, y: 500 }),
  );
});

afterEach(() => {
  vi.restoreAllMocks();
});

async function settle(): Promise<void> {
  for (let index = 0; index < 3; index += 1) {
    await nextTick();
    await Promise.resolve();
  }
}

function mountSheet(onSelect: (event: ActionSheetSelectEvent) => void = () => undefined) {
  return mountInteraction(ActionSheet, {
    props: { id: "photo" },
    record: ["update:open"],
    slots: {
      default: () => [
        h(ActionSheetTrigger, null, () => "Photo options"),
        h(ActionSheetContent, null, () => [
          h(ActionSheetTitle, null, () => "Photo"),
          h(ActionSheetMenu, null, () => [
            h(ActionSheetItem, { value: "share", onSelect }, () => "Share"),
            h(ActionSheetItem, { value: "archive", disabled: true, onSelect }, () => "Archive"),
            h(ActionSheetItem, { value: "copy", onSelect }, () => "Copy link"),
            h(ActionSheetItem, { value: "delete", destructive: true, onSelect }, () => "Delete"),
          ]),
          h(ActionSheetCancel, null, () => "Cancel"),
        ]),
      ],
    },
  });
}

function items(root: HTMLElement): HTMLElement[] {
  return [...root.querySelectorAll<HTMLElement>('[role="menuitem"]')];
}

async function open(root: HTMLElement): Promise<void> {
  root.querySelector<HTMLButtonElement>('[data-vize-ui="dialog-trigger"], button')?.click();
  await settle();
}

async function key(target: Element, name: string): Promise<void> {
  target.dispatchEvent(
    new KeyboardEvent("keydown", { key: name, bubbles: true, cancelable: true }),
  );
  await settle();
}

test("opens a bottom sheet whose actions form a menu labelled by the title", async () => {
  const handle = mountSheet();
  await open(handle.root());
  const menu = handle.root().querySelector('[role="menu"]');
  const title = handle.root().querySelector('[data-vize-ui="dialog-title"], h2');
  assert.ok(menu instanceof HTMLElement && title instanceof HTMLElement);
  assert.equal(menu.getAttribute("aria-labelledby"), title.id);
  assert.equal(menu.getAttribute("aria-orientation"), "vertical");
  const [share, archive, , remove] = items(handle.root());
  assert.equal(share?.tabIndex, 0, "the first enabled action is the tab stop");
  assert.equal(archive?.getAttribute("aria-disabled"), "true");
  assert.equal(remove?.getAttribute("data-destructive"), "true");
  assert.equal(handle.root().querySelector("dialog")?.getAttribute("data-side"), "bottom");
  handle.unmount();
});

test("arrow keys, Home, End, and typeahead move a roving focus that skips disabled actions", async () => {
  const handle = mountSheet();
  await open(handle.root());
  const [share, , copy, remove] = items(handle.root());
  share?.focus();
  await key(share as HTMLElement, "ArrowDown");
  assert.ok(document.activeElement === copy, "disabled actions are skipped");
  assert.equal(copy?.tabIndex, 0);
  assert.equal(share?.tabIndex, -1);
  await key(copy as HTMLElement, "End");
  assert.ok(document.activeElement === remove);
  await key(remove as HTMLElement, "ArrowDown");
  assert.ok(document.activeElement === share, "focus wraps");
  await key(share as HTMLElement, "ArrowUp");
  assert.ok(document.activeElement === remove);
  await key(remove as HTMLElement, "c");
  assert.ok(document.activeElement === copy, "typeahead matches the label");
  await key(copy as HTMLElement, "Home");
  assert.ok(document.activeElement === share);
  handle.unmount();
});

test("selecting an action emits select and closes; preventDefault keeps the sheet open", async () => {
  const selected: string[] = [];
  let keepOpen = true;
  const handle = mountSheet((event) => {
    selected.push(event.value);
    if (keepOpen) event.preventDefault();
  });
  await open(handle.root());
  const [share, archive, copy] = items(handle.root());

  share?.click();
  await settle();
  assert.equal(handle.root().querySelector("dialog")?.open, true, "prevented selections stay open");
  archive?.click();
  keepOpen = false;
  await key(copy as HTMLElement, "Enter");
  assert.deepEqual(selected, ["share", "copy"], "disabled actions never select");
  assert.deepEqual(handle.recorded().at(-1), { event: "update:open", payload: [false] });
  handle.unmount();
});

test("menus and items require their providers", () => {
  const warn = console.warn;
  console.warn = () => undefined;
  try {
    assert.throws(() => mountInteraction(ActionSheetMenu), /VIZE_UI_CONTEXT_MISSING: Dialog/);
    assert.throws(
      () => mountInteraction(ActionSheetItem, { props: { value: "x" } }),
      /VIZE_UI_CONTEXT_MISSING: ActionSheetMenu/,
    );
  } finally {
    console.warn = warn;
  }
});
