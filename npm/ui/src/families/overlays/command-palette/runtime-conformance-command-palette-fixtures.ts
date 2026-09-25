import assert from "node:assert/strict";

import { h } from "vue";

import {
  CommandPaletteDialog,
  CommandPaletteEmpty,
  CommandPaletteGroup,
  CommandPaletteInput,
  CommandPaletteItem,
  CommandPaletteList,
  CommandPaletteLoading,
  CommandPaletteRoot,
} from "./command-palette.ts";
import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";

const commandPaletteFamilyRoot = "families/overlays/command-palette/";

function palette(children: () => unknown): ReturnType<typeof h> {
  return h(CommandPaletteRoot, { id: "runtime-palette", loading: true }, children);
}

function listWith(children: () => unknown): ReturnType<typeof h> {
  return palette(() => [h(CommandPaletteInput), h(CommandPaletteList, null, children)]);
}

function expectPart(host: HTMLElement, name: string): HTMLElement {
  const element = host.querySelector(`[data-vize-ui="${name}"]`);
  assert.ok(element instanceof HTMLElement);
  return element;
}

export const commandPaletteRuntimeFixtures: readonly RuntimeFixture[] = [
  {
    name: "command-palette-root",
    sourceFile: `${commandPaletteFamilyRoot}command-palette-root.vue`,
    render: () => palette(() => "Palette"),
    assertServerMarkup(html) {
      assert.match(html, /id="runtime-palette"/);
      assert.match(html, /data-vize-ui="command-palette-root"/);
      assert.match(html, /role="status"/);
    },
    assertHydratedDom(host) {
      assert.equal(expectPart(host, "command-palette-root").getAttribute("data-state"), "open");
    },
  },
  {
    name: "command-palette-input",
    sourceFile: `${commandPaletteFamilyRoot}command-palette-input.vue`,
    render: () => listWith(() => "List"),
    assertServerMarkup(html) {
      assert.match(html, /role="combobox"/);
      assert.match(html, /aria-controls="runtime-palette-list"/);
    },
    assertHydratedDom(host) {
      assert.ok(expectPart(host, "command-palette-input") instanceof HTMLInputElement);
    },
  },
  {
    name: "command-palette-list",
    sourceFile: `${commandPaletteFamilyRoot}command-palette-list.vue`,
    render: () => listWith(() => "List"),
    assertServerMarkup(html) {
      assert.match(html, /role="listbox"/);
      assert.match(html, /aria-busy="true"/);
    },
    assertHydratedDom(host) {
      assert.equal(expectPart(host, "command-palette-list").id, "runtime-palette-list");
    },
  },
  {
    name: "command-palette-group",
    sourceFile: `${commandPaletteFamilyRoot}command-palette-group.vue`,
    render: () =>
      listWith(() =>
        h(CommandPaletteGroup, { heading: "Files" }, () =>
          h(CommandPaletteItem, { textValue: "Open" }),
        ),
      ),
    assertServerMarkup(html) {
      assert.match(html, /role="group"/);
      assert.match(html, /Files/);
    },
    assertHydratedDom(host) {
      assert.equal(expectPart(host, "command-palette-group").getAttribute("role"), "group");
    },
  },
  {
    name: "command-palette-item",
    sourceFile: `${commandPaletteFamilyRoot}command-palette-item.vue`,
    render: () =>
      listWith(() => h(CommandPaletteItem, { textValue: "Open", shortcut: "Control+O" })),
    assertServerMarkup(html) {
      assert.match(html, /role="option"/);
      assert.match(html, /aria-keyshortcuts="Control\+O"/);
    },
    assertHydratedDom(host) {
      assert.equal(expectPart(host, "command-palette-item").textContent?.trim(), "Open");
    },
  },
  {
    name: "command-palette-empty",
    sourceFile: `${commandPaletteFamilyRoot}command-palette-empty.vue`,
    render: () => listWith(() => h(CommandPaletteEmpty)),
    assertServerMarkup(html) {
      assert.match(html, /data-vize-ui="command-palette-empty"/);
    },
    assertHydratedDom(host) {
      assert.equal(expectPart(host, "command-palette-empty").hidden, true);
    },
  },
  {
    name: "command-palette-loading",
    sourceFile: `${commandPaletteFamilyRoot}command-palette-loading.vue`,
    render: () => listWith(() => h(CommandPaletteLoading)),
    assertServerMarkup(html) {
      assert.match(html, /role="progressbar"/);
    },
    assertHydratedDom(host) {
      assert.equal(expectPart(host, "command-palette-loading").hidden, false);
    },
  },
  {
    name: "command-palette-dialog",
    sourceFile: `${commandPaletteFamilyRoot}command-palette-dialog.vue`,
    render: () => h(CommandPaletteDialog, null, () => "Dialog"),
    assertServerMarkup(html) {
      assert.match(html, /data-vize-ui="command-palette-dialog"/);
      assert.match(html, /data-state="closed"/);
    },
    assertHydratedDom(host) {
      assert.equal(
        expectPart(host, "command-palette-dialog").getAttribute("data-shortcut"),
        "Mod+K",
      );
    },
  },
];
