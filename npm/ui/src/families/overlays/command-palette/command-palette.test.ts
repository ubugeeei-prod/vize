import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h, nextTick, ref } from "vue";

import { createCommandRouter } from "../../foundations/command/command.ts";
import { detectShortcutPlatform } from "../../interaction/shortcut/shortcut.ts";
import type { CommandInfo } from "../../foundations/command/command.ts";
import type { CommandPaletteRootExpose, CommandPaletteSlotState } from "./command-palette.ts";
import { defaultCommandPaletteFilter } from "./command-palette.ts";
import CommandPaletteDialog from "./command-palette-dialog.vue";
import CommandPaletteEmpty from "./command-palette-empty.vue";
import CommandPaletteGroup from "./command-palette-group.vue";
import CommandPaletteInput from "./command-palette-input.vue";
import CommandPaletteItem from "./command-palette-item.vue";
import CommandPaletteList from "./command-palette-list.vue";
import CommandPaletteLoading from "./command-palette-loading.vue";
import CommandPaletteRoot from "./command-palette-root.vue";
import { mountInteraction } from "../../../testing/mount.ts";

const modifier: KeyboardEventInit =
  detectShortcutPlatform() === "apple" ? { metaKey: true } : { ctrlKey: true };

async function settle(): Promise<void> {
  await nextTick();
  await nextTick();
  await nextTick();
}

function key(target: Element, keyName: string, init: KeyboardEventInit = {}): KeyboardEvent {
  const event = new KeyboardEvent("keydown", {
    key: keyName,
    bubbles: true,
    cancelable: true,
    ...init,
  });
  target.dispatchEvent(event);
  return event;
}

async function type(input: HTMLInputElement, value: string): Promise<void> {
  input.value = value;
  input.dispatchEvent(new Event("input", { bubbles: true }));
  await settle();
}

interface FixtureOptions {
  readonly rootProps?: Record<string, unknown>;
  readonly onSelect?: (value: unknown) => void;
}

function mountPalette(options: FixtureOptions = {}) {
  return mountInteraction(CommandPaletteRoot, {
    props: { id: "palette", ...options.rootProps },
    slots: {
      default: () => [
        h(CommandPaletteInput, { placeholder: "Type a command" }),
        h(CommandPaletteList, null, () => [
          h(CommandPaletteGroup, { heading: "Files" }, () => [
            h(CommandPaletteItem, {
              value: "new",
              textValue: "New file",
              onSelect: options.onSelect,
            }),
            h(CommandPaletteItem, {
              value: "open",
              textValue: "Open file",
              keywords: ["browse"],
              onSelect: options.onSelect,
            }),
            h(CommandPaletteItem, { value: "save", textValue: "Save", disabled: true }),
          ]),
          h(CommandPaletteGroup, { heading: "View" }, () => [
            h(CommandPaletteItem, { value: "zoom", textValue: "Zoom in", shortcut: "Control+=" }),
          ]),
          h(CommandPaletteEmpty, null, () => "Nothing found"),
          h(CommandPaletteLoading, null, () => "Loading"),
        ]),
      ],
    },
  });
}

function parts(handle: ReturnType<typeof mountPalette>) {
  const input = handle.getByRole("combobox");
  assert.ok(input instanceof HTMLInputElement);
  const list = handle.getByRole("listbox");
  const options = [...handle.root().querySelectorAll<HTMLElement>('[role="option"]')];
  const empty = handle.root().querySelector<HTMLElement>('[data-vize-ui="command-palette-empty"]');
  assert.ok(empty);
  return { input, list, options, empty };
}

test("renders combobox and listbox semantics with an active descendant", async () => {
  const handle = mountPalette();
  await settle();
  const { input, list, options } = parts(handle);

  assert.equal(handle.root().getAttribute("data-vize-ui"), "command-palette-root");
  assert.equal(input.id, "palette-input");
  assert.equal(input.getAttribute("aria-controls"), "palette-list");
  assert.equal(input.getAttribute("aria-expanded"), "true");
  assert.equal(input.getAttribute("aria-autocomplete"), "list");
  assert.equal(list.id, "palette-list");
  assert.equal(options.length, 4);
  assert.equal(input.getAttribute("aria-activedescendant"), options[0]?.id);
  assert.equal(options[0]?.getAttribute("aria-selected"), "true");
  assert.equal(options[1]?.getAttribute("aria-selected"), "false");
  assert.equal(options[2]?.getAttribute("aria-disabled"), "true");
  assert.equal(options[3]?.getAttribute("aria-keyshortcuts"), "Control+=");
  const group = handle.getByRole("group", { name: "Files" });
  assert.ok(group.contains(options[0] ?? null));

  handle.unmount();
});

test("typing filters items and groups, resets the active item, and announces the count", async () => {
  const handle = mountPalette();
  await settle();
  const { input, options, empty } = parts(handle);

  await type(input, "brow");
  assert.equal(options[0]?.hidden, true);
  assert.equal(options[1]?.hidden, false, "keywords match");
  assert.equal(input.getAttribute("aria-activedescendant"), options[1]?.id);
  assert.equal(handle.getByRole("group", { name: "View" }).hidden, true);
  assert.deepEqual(handle.wrapper.emitted("update:search")?.at(-1), ["brow"]);
  assert.equal(handle.root().querySelector('[role="status"]')?.textContent?.trim(), "1 result");

  await type(input, "zzzz");
  assert.equal(empty.hidden, false);
  assert.equal(handle.getByRole("listbox").getAttribute("data-empty"), "true");
  assert.equal(input.getAttribute("aria-activedescendant"), null);
  assert.equal(handle.root().querySelector('[role="status"]')?.textContent?.trim(), "0 results");

  handle.unmount();
});

test("arrow keys, Home, and End move the active option and skip disabled ones", async () => {
  const handle = mountPalette();
  await settle();
  const { input, options } = parts(handle);

  assert.equal(key(input, "ArrowDown").defaultPrevented, true);
  await settle();
  assert.equal(input.getAttribute("aria-activedescendant"), options[1]?.id);
  key(input, "ArrowDown");
  await settle();
  assert.equal(input.getAttribute("aria-activedescendant"), options[3]?.id, "disabled skipped");
  key(input, "ArrowDown");
  await settle();
  assert.equal(input.getAttribute("aria-activedescendant"), options[0]?.id, "wraps");
  key(input, "End");
  await settle();
  assert.equal(input.getAttribute("aria-activedescendant"), options[3]?.id);
  key(input, "Home");
  await settle();
  assert.equal(input.getAttribute("aria-activedescendant"), options[0]?.id);
  key(input, "ArrowUp");
  await settle();
  assert.equal(input.getAttribute("aria-activedescendant"), options[3]?.id);

  handle.unmount();
});

test("Enter selects the active item, IME composition is ignored, and Escape clears the search", async () => {
  const selected: unknown[] = [];
  const handle = mountPalette({ onSelect: (value) => selected.push(value) });
  await settle();
  const { input } = parts(handle);

  assert.equal(key(input, "Enter", { isComposing: true }).defaultPrevented, false);
  assert.deepEqual(selected, []);
  key(input, "ArrowDown");
  await settle();
  assert.equal(key(input, "Enter").defaultPrevented, true);
  assert.deepEqual(selected, ["open"]);
  const select = handle.wrapper.emitted("select")?.[0];
  assert.equal(select?.[0], null);
  assert.ok(select?.[1] instanceof KeyboardEvent);

  await type(input, "new");
  assert.equal(key(input, "Escape").defaultPrevented, true);
  await settle();
  assert.equal(input.value, "");
  assert.equal(key(input, "Escape").defaultPrevented, false, "empty search lets Escape through");

  handle.unmount();
});

test("pointer movement activates options and clicks select without stealing focus", async () => {
  const selected: unknown[] = [];
  const handle = mountPalette({ onSelect: (value) => selected.push(value) });
  await settle();
  const { input, options } = parts(handle);
  const open = options[1];
  assert.ok(open);

  open.dispatchEvent(new PointerEvent("pointermove", { bubbles: true }));
  await settle();
  assert.equal(input.getAttribute("aria-activedescendant"), open.id);
  const mousedown = new MouseEvent("mousedown", { bubbles: true, cancelable: true });
  open.dispatchEvent(mousedown);
  assert.equal(mousedown.defaultPrevented, true);
  open.click();
  assert.deepEqual(selected, ["open"]);
  options[2]?.click();
  assert.deepEqual(selected, ["open"], "disabled options refuse selection");

  handle.unmount();
});

test("router commands render through the slot, run with the palette source, and track recents", async () => {
  const runs: string[] = [];
  const enabled = ref(false);
  const router = createCommandRouter<"theme" | "reload" | "deploy">();
  router.register({
    id: "theme",
    title: "Toggle theme",
    keywords: ["dark"],
    run: ({ source }) => runs.push(`theme:${source}`),
  });
  router.register({ id: "reload", title: "Reload window", run: () => runs.push("reload") });
  router.register({ id: "deploy", title: "Deploy", when: enabled, run: () => runs.push("deploy") });
  let rootExpose: CommandPaletteRootExpose | null = null;
  const seen: { recent: readonly CommandInfo<string>[] }[] = [];
  const Probe = defineComponent({
    name: "CommandPaletteRouterProbe",
    setup: () => () =>
      h(
        CommandPaletteRoot,
        {
          router,
          ref: (value) => {
            rootExpose = value as CommandPaletteRootExpose | null;
          },
        },
        {
          default: (state: CommandPaletteSlotState) => {
            seen.push({ recent: state.recentCommands });
            return [
              h(CommandPaletteInput),
              h(CommandPaletteList, null, () =>
                state.commands.map((command) =>
                  h(CommandPaletteItem, { key: command.id, command: command.id }),
                ),
              ),
            ];
          },
        },
      ),
  });
  const handle = mountInteraction(Probe);
  await settle();
  const input = handle.getByRole("combobox");
  assert.ok(input instanceof HTMLInputElement);
  const options = [...handle.root().querySelectorAll<HTMLElement>('[role="option"]')];

  assert.equal(options.length, 3);
  assert.equal(options[0]?.textContent?.trim(), "Toggle theme");
  assert.equal(options[2]?.getAttribute("aria-disabled"), "true");
  await type(input, "dark");
  assert.equal(handle.root().querySelectorAll('[role="option"]').length, 1);
  key(input, "Enter");
  await settle();
  assert.deepEqual(runs, ["theme:palette"]);
  assert.deepEqual(
    seen.at(-1)?.recent.map((command) => command.id),
    ["theme"],
  );
  if (rootExpose === null) assert.fail("root must expose its API");
  const root: CommandPaletteRootExpose = rootExpose;
  assert.equal(root.search, "dark");
  assert.equal(root.setSearch(""), true);
  assert.equal(root.resultCount >= 0, true);

  handle.unmount();
});

test("shouldFilter=false, custom filters, loading, and controlled search are honored", async () => {
  const unfiltered = mountPalette({ rootProps: { shouldFilter: false, search: "zzz" } });
  await settle();
  assert.equal(
    parts(unfiltered).options.every((option) => !option.hidden),
    true,
  );
  unfiltered.unmount();

  const custom = mountPalette({
    rootProps: { filter: (text: string) => (text.startsWith("Z") ? 1 : 0), defaultSearch: "x" },
  });
  await settle();
  assert.deepEqual(
    parts(custom).options.map((option) => option.hidden),
    [true, true, true, false],
  );
  custom.unmount();

  const loading = mountPalette({ rootProps: { loading: true, defaultSearch: "zzz" } });
  await settle();
  const { list, empty } = parts(loading);
  assert.equal(list.getAttribute("aria-busy"), "true");
  assert.equal(empty.hidden, true, "empty waits for loading to finish");
  assert.equal(loading.getByRole("progressbar", { name: "Loading results" }).hidden, false);
  loading.unmount();

  const controlled = mountPalette({ rootProps: { search: "" } });
  await settle();
  const input = parts(controlled).input;
  await type(input, "save");
  assert.deepEqual(controlled.wrapper.emitted("update:search"), [["save"]]);
  assert.equal(parts(controlled).options[0]?.hidden, false, "controlled search waits for parent");
  await controlled.wrapper.setProps({ search: "save" });
  await settle();
  assert.equal(parts(controlled).options[0]?.hidden, true);
  controlled.unmount();
});

test("closed palettes hide the list and arrows reopen it", async () => {
  const handle = mountPalette({ rootProps: { defaultOpen: false } });
  await settle();
  const { input, list } = parts(handle);

  assert.equal(input.getAttribute("aria-expanded"), "false");
  assert.equal(list.hidden, true);
  assert.equal(input.getAttribute("aria-activedescendant"), null);
  key(input, "ArrowDown");
  await settle();
  assert.equal(input.getAttribute("aria-expanded"), "true");
  assert.deepEqual(handle.wrapper.emitted("update:open"), [[true]]);

  handle.unmount();
});

test("the dialog toggles with Mod+K and closes after a selection", async () => {
  const selected: unknown[] = [];
  const handle = mountInteraction(CommandPaletteDialog, {
    props: { shortcut: "Mod+K" },
    slots: {
      default: () =>
        h(CommandPaletteRoot, null, () => [
          h(CommandPaletteInput),
          h(CommandPaletteList, null, () =>
            h(CommandPaletteItem, {
              value: 1,
              textValue: "First",
              onSelect: (value: unknown) => selected.push(value),
            }),
          ),
        ]),
    },
  });
  await settle();
  assert.equal(handle.root().getAttribute("data-state"), "closed");
  assert.equal(document.querySelector('[data-vize-ui="command-palette-input"]'), null);

  key(document.body, "k", modifier);
  await settle();
  assert.equal(handle.root().getAttribute("data-state"), "open");
  const input = document.querySelector<HTMLInputElement>('[data-vize-ui="command-palette-input"]');
  assert.ok(input);
  assert.equal(
    document.querySelector('[data-vize-ui="dialog-content"]')?.getAttribute("aria-label"),
    "Command palette",
  );
  key(input, "Enter");
  await settle();
  assert.deepEqual(selected, [1]);
  assert.equal(handle.root().getAttribute("data-state"), "closed");
  assert.deepEqual(handle.wrapper.emitted("update:open"), [[true], [false]]);

  await handle.wrapper.setProps({ shortcut: null });
  key(document.body, "k", modifier);
  await settle();
  assert.equal(handle.root().getAttribute("data-state"), "closed", "null shortcut disables");

  handle.unmount();
});

test("default filter ranks exact, prefix, word, substring, and subsequence matches", () => {
  const score = (text: string, search: string, keywords: readonly string[] = []) =>
    defaultCommandPaletteFilter(text, search, keywords);
  assert.equal(score("Open", ""), 1);
  assert.ok(score("Open", "open") > score("Open file", "open"));
  assert.ok(score("Open file", "open") > score("Reopen file", "file"));
  assert.ok(score("Reopen file", "file") > score("Reopen", "ope"));
  assert.ok(score("Reopen", "ope") > score("Toggle theme", "tgth"));
  assert.ok(score("Toggle theme", "tgth") > 0);
  assert.equal(score("Toggle theme", "xyz"), 0);
  assert.ok(score("Café", "cafe") > 0, "diacritics are ignored");
  assert.ok(score("Settings", "prefs", ["preferences"]) > 0);
});

test("palette parts require a root provider", () => {
  assert.throws(() => mountInteraction(CommandPaletteInput), /VIZE_UI_CONTEXT_MISSING/);
  assert.throws(() => mountInteraction(CommandPaletteList), /VIZE_UI_CONTEXT_MISSING/);
  assert.throws(() => mountInteraction(CommandPaletteItem), /VIZE_UI_CONTEXT_MISSING/);
  assert.throws(
    () => mountInteraction(CommandPaletteGroup, { props: { heading: "G" } }),
    /VIZE_UI_CONTEXT_MISSING/,
  );
  assert.throws(() => mountInteraction(CommandPaletteEmpty), /VIZE_UI_CONTEXT_MISSING/);
  assert.throws(() => mountInteraction(CommandPaletteLoading), /VIZE_UI_CONTEXT_MISSING/);
});
