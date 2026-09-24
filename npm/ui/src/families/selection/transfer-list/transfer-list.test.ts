import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h, nextTick } from "vue";

import { mountInteraction } from "../../../testing/mount.ts";
import TransferListAction from "./transfer-list-action.vue";
import TransferListEmpty from "./transfer-list-empty.vue";
import TransferListItem from "./transfer-list-item.vue";
import TransferListPanel from "./transfer-list-panel.vue";
import TransferListRoot from "./transfer-list-root.vue";
import TransferListSearch from "./transfer-list-search.vue";
import { transferItems } from "./transfer-list-model.ts";
import type { TransferListSlotState } from "./transfer-list-types.ts";

interface Perm {
  readonly key: string;
  readonly label: string;
  readonly locked?: boolean;
}

const perms: readonly Perm[] = [
  { key: "read", label: "Read" },
  { key: "write", label: "Write" },
  { key: "delete", label: "Delete", locked: true },
  { key: "admin", label: "Admin" },
  { key: "audit", label: "Audit" },
];

function mountTransfer(props: Record<string, unknown> = {}) {
  return mountInteraction(TransferListRoot, {
    props: {
      by: "key",
      id: "perms",
      itemDisabled: (perm: Perm) => perm.locked === true,
      itemText: (perm: Perm) => perm.label,
      items: perms,
      ...props,
    },
    record: ["update:modelValue", "change", "update:sourceQuery"],
    slots: {
      default: (state: TransferListSlotState<Perm>) => [
        h(TransferListSearch, { ariaLabel: "Search available", side: "source" }),
        h(TransferListPanel, { ariaLabel: "Available", side: "source" }, () =>
          state.visibleSource.map((perm) =>
            h(TransferListItem<Perm>, { key: perm.key, value: perm }, () => perm.label),
          ),
        ),
        h(
          TransferListEmpty,
          { side: "source" },
          {
            default: ({ filtered }: { readonly filtered: boolean }) =>
              filtered ? "No matches" : "Nothing left",
          },
        ),
        h(TransferListAction, { action: "move-selected-to-target" }, () => ">"),
        h(TransferListAction, { action: "move-all-to-target" }, () => ">>"),
        h(TransferListAction, { action: "move-selected-to-source" }, () => "<"),
        h(TransferListAction, { action: "move-all-to-source" }, () => "<<"),
        h(TransferListPanel, { ariaLabel: "Selected", side: "target" }, () =>
          state.visibleTarget.map((perm) =>
            h(TransferListItem<Perm>, { key: perm.key, value: perm }, () => perm.label),
          ),
        ),
      ],
    },
  });
}

function labels(panel: HTMLElement): string[] {
  return [...panel.querySelectorAll("[role='option']")].map((option) => option.textContent ?? "");
}

function keydown(target: Element, key: string, init: KeyboardEventInit = {}): KeyboardEvent {
  const event = new KeyboardEvent("keydown", { bubbles: true, cancelable: true, key, ...init });
  target.dispatchEvent(event);
  return event;
}

async function settle(): Promise<void> {
  await nextTick();
  await nextTick();
}

function panels(handle: ReturnType<typeof mountTransfer>) {
  return {
    source: handle.getByRole("listbox", { name: "Available" }),
    target: handle.getByRole("listbox", { name: "Selected" }),
  };
}

function activeLabel(panel: HTMLElement): string | null {
  const id = panel.getAttribute("aria-activedescendant");
  return id === null ? null : (document.getElementById(id)?.textContent ?? null);
}

test("renders two multiselect listboxes that split items by the target model", async () => {
  const handle = mountTransfer({ defaultValue: [perms[3]], name: "perms" });
  await settle();
  const { source, target } = panels(handle);
  assert.equal(source.getAttribute("aria-multiselectable"), "true");
  assert.equal(source.id, "perms-source-listbox");
  assert.deepEqual(labels(source), ["Read", "Write", "Delete", "Audit"]);
  assert.deepEqual(labels(target), ["Admin"]);
  assert.equal(target.getAttribute("data-count"), "1");
  const hidden = handle
    .root()
    .querySelector<HTMLInputElement>("[data-vize-ui='transfer-list-native']");
  assert.equal(hidden?.value, "admin");
  assert.equal(
    handle.getByRole("button", { name: "Add selected" }).hasAttribute("disabled"),
    true,
    "nothing checked yet",
  );
  handle.unmount();
});

test("clicking checks items and the selected action moves them, emitting the move", async () => {
  const handle = mountTransfer();
  const { source, target } = panels(handle);
  await handle.click(handle.getByRole("option", { name: "Write" }));
  await handle.click(handle.getByRole("option", { name: "Read" }));
  assert.equal(handle.getByRole("option", { name: "Read" }).getAttribute("aria-selected"), "true");
  await handle.click(handle.getByRole("option", { name: "Delete" }));
  assert.equal(
    handle.getByRole("option", { name: "Delete" }).getAttribute("aria-selected"),
    "false",
    "disabled items cannot be checked",
  );
  await handle.click(handle.getByRole("button", { name: "Add selected" }));
  await settle();
  assert.deepEqual(labels(target), ["Write", "Read"], "append order by default");
  assert.deepEqual(labels(source), ["Delete", "Admin", "Audit"]);
  const change = handle.wrapper.emitted("change")?.[0];
  assert.deepEqual(change?.slice(0, 4), [
    [perms[1], perms[0]],
    [],
    [perms[1], perms[0]],
    "to-target",
  ]);
  handle.unmount();
});

test("source-order mode keeps canonical order and move-all skips disabled items", async () => {
  const handle = mountTransfer({ orderMode: "source-order" });
  const { source, target } = panels(handle);
  await handle.click(handle.getByRole("option", { name: "Audit" }));
  await handle.click(handle.getByRole("button", { name: "Add selected" }));
  await handle.click(handle.getByRole("button", { name: "Add all" }));
  await settle();
  assert.deepEqual(labels(target), ["Read", "Write", "Admin", "Audit"]);
  assert.deepEqual(labels(source), ["Delete"]);
  assert.equal(handle.getByRole("button", { name: "Add all" }).hasAttribute("disabled"), true);
  await handle.click(handle.getByRole("button", { name: "Remove all" }));
  await settle();
  assert.deepEqual(labels(target), []);
  assert.equal(
    handle.root().querySelector("[data-vize-ui='transfer-list-empty']")?.hasAttribute("hidden"),
    true,
  );
  handle.unmount();
});

test("max caps the target and disables moving further items", async () => {
  const handle = mountTransfer({ max: 2 });
  const { target } = panels(handle);
  await handle.click(handle.getByRole("button", { name: "Add all" }));
  await settle();
  assert.deepEqual(labels(target), ["Read", "Write"]);
  assert.equal(handle.root().getAttribute("data-full"), "true");
  assert.equal(handle.getByRole("button", { name: "Add all" }).hasAttribute("disabled"), true);
  handle.unmount();
});

test("search filters one panel accent-insensitively and drives the empty state", async () => {
  const handle = mountTransfer();
  const search = handle.getByRole("searchbox", { name: "Search available" });
  if (!(search instanceof HTMLInputElement)) assert.fail("search input expected");
  search.value = "adm";
  search.dispatchEvent(new Event("input", { bubbles: true }));
  await settle();
  assert.deepEqual(labels(panels(handle).source), ["Admin"]);
  assert.deepEqual(handle.wrapper.emitted("update:sourceQuery"), [["adm"]]);
  await handle.click(handle.getByRole("button", { name: "Add all" }));
  await settle();
  assert.deepEqual(labels(panels(handle).target), ["Admin"], "move-all moves only visible items");
  search.value = "zzz";
  search.dispatchEvent(new Event("input", { bubbles: true }));
  await settle();
  const empty = handle.root().querySelector("[data-vize-ui='transfer-list-empty']");
  assert.equal(empty?.textContent, "No matches");
  keydown(search, "Escape");
  await settle();
  assert.equal(search.value, "");
  keydown(search, "ArrowDown");
  assert.equal(document.activeElement, panels(handle).source, "ArrowDown enters the panel");
  handle.unmount();
});

test("keyboard: navigation, Space toggles, Shift+Arrow extends, Ctrl+A, Enter moves", async () => {
  const handle = mountTransfer();
  const { source, target } = panels(handle);
  source.focus();
  await settle();
  assert.equal(activeLabel(source), "Read");
  keydown(source, "ArrowDown");
  await settle();
  assert.equal(activeLabel(source), "Write");
  keydown(source, " ");
  keydown(source, "ArrowDown", { shiftKey: true });
  await settle();
  assert.equal(activeLabel(source), "Admin", "disabled Delete is skipped");
  assert.equal(handle.getByRole("option", { name: "Admin" }).getAttribute("aria-selected"), "true");
  const enter = keydown(source, "Enter");
  await settle();
  assert.equal(enter.defaultPrevented, true);
  assert.deepEqual(labels(target), ["Write", "Admin"]);
  assert.equal(activeLabel(source), "Audit", "the highlight recovers to a neighbour");

  keydown(source, "Home");
  keydown(source, "a", { ctrlKey: true });
  await settle();
  assert.deepEqual(
    [...source.querySelectorAll("[aria-selected='true']")].map((option) => option.textContent),
    ["Read", "Audit"],
  );
  target.focus();
  await settle();
  keydown(target, "Enter");
  await settle();
  assert.deepEqual(labels(target), ["Admin"], "Enter without checks moves the active item");
  handle.unmount();
});

test("double-click moves an item and typeahead finds items by text", async () => {
  const handle = mountTransfer();
  const { source, target } = panels(handle);
  handle
    .getByRole("option", { name: "Audit" })
    .dispatchEvent(new MouseEvent("dblclick", { bubbles: true }));
  await settle();
  assert.deepEqual(labels(target), ["Audit"]);
  source.focus();
  keydown(source, "a");
  await settle();
  assert.equal(activeLabel(source), "Admin");
  handle.unmount();
});

test("controlled target waits for the parent", async () => {
  const handle = mountTransfer({ modelValue: [] });
  await handle.click(handle.getByRole("button", { name: "Add all" }));
  await settle();
  assert.equal(handle.wrapper.emitted("update:modelValue")?.length, 1);
  assert.deepEqual(labels(panels(handle).target), []);
  await handle.wrapper.setProps({ modelValue: [perms[0]] });
  assert.deepEqual(labels(panels(handle).target), ["Read"]);
  handle.unmount();
});

test("disabled transfer lists block every interaction", async () => {
  const handle = mountTransfer({ disabled: true });
  const { source } = panels(handle);
  assert.equal(source.hasAttribute("tabindex"), false);
  await handle.click(handle.getByRole("option", { name: "Read" }));
  assert.equal(handle.getByRole("button", { name: "Add all" }).hasAttribute("disabled"), true);
  assert.equal(
    handle.getByRole("searchbox", { name: "Search available" }).hasAttribute("disabled"),
    true,
  );
  assert.equal(handle.wrapper.emitted("change"), undefined);
  handle.unmount();
});

test("exposed methods move checked items", async () => {
  let api: { moveToTarget: () => readonly Perm[]; target: readonly Perm[] } | null = null;
  const Probe = defineComponent({
    setup: () => () =>
      h(
        TransferListRoot<Perm>,
        {
          by: "key",
          items: perms,
          ref: (value) => {
            api = value as typeof api;
          },
        },
        () => [],
      ),
  });
  const handle = mountInteraction(Probe);
  if (api === null) assert.fail("expose expected");
  const exposed: { moveToTarget: () => readonly Perm[] } = api;
  assert.deepEqual(exposed.moveToTarget(), []);
  handle.unmount();
});

test("parts require a TransferList provider", () => {
  assert.throws(
    () => mountInteraction(TransferListPanel, { props: { side: "source" } }),
    /VIZE_UI_CONTEXT_MISSING/,
  );
});

test("transferItems respects max, order modes, and direction", () => {
  const equals = (left: string, right: string) => left === right;
  const items = ["a", "b", "c", "d"];
  assert.deepEqual(
    transferItems({
      direction: "to-target",
      equals,
      items,
      max: 3,
      moving: ["d", "a", "b"],
      orderMode: "append",
      target: ["c"],
    }),
    { moved: ["d", "a"], target: ["c", "d", "a"] },
  );
  assert.deepEqual(
    transferItems({
      direction: "to-target",
      equals,
      items,
      max: 9,
      moving: ["d", "a"],
      orderMode: "source-order",
      target: ["c"],
    }).target,
    ["a", "c", "d"],
  );
  assert.deepEqual(
    transferItems({
      direction: "to-source",
      equals,
      items,
      max: 9,
      moving: ["c", "x"],
      orderMode: "append",
      target: ["c", "d"],
    }),
    { moved: ["c"], target: ["d"] },
  );
});
