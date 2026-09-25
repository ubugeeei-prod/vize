import assert from "node:assert/strict";

import { h } from "vue";

import {
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarInset,
  SidebarProvider,
  SidebarRail,
  SidebarRoot,
  SidebarTrigger,
} from "./sidebar.ts";
import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";

const sidebarFamilyRoot = "families/layout/sidebar/";

function sidebar(children: () => unknown): ReturnType<typeof h> {
  return h(SidebarProvider, { id: "runtime-sidebar", mobileQuery: null }, children);
}

function inRoot(children: () => unknown): ReturnType<typeof h> {
  return sidebar(() => h(SidebarRoot, null, children));
}

function part(name: string, render: () => ReturnType<typeof h>, tag = "div"): RuntimeFixture {
  return {
    name: `sidebar-${name}`,
    sourceFile: `${sidebarFamilyRoot}sidebar-${name}.vue`,
    render,
    assertServerMarkup(html) {
      assert.match(html, new RegExp(`data-vize-ui="sidebar-${name}"`));
    },
    assertHydratedDom(host) {
      const element = host.querySelector(`[data-vize-ui="sidebar-${name}"]`);
      assert.ok(element instanceof HTMLElement);
      assert.equal(element.tagName.toLowerCase(), tag);
    },
  };
}

export const sidebarRuntimeFixtures: readonly RuntimeFixture[] = [
  part("provider", () => sidebar(() => "Shell")),
  part("root", () => inRoot(() => "Navigation"), "aside"),
  part("trigger", () => sidebar(() => h(SidebarTrigger)), "button"),
  part("rail", () => inRoot(() => h(SidebarRail)), "button"),
  part("header", () => inRoot(() => h(SidebarHeader, null, () => "Header"))),
  part("content", () => inRoot(() => h(SidebarContent, null, () => "Content"))),
  part("footer", () => inRoot(() => h(SidebarFooter, null, () => "Footer"))),
  part("group", () => inRoot(() => h(SidebarGroup, null, () => "Group"))),
  part("group-label", () =>
    inRoot(() => h(SidebarGroup, null, () => h(SidebarGroupLabel, null, () => "Label"))),
  ),
  part("inset", () => sidebar(() => h(SidebarInset, null, () => "Main")), "main"),
];
