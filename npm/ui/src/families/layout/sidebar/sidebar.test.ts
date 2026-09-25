import assert from "node:assert/strict";

import { afterEach, test } from "vite-plus/test";
import { defineComponent, h, nextTick } from "vue";

import type { SidebarProviderExpose, SidebarStorage } from "./sidebar.ts";
import SidebarContent from "./sidebar-content.vue";
import SidebarFooter from "./sidebar-footer.vue";
import SidebarGroup from "./sidebar-group.vue";
import SidebarGroupLabel from "./sidebar-group-label.vue";
import SidebarHeader from "./sidebar-header.vue";
import SidebarInset from "./sidebar-inset.vue";
import SidebarProvider from "./sidebar-provider.vue";
import SidebarRail from "./sidebar-rail.vue";
import SidebarRoot from "./sidebar-root.vue";
import SidebarTrigger from "./sidebar-trigger.vue";
import { detectShortcutPlatform } from "../../interaction/shortcut/shortcut.ts";
import { mountInteraction } from "../../../testing/mount.ts";

const originalMatchMedia = window.matchMedia;

afterEach(() => {
  window.matchMedia = originalMatchMedia;
});

interface FakeMedia {
  matches: boolean;
  readonly listeners: Set<(event: MediaQueryListEvent) => void>;
}

function installMatchMedia(matches: boolean): FakeMedia {
  const media: FakeMedia = { matches, listeners: new Set() };
  window.matchMedia = (query: string) => {
    const list = new EventTarget();
    return Object.assign(list, {
      get matches() {
        return media.matches;
      },
      media: query,
      onchange: null,
      addEventListener: (_type: string, listener: (event: MediaQueryListEvent) => void) => {
        media.listeners.add(listener);
      },
      removeEventListener: (_type: string, listener: (event: MediaQueryListEvent) => void) => {
        media.listeners.delete(listener);
      },
      addListener: () => undefined,
      removeListener: () => undefined,
      dispatchEvent: () => true,
    }) as unknown as MediaQueryList;
  };
  return media;
}

function changeMedia(media: FakeMedia, matches: boolean): void {
  media.matches = matches;
  for (const listener of media.listeners) {
    listener({ matches } as MediaQueryListEvent);
  }
}

function layout() {
  return [
    h(SidebarRoot, { portalDisabled: true }, () => [
      h(SidebarHeader, null, () => "Workspace"),
      h(SidebarContent, null, () =>
        h(SidebarGroup, { id: "nav-group" }, () => [
          h(SidebarGroupLabel, null, () => "Navigate"),
          h("a", { href: "/inbox" }, "Inbox"),
        ]),
      ),
      h(SidebarFooter, null, () => "Account"),
      h(SidebarRail),
    ]),
    h(SidebarInset, null, () => [h(SidebarTrigger), h("p", "Main")]),
  ];
}

function mountSidebar(props: Record<string, unknown> = {}) {
  return mountInteraction(SidebarProvider, {
    props: { id: "app", mobileQuery: null, ...props },
    slots: { default: () => layout() },
  });
}

function aside(handle: ReturnType<typeof mountSidebar>): HTMLElement {
  const element = handle.root().querySelector('[data-vize-ui="sidebar-root"]');
  assert.ok(element instanceof HTMLElement);
  return element;
}

function trigger(handle: ReturnType<typeof mountSidebar>): HTMLButtonElement {
  const element = handle.root().querySelector('[data-vize-ui="sidebar-trigger"]');
  assert.ok(element instanceof HTMLButtonElement);
  return element;
}

test("renders a labelled complementary landmark, sections, and CSS width variables", () => {
  const handle = mountSidebar({ width: "18rem", widthIcon: "4rem" });
  const root = handle.root();
  const sidebar = aside(handle);
  const group = root.querySelector('[data-vize-ui="sidebar-group"]');
  const inset = root.querySelector('[data-vize-ui="sidebar-inset"]');

  assert.equal(root.getAttribute("data-vize-ui"), "sidebar-provider");
  assert.equal(root.style.getPropertyValue("--vize-sidebar-width"), "18rem");
  assert.equal(root.style.getPropertyValue("--vize-sidebar-width-icon"), "4rem");
  assert.equal(sidebar.tagName, "ASIDE");
  assert.equal(sidebar.id, "app-panel");
  assert.equal(sidebar.getAttribute("aria-label"), "Sidebar");
  assert.equal(sidebar.getAttribute("data-state"), "expanded");
  assert.equal(sidebar.getAttribute("data-side"), "left");
  assert.equal(sidebar.getAttribute("data-variant"), "sidebar");
  assert.equal(group?.getAttribute("role"), "group");
  assert.equal(group?.getAttribute("aria-labelledby"), "nav-group-label");
  assert.equal(root.querySelector("#nav-group-label")?.textContent, "Navigate");
  assert.equal(inset?.tagName, "MAIN");
  assert.ok(root.querySelector('[data-vize-ui="sidebar-header"]'));
  assert.ok(root.querySelector('[data-vize-ui="sidebar-footer"]'));
  assert.ok(root.querySelector('[data-vize-ui="sidebar-content"]'));

  handle.unmount();
});

test("trigger toggles an off-canvas sidebar and makes the collapsed landmark inert", async () => {
  const handle = mountSidebar();
  const button = trigger(handle);

  assert.equal(button.getAttribute("aria-expanded"), "true");
  assert.equal(button.getAttribute("aria-controls"), "app-panel");
  await handle.click(button);
  assert.equal(button.getAttribute("aria-expanded"), "false");
  assert.equal(aside(handle).getAttribute("data-state"), "collapsed");
  assert.equal(aside(handle).getAttribute("data-collapsible"), "offcanvas");
  assert.equal(aside(handle).hasAttribute("inert"), true);
  assert.deepEqual(handle.wrapper.emitted("update:open"), [[false]]);
  const change = handle.wrapper.emitted("open-change")?.[0];
  assert.deepEqual(change?.slice(0, 2), [false, true]);

  await handle.click(button);
  assert.equal(aside(handle).hasAttribute("inert"), false);

  handle.unmount();
});

test("icon collapse keeps the rail interactive and the rail toggles the sidebar", async () => {
  const handle = mountSidebar({ collapsible: "icon" });
  const rail = handle.root().querySelector('[data-vize-ui="sidebar-rail"]');
  assert.ok(rail instanceof HTMLButtonElement);
  assert.equal(rail.tabIndex, -1);

  await handle.click(rail);
  assert.equal(aside(handle).getAttribute("data-collapsible"), "icon");
  assert.equal(aside(handle).hasAttribute("inert"), false);
  assert.equal(rail.getAttribute("aria-expanded"), "false");

  handle.unmount();
});

test("collapsible none keeps the sidebar expanded and disables toggles", async () => {
  const handle = mountSidebar({ collapsible: "none", defaultOpen: false });
  const button = trigger(handle);

  assert.equal(aside(handle).getAttribute("data-state"), "expanded");
  assert.equal(button.disabled, true);
  await handle.click(button);
  assert.equal(handle.wrapper.emitted("update:open"), undefined);

  handle.unmount();
});

test("controlled open waits for the parent and click is preventable", async () => {
  const handle = mountSidebar({ open: true });
  const button = trigger(handle);

  await handle.click(button);
  assert.equal(button.getAttribute("aria-expanded"), "true");
  assert.deepEqual(handle.wrapper.emitted("update:open"), [[false]]);
  await handle.wrapper.setProps({ open: false });
  assert.equal(button.getAttribute("aria-expanded"), "false");
  handle.unmount();

  const prevented = mountInteraction(SidebarProvider, {
    props: { mobileQuery: null },
    slots: {
      default: () => [
        h(SidebarRoot, null, () => "Nav"),
        h(SidebarTrigger, { onClick: (event: MouseEvent) => event.preventDefault() }),
      ],
    },
  });
  const preventedButton = prevented.root().querySelector("button");
  assert.ok(preventedButton);
  await prevented.click(preventedButton);
  assert.equal(prevented.wrapper.emitted("update:open"), undefined);
  prevented.unmount();
});

test("persisted state is restored after mount and written on change", async () => {
  const writes: string[] = [];
  const storage: SidebarStorage = {
    get: (key) => (key === "nav-state" ? "false" : null),
    set: (key, value) => writes.push(`${key}=${value}`),
  };
  const handle = mountSidebar({ storage, storageKey: "nav-state" });
  await nextTick();

  assert.equal(aside(handle).getAttribute("data-state"), "collapsed");
  await handle.click(trigger(handle));
  assert.equal(aside(handle).getAttribute("data-state"), "expanded");
  assert.deepEqual(writes, ["nav-state=false", "nav-state=true"]);

  handle.unmount();
});

test("the Mod+B shortcut toggles the sidebar and can be disabled", async () => {
  const platform = detectShortcutPlatform();
  const modifier = platform === "apple" ? { metaKey: true } : { ctrlKey: true };
  const handle = mountSidebar();
  await nextTick();

  const event = new KeyboardEvent("keydown", {
    key: "b",
    bubbles: true,
    cancelable: true,
    ...modifier,
  });
  document.body.dispatchEvent(event);
  await nextTick();
  assert.equal(event.defaultPrevented, true);
  assert.equal(aside(handle).getAttribute("data-state"), "collapsed");
  handle.unmount();

  const disabled = mountSidebar({ keyboardShortcut: null });
  await nextTick();
  document.body.dispatchEvent(
    new KeyboardEvent("keydown", { key: "b", bubbles: true, cancelable: true, ...modifier }),
  );
  await nextTick();
  assert.equal(aside(disabled).getAttribute("data-state"), "expanded");
  disabled.unmount();
});

test("mobile query renders the sidebar as a modal sheet toggled by the trigger", async () => {
  const media = installMatchMedia(true);
  const handle = mountSidebar({ mobileQuery: "(max-width: 767px)" });
  await nextTick();
  const button = trigger(handle);

  assert.equal(handle.root().getAttribute("data-mobile"), "true");
  assert.equal(document.querySelector('[data-vize-ui="dialog-content"]'), null);
  assert.equal(button.getAttribute("aria-expanded"), "false");
  await handle.click(button);
  await nextTick();
  const sheet = document.querySelector('[data-vize-ui="dialog-content"]');
  assert.ok(sheet instanceof HTMLElement);
  assert.equal(sheet.getAttribute("aria-modal"), "true");
  assert.equal(sheet.getAttribute("aria-label"), "Sidebar");
  assert.equal(
    sheet.querySelector('[data-vize-ui="sidebar-root"]')?.getAttribute("data-mobile"),
    "true",
  );
  assert.equal(button.getAttribute("aria-expanded"), "true");
  assert.deepEqual(handle.wrapper.emitted("update:openMobile"), [[true]]);
  assert.equal(handle.wrapper.emitted("update:open"), undefined);

  sheet.dispatchEvent(
    new KeyboardEvent("keydown", { key: "Escape", bubbles: true, cancelable: true }),
  );
  await nextTick();
  assert.deepEqual(handle.wrapper.emitted("update:openMobile"), [[true], [false]]);

  changeMedia(media, false);
  await nextTick();
  assert.equal(aside(handle).tagName, "ASIDE");
  assert.equal(handle.root().getAttribute("data-mobile"), null);

  handle.unmount();
  assert.equal(media.listeners.size, 0, "media listeners are released on unmount");
});

test("provider exposes typed state and programmatic controls", async () => {
  let exposed: SidebarProviderExpose | null = null;
  const Probe = defineComponent({
    name: "SidebarExposeProbe",
    setup: () => () =>
      h(
        SidebarProvider,
        {
          mobileQuery: null,
          side: "right",
          variant: "floating",
          ref: (value) => {
            exposed = value as SidebarProviderExpose | null;
          },
        },
        () => h(SidebarRoot, null, () => "Nav"),
      ),
  });
  const handle = mountInteraction(Probe);
  if (exposed === null) assert.fail("SidebarProvider must expose its API");
  const provider: SidebarProviderExpose = exposed;

  assert.equal(provider.state, "expanded");
  assert.equal(provider.side, "right");
  assert.equal(provider.variant, "floating");
  assert.equal(provider.isMobile, false);
  assert.match(provider.sidebarId, /-sidebar-panel$/);
  assert.equal(provider.toggle(), true);
  await nextTick();
  assert.equal(provider.open, false);
  assert.equal(provider.setOpen(false), false);
  assert.equal(provider.setOpenMobile(true), true);
  await nextTick();
  assert.equal(provider.openMobile, true);

  handle.unmount();
});

test("sidebar parts require a provider", () => {
  for (const part of [
    SidebarRoot,
    SidebarTrigger,
    SidebarRail,
    SidebarContent,
    SidebarHeader,
    SidebarFooter,
    SidebarGroup,
    SidebarInset,
  ]) {
    assert.throws(() => mountInteraction(part), /VIZE_UI_CONTEXT_MISSING/);
  }
  assert.throws(
    () =>
      mountInteraction(SidebarProvider, {
        props: { mobileQuery: null },
        slots: { default: () => h(SidebarGroupLabel) },
      }),
    /SidebarGroup/,
  );
});
