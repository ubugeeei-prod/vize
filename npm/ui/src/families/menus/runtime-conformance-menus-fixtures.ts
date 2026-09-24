import assert from "node:assert/strict";

import { h } from "vue";
import type { VNode } from "vue";

import type { RuntimeFixture } from "../../conformance/runtime-conformance-fixtures.ts";
import ContextMenuRoot from "./context-menu/context-menu-root.vue";
import ContextMenuTrigger from "./context-menu/context-menu-trigger.vue";
import DropdownMenuRoot from "./dropdown-menu/dropdown-menu-root.vue";
import DropdownMenuTrigger from "./dropdown-menu/dropdown-menu-trigger.vue";
import MenuArrow from "./menu/menu-arrow.vue";
import MenuCheckboxItem from "./menu/menu-checkbox-item.vue";
import MenuContent from "./menu/menu-content.vue";
import MenuGroup from "./menu/menu-group.vue";
import MenuItem from "./menu/menu-item.vue";
import MenuItemIndicator from "./menu/menu-item-indicator.vue";
import MenuLabel from "./menu/menu-label.vue";
import MenuRadioGroup from "./menu/menu-radio-group.vue";
import MenuRadioItem from "./menu/menu-radio-item.vue";
import MenuRoot from "./menu/menu-root.vue";
import MenuSeparator from "./menu/menu-separator.vue";
import MenuSub from "./menu/menu-sub.vue";
import MenuSubContent from "./menu/menu-sub-content.vue";
import MenuSubTrigger from "./menu/menu-sub-trigger.vue";
import MenuTrigger from "./menu/menu-trigger.vue";
import MenubarMenu from "./menubar/menubar-menu.vue";
import MenubarRoot from "./menubar/menubar-root.vue";
import MenubarTrigger from "./menubar/menubar-trigger.vue";

type MenuPart = "check" | "group" | "radio" | "sub";

function menuItems(parts: readonly MenuPart[]): VNode[] {
  const items: VNode[] = [h(MenuItem, null, () => "New"), h(MenuSeparator), h(MenuArrow)];
  if (parts.includes("group")) {
    items.push(
      h(MenuGroup, null, () => [
        h(MenuLabel, { id: "runtime-menu-label" }, () => "File"),
        h(MenuItem, null, () => "Open"),
      ]),
    );
  }
  if (parts.includes("check")) {
    items.push(
      h(MenuCheckboxItem, { defaultValue: true }, () => [
        h(MenuItemIndicator, null, () => "✓"),
        "Wrap",
      ]),
    );
  }
  if (parts.includes("radio")) {
    items.push(
      h(MenuRadioGroup<string>, { defaultValue: "a" }, () => [
        h(MenuRadioItem<string>, { value: "a" }, () => "A"),
      ]),
    );
  }
  if (parts.includes("sub")) {
    items.push(
      h(MenuSub, { id: "runtime-menu-sub", defaultOpen: true }, () => [
        h(MenuSubTrigger, null, () => "More"),
        h(MenuSubContent, { portalDisabled: true }, () => h(MenuItem, null, () => "Nested")),
      ]),
    );
  }
  return items;
}

function menuTree(parts: readonly MenuPart[]): () => VNode {
  return () =>
    h("div", [
      h(MenuRoot, { id: "runtime-menu", defaultOpen: true }, () => [
        h(MenuTrigger, null, () => "Actions"),
        h(MenuContent, { portalDisabled: true }, () => menuItems(parts)),
      ]),
    ]);
}

function query(host: HTMLElement, name: string): HTMLElement {
  const element = host.querySelector(`[data-vize-ui="${name}"]`);
  assert.ok(element instanceof HTMLElement, `${name} must hydrate`);
  return element;
}

function menuFixture(
  file: string,
  name: string,
  parts: readonly MenuPart[],
  assertServer: (html: string) => void,
  assertDom: (element: HTMLElement) => void,
): RuntimeFixture {
  return {
    name,
    sourceFile: `families/menus/menu/${file}`,
    render: menuTree(parts),
    assertServerMarkup(html) {
      assert.match(html, new RegExp(`data-vize-ui="${name}"`));
      assertServer(html);
    },
    assertHydratedDom(host) {
      assertDom(query(host, name));
    },
  };
}

export const menusRuntimeFixtures: readonly RuntimeFixture[] = [
  {
    name: "context-menu-root",
    sourceFile: "families/menus/context-menu/context-menu-root.vue",
    render: () =>
      h("div", [
        h(ContextMenuRoot, { id: "runtime-context", defaultOpen: true }, () => [
          h(ContextMenuTrigger, null, () => "Canvas"),
          h(MenuContent, { portalDisabled: true, ariaLabel: "Canvas" }, () =>
            h(MenuItem, null, () => "Paste"),
          ),
        ]),
      ]),
    assertServerMarkup(html) {
      assert.match(html, /data-menu-kind="context-menu"/);
      assert.match(html, /aria-label="Canvas"/);
    },
    assertHydratedDom(host) {
      assert.equal(query(host, "menu-content").getAttribute("data-menu-kind"), "context-menu");
    },
  },
  {
    name: "context-menu-trigger",
    sourceFile: "families/menus/context-menu/context-menu-trigger.vue",
    render: () =>
      h("div", [
        h(ContextMenuRoot, { id: "runtime-context-trigger" }, () =>
          h(ContextMenuTrigger, null, () => "Canvas"),
        ),
      ]),
    assertServerMarkup(html) {
      assert.match(html, /data-vize-ui="context-menu-trigger"/);
      assert.match(html, /data-state="closed"/);
    },
    assertHydratedDom(host) {
      assert.equal(query(host, "context-menu-trigger").textContent, "Canvas");
    },
  },
  {
    name: "dropdown-menu-root",
    sourceFile: "families/menus/dropdown-menu/dropdown-menu-root.vue",
    render: () =>
      h("div", [
        h(DropdownMenuRoot, { id: "runtime-dropdown", defaultOpen: true }, () => [
          h(DropdownMenuTrigger, null, () => "File"),
          h(MenuContent, { portalDisabled: true }, () => h(MenuItem, null, () => "New")),
        ]),
      ]),
    assertServerMarkup(html) {
      assert.match(html, /data-menu-kind="dropdown-menu"/);
      assert.match(html, /aria-labelledby="runtime-dropdown-trigger"/);
    },
    assertHydratedDom(host) {
      assert.equal(query(host, "menu-content").id, "runtime-dropdown-content");
    },
  },
  {
    name: "dropdown-menu-trigger",
    sourceFile: "families/menus/dropdown-menu/dropdown-menu-trigger.vue",
    render: () =>
      h("div", [
        h(DropdownMenuRoot, { id: "runtime-dropdown-trigger" }, () =>
          h(DropdownMenuTrigger, null, () => "File"),
        ),
      ]),
    assertServerMarkup(html) {
      assert.match(html, /aria-haspopup="menu"/);
      assert.match(html, /aria-expanded="false"/);
    },
    assertHydratedDom(host) {
      assert.equal(query(host, "dropdown-menu-trigger").id, "runtime-dropdown-trigger-trigger");
    },
  },
  menuFixture(
    "menu-arrow.vue",
    "menu-arrow",
    [],
    (html) => assert.match(html, /aria-hidden="true"/),
    (arrow) => assert.equal(arrow.getAttribute("data-state"), "open"),
  ),
  menuFixture(
    "menu-checkbox-item.vue",
    "menu-checkbox-item",
    ["check"],
    (html) => assert.match(html, /role="menuitemcheckbox"[^>]*aria-checked="true"/),
    (item) => assert.equal(item.getAttribute("data-state"), "checked"),
  ),
  menuFixture(
    "menu-content.vue",
    "menu-content",
    [],
    (html) => assert.match(html, /id="runtime-menu-content"[^>]*role="menu"|role="menu"/),
    (content) => assert.equal(content.id, "runtime-menu-content"),
  ),
  menuFixture(
    "menu-group.vue",
    "menu-group",
    ["group"],
    (html) => assert.match(html, /role="group"/),
    (group) => assert.equal(group.getAttribute("role"), "group"),
  ),
  menuFixture(
    "menu-item.vue",
    "menu-item",
    [],
    (html) => assert.match(html, /role="menuitem"/),
    (item) => assert.equal(item.getAttribute("tabindex"), "-1"),
  ),
  menuFixture(
    "menu-item-indicator.vue",
    "menu-item-indicator",
    ["check"],
    (html) => assert.match(html, /data-vize-ui="menu-item-indicator"[^>]*data-state="checked"/),
    (indicator) => assert.equal(indicator.textContent, "✓"),
  ),
  menuFixture(
    "menu-label.vue",
    "menu-label",
    ["group"],
    (html) => assert.match(html, /id="runtime-menu-label"/),
    (label) => assert.equal(label.id, "runtime-menu-label"),
  ),
  menuFixture(
    "menu-radio-group.vue",
    "menu-radio-group",
    ["radio"],
    (html) => assert.match(html, /data-vize-ui="menu-radio-group"/),
    (group) => assert.equal(group.getAttribute("role"), "group"),
  ),
  menuFixture(
    "menu-radio-item.vue",
    "menu-radio-item",
    ["radio"],
    (html) => assert.match(html, /role="menuitemradio"[^>]*aria-checked="true"/),
    (item) => assert.equal(item.getAttribute("aria-checked"), "true"),
  ),
  menuFixture(
    "menu-root.vue",
    "menu-trigger",
    [],
    (html) => assert.match(html, /id="runtime-menu-trigger"/),
    (trigger) => assert.equal(trigger.getAttribute("aria-controls"), "runtime-menu-content"),
  ),
  menuFixture(
    "menu-separator.vue",
    "menu-separator",
    [],
    (html) => assert.match(html, /role="separator"/),
    (separator) => assert.equal(separator.getAttribute("aria-orientation"), "horizontal"),
  ),
  menuFixture(
    "menu-sub.vue",
    "menu-sub-trigger",
    ["sub"],
    (html) => assert.match(html, /id="runtime-menu-sub-trigger"/),
    (trigger) => assert.equal(trigger.getAttribute("aria-expanded"), "true"),
  ),
  menuFixture(
    "menu-sub-content.vue",
    "menu-sub-content",
    ["sub"],
    (html) => assert.match(html, /id="runtime-menu-sub-content"/),
    (content) => assert.equal(content.getAttribute("aria-labelledby"), "runtime-menu-sub-trigger"),
  ),
  menuFixture(
    "menu-sub-trigger.vue",
    "menu-sub-trigger",
    ["sub"],
    (html) => assert.match(html, /aria-haspopup="menu"/),
    (trigger) => assert.equal(trigger.getAttribute("role"), "menuitem"),
  ),
  menuFixture(
    "menu-trigger.vue",
    "menu-trigger",
    [],
    (html) => assert.match(html, /aria-expanded="true"/),
    (trigger) => assert.equal(trigger.getAttribute("aria-haspopup"), "menu"),
  ),
  ...["menubar-menu.vue", "menubar-root.vue", "menubar-trigger.vue"].map(
    (file): RuntimeFixture => ({
      name: file.replace(".vue", ""),
      sourceFile: `families/menus/menubar/${file}`,
      render: () =>
        h("div", [
          h(MenubarRoot, { id: "runtime-menubar", ariaLabel: "App", defaultValue: "file" }, () => [
            h(MenubarMenu, { value: "file" }, () => [
              h(MenubarTrigger, null, () => "File"),
              h(MenuContent, { portalDisabled: true }, () => h(MenuItem, null, () => "New")),
            ]),
            h(MenubarMenu, { value: "edit" }, () => h(MenubarTrigger, null, () => "Edit")),
          ]),
        ]),
      assertServerMarkup(html) {
        assert.match(html, /role="menubar"/);
        assert.match(html, /data-menu-kind="menubar"/);
        assert.equal(html.match(/tabindex="0"/g)?.length, 1);
      },
      assertHydratedDom(host) {
        assert.equal(query(host, "menubar").id, "runtime-menubar");
        assert.equal(host.querySelectorAll('[data-vize-ui="menubar-trigger"]').length, 2);
      },
    }),
  ),
];
