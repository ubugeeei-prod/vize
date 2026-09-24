import assert from "node:assert/strict";

import { afterEach, beforeEach, test, vi } from "vite-plus/test";
import { defineComponent, h, nextTick } from "vue";

import type { ToastStore, ToastViewportSlotState } from "./toast.ts";
import { createToastStore, useToast } from "./toast.ts";
import ToastAction from "./toast-action.vue";
import ToastClose from "./toast-close.vue";
import ToastDescription from "./toast-description.vue";
import ToastProvider from "./toast-provider.vue";
import ToastRoot from "./toast-root.vue";
import ToastTitle from "./toast-title.vue";
import ToastViewport from "./toast-viewport.vue";
import { mountInteraction } from "../../../testing/mount.ts";

beforeEach(() => {
  vi.useFakeTimers();
});

afterEach(() => {
  vi.useRealTimers();
  document.body.replaceChildren();
});

async function settle(): Promise<void> {
  await nextTick();
  await nextTick();
  await nextTick();
}

function mountToaster(
  providerProps: Record<string, unknown> = {},
  viewportProps: Record<string, unknown> = {},
  viewportSlot?: (state: ToastViewportSlotState) => unknown,
) {
  let store: ToastStore | null = null;
  const Probe = defineComponent({
    name: "ToastStoreProbe",
    setup() {
      store = useToast();
      return () => null;
    },
  });
  const handle = mountInteraction(ToastProvider, {
    props: providerProps,
    slots: {
      default: () => [
        h(
          ToastViewport,
          viewportProps,
          viewportSlot === undefined ? undefined : { default: viewportSlot },
        ),
        h(Probe),
      ],
    },
  });
  if (store === null) assert.fail("useToast must resolve inside ToastProvider");
  const region = () => {
    const element = handle.root().querySelector<HTMLElement>("[data-vize-ui='toast-viewport']");
    if (!element) assert.fail("viewport must render a region");
    return element;
  };
  const items = () => [...handle.root().querySelectorAll<HTMLLIElement>("[data-vize-ui='toast']")];
  const announcer = () => {
    const element = handle.root().querySelector<HTMLElement>("[data-vize-ui='live-region']");
    if (!element) assert.fail("viewport must embed a live region");
    return element;
  };
  return { handle, store: store as ToastStore, region, items, announcer };
}

test("renders a labelled region with list items and every default part", async () => {
  const { handle, store, region, items } = mountToaster();
  store.toast({
    title: "Draft saved",
    description: "Synced to cloud",
    action: { label: "Undo", altText: "Undo from the history panel" },
  });
  await settle();

  const section = region();
  assert.equal(section.tagName, "SECTION");
  assert.equal(section.getAttribute("aria-label"), "Notifications (F8)");
  assert.equal(section.getAttribute("tabindex"), "-1");
  assert.equal(section.getAttribute("data-vize-ui"), "toast-viewport");
  const [item] = items();
  assert.ok(item);
  assert.equal(item.parentElement?.tagName, "OL");
  assert.equal(item.getAttribute("role"), "status");
  assert.equal(item.getAttribute("aria-live"), "off");
  assert.equal(item.getAttribute("aria-atomic"), "true");
  assert.equal(item.getAttribute("tabindex"), "0");
  assert.equal(item.getAttribute("data-state"), "open");
  assert.equal(item.getAttribute("data-type"), "default");
  assert.equal(item.getAttribute("data-priority"), "normal");
  assert.equal(item.getAttribute("data-swipe-direction"), "right");
  assert.equal(item.querySelector("[data-vize-ui='toast-title']")?.textContent, "Draft saved");
  assert.equal(
    item.querySelector("[data-vize-ui='toast-description']")?.textContent,
    "Synced to cloud",
  );
  const action = item.querySelector<HTMLButtonElement>("[data-vize-ui='toast-action']");
  assert.equal(action?.textContent, "Undo");
  assert.equal(action?.getAttribute("data-alt-text"), "Undo from the history panel");
  const close = item.querySelector<HTMLButtonElement>("[data-vize-ui='toast-close']");
  assert.equal(close?.getAttribute("aria-label"), "Dismiss notification");

  handle.unmount();
});

test("visible toasts auto-dismiss after their duration and leave the DOM without exit motion", async () => {
  const { handle, store, items } = mountToaster({ duration: 1000 });
  store.toast("Copied");
  await settle();
  assert.equal(items().length, 1);

  vi.advanceTimersByTime(999);
  await settle();
  assert.equal(items().length, 1);
  vi.advanceTimersByTime(1);
  await settle();
  assert.equal(items().length, 0);
  assert.equal(store.toasts.value.length, 0);

  handle.unmount();
});

test("hovering or focusing the region and a hidden page pause timers", async () => {
  const { handle, store, region, items } = mountToaster({ duration: 1000 });
  store.toast("Paused");
  await settle();

  region().dispatchEvent(new PointerEvent("pointerenter"));
  vi.advanceTimersByTime(5000);
  await settle();
  assert.equal(items().length, 1);
  assert.equal(region().getAttribute("data-paused"), "true");
  region().dispatchEvent(new PointerEvent("pointerleave"));

  items()[0]?.focus();
  await settle();
  vi.advanceTimersByTime(5000);
  assert.equal(store.visibleToasts.value[0]?.open, true);
  items()[0]?.blur();
  await settle();

  Object.defineProperty(document, "visibilityState", { configurable: true, value: "hidden" });
  document.dispatchEvent(new Event("visibilitychange"));
  vi.advanceTimersByTime(5000);
  assert.equal(store.visibleToasts.value[0]?.open, true);
  Object.defineProperty(document, "visibilityState", { configurable: true, value: "visible" });
  document.dispatchEvent(new Event("visibilitychange"));
  assert.equal(store.paused.value, false);
  vi.advanceTimersByTime(1000);
  await settle();
  assert.equal(items().length, 0);

  handle.unmount();
});

test("the hotkey moves focus to the region and custom hotkeys rename it", async () => {
  const defaults = mountToaster();
  document.dispatchEvent(new KeyboardEvent("keydown", { key: "F8", code: "F8" }));
  await settle();
  assert.ok(document.activeElement === defaults.region());
  defaults.handle.unmount();

  const custom = mountToaster({ hotkey: ["altKey", "KeyT"], label: "Alerts" });
  const section = custom.region();
  assert.equal(section.getAttribute("aria-label"), "Alerts (Alt+T)");
  document.dispatchEvent(new KeyboardEvent("keydown", { code: "KeyT" }));
  assert.ok(document.activeElement !== section);
  const event = new KeyboardEvent("keydown", { altKey: true, code: "KeyT", cancelable: true });
  document.dispatchEvent(event);
  assert.ok(document.activeElement === section);
  assert.equal(event.defaultPrevented, true);
  custom.handle.unmount();
});

test("Escape and the close button dismiss dismissible toasts and return focus to the region", async () => {
  const { handle, store, region, items } = mountToaster();
  store.toast("First");
  store.toast({ title: "Pinned", dismissible: false });
  await settle();

  const [first, pinned] = items();
  assert.ok(first && pinned);
  first.focus();
  const escape = await handle.press(first, "Escape");
  assert.equal(escape.keydownPrevented, true);
  await settle();
  assert.equal(items().length, 1);
  assert.ok(document.activeElement === region());

  const pinnedEscape = await handle.press(pinned, "Escape");
  assert.equal(pinnedEscape.keydownPrevented, false);
  assert.equal(pinned.querySelector("[data-vize-ui='toast-close']"), null);
  assert.equal(items().length, 1);

  store.toast("Closable");
  await settle();
  const close = items()[1]?.querySelector<HTMLButtonElement>("[data-vize-ui='toast-close']");
  assert.ok(close);
  await handle.click(close);
  await settle();
  assert.equal(items().length, 1);
  assert.equal(store.get("toast-3"), undefined);

  handle.unmount();
});

test("swiping past the threshold dismisses while a short swipe cancels", async () => {
  const { handle, store, items } = mountToaster({ swipeThreshold: 40 });
  store.toast("Swipe me");
  await settle();
  const item = items()[0];
  assert.ok(item);

  item.dispatchEvent(
    new PointerEvent("pointerdown", { button: 0, clientX: 0, clientY: 0, pointerId: 1 }),
  );
  item.dispatchEvent(new PointerEvent("pointermove", { clientX: 10, clientY: 0, pointerId: 1 }));
  await nextTick();
  assert.equal(item.getAttribute("data-swipe"), "start");
  assert.equal(store.paused.value, true);
  item.dispatchEvent(new PointerEvent("pointermove", { clientX: 20, clientY: 3, pointerId: 1 }));
  await nextTick();
  assert.equal(item.getAttribute("data-swipe"), "move");
  assert.equal(item.style.getPropertyValue("--vize-toast-swipe-move-x"), "20px");
  item.dispatchEvent(new PointerEvent("pointerup", { clientX: 20, clientY: 0, pointerId: 1 }));
  await nextTick();
  assert.equal(item.getAttribute("data-swipe"), "cancel");
  assert.equal(store.paused.value, false);

  item.dispatchEvent(
    new PointerEvent("pointerdown", { button: 0, clientX: 0, clientY: 0, pointerId: 2 }),
  );
  item.dispatchEvent(new PointerEvent("pointermove", { clientX: -30, clientY: 0, pointerId: 2 }));
  await nextTick();
  assert.equal(item.getAttribute("data-swipe"), null);
  item.dispatchEvent(new PointerEvent("pointermove", { clientX: 60, clientY: 0, pointerId: 2 }));
  item.dispatchEvent(new PointerEvent("pointerup", { clientX: 60, clientY: 0, pointerId: 2 }));
  assert.equal(store.get("toast-1")?.dismissReason, "swipe");
  await settle();
  assert.equal(items().length, 0);

  handle.unmount();
});

test("vertical swipe directions publish the y offset", async () => {
  const { handle, store, items } = mountToaster({ swipeDirection: "up" });
  store.toast("Up");
  await settle();
  const item = items()[0];
  assert.ok(item);
  item.dispatchEvent(
    new PointerEvent("pointerdown", { button: 0, clientX: 0, clientY: 100, pointerId: 1 }),
  );
  item.dispatchEvent(new PointerEvent("pointermove", { clientX: 0, clientY: 80, pointerId: 1 }));
  await nextTick();
  assert.equal(item.style.getPropertyValue("--vize-toast-swipe-move-y"), "-20px");
  item.dispatchEvent(new PointerEvent("pointercancel", { clientX: 0, clientY: 0, pointerId: 1 }));
  await nextTick();
  assert.equal(item.getAttribute("data-swipe"), "cancel");
  assert.equal(store.get("toast-1")?.open, true);
  handle.unmount();
});

test("action activation runs the toast callback and dismisses unless prevented", async () => {
  const { handle, store, items } = mountToaster();
  const clicks: string[] = [];
  store.toast({
    title: "Archived",
    action: {
      label: "Undo",
      altText: "Undo in history",
      onClick: (event) => {
        clicks.push("undo");
        if (clicks.length === 1) event.preventDefault();
      },
    },
  });
  await settle();
  const action = () =>
    items()[0]?.querySelector<HTMLButtonElement>("[data-vize-ui='toast-action']") ?? null;

  const button = action();
  assert.ok(button);
  await handle.click(button);
  await settle();
  assert.equal(items().length, 1);
  await handle.click(button);
  await settle();
  assert.deepEqual(clicks, ["undo", "undo"]);
  assert.equal(items().length, 0);
  assert.equal(store.toasts.value.length, 0);

  handle.unmount();
});

test("limit stacks visible toasts and promotes queued ones by priority", async () => {
  const { handle, store, items } = mountToaster({ limit: 2 });
  store.toast("One");
  store.toast("Two");
  store.toast({ title: "Later", priority: "low" });
  store.toast({ title: "Urgent", priority: "high" });
  await settle();
  assert.deepEqual(
    items().map((item) => item.textContent?.replace("×", "")),
    ["One", "Two"],
  );

  store.dismiss("toast-1");
  await settle();
  const titles = items().map(
    (item) => item.querySelector("[data-vize-ui='toast-title']")?.textContent,
  );
  assert.deepEqual(titles, ["Two", "Urgent"]);
  assert.equal(items()[1]?.getAttribute("data-priority"), "high");

  handle.unmount();
});

test("new toasts are announced politely and errors or high priority assertively", async () => {
  const { handle, store, announcer } = mountToaster();
  store.toast({ title: "Saved", description: "All changes" });
  await settle();
  assert.equal(announcer().getAttribute("aria-live"), "polite");
  assert.equal(announcer().textContent?.trim(), "Saved. All changes");

  store.error("Upload failed");
  await settle();
  assert.equal(announcer().getAttribute("aria-live"), "assertive");
  assert.equal(announcer().textContent?.trim(), "Upload failed");

  store.update("toast-1", { title: "Saved again" });
  await settle();
  assert.equal(announcer().getAttribute("aria-live"), "polite");
  assert.equal(announcer().textContent?.trim(), "Saved again. All changes");

  handle.unmount();
});

test("promise toasts render loading then the settled phase in place", async () => {
  const { handle, store, items } = mountToaster();
  let resolve: (value: string) => void = () => undefined;
  const pending = new Promise<string>((done) => {
    resolve = done;
  });
  void store.promise(pending, {
    loading: "Uploading",
    success: (name) => `Uploaded ${name}`,
    error: "Upload failed",
  });
  await settle();
  assert.equal(items()[0]?.getAttribute("data-type"), "loading");
  vi.advanceTimersByTime(60_000);
  await settle();
  assert.equal(items().length, 1);

  resolve("report.pdf");
  await pending;
  await settle();
  assert.equal(items().length, 1);
  assert.equal(items()[0]?.getAttribute("data-type"), "success");
  assert.equal(
    items()[0]?.querySelector("[data-vize-ui='toast-title']")?.textContent,
    "Uploaded report.pdf",
  );

  handle.unmount();
});

test("custom slot rendering receives typed data and a dismiss callback", async () => {
  const store = createToastStore<{ readonly count: number }>();
  const handle = mountInteraction(ToastProvider, {
    props: { store },
    slots: {
      default: () =>
        h(ToastViewport, null, {
          default: ({ toast, index, count, dismiss }: ToastViewportSlotState) =>
            h(ToastRoot, { toast }, () => [
              h(ToastTitle, null, () => `custom ${index + 1}/${count}`),
              h(ToastDescription),
              h(ToastClose, { ariaLabel: "Close custom", onClick: () => undefined }, () => "x"),
              h(
                "button",
                { type: "button", "data-custom": "dismiss", onClick: dismiss },
                "dismiss",
              ),
            ]),
        }),
    },
  });
  store.toast({ title: "ignored", description: "details", data: { count: 2 } });
  await settle();
  assert.equal(store.get("toast-1")?.data?.count, 2);
  const root = handle.root();
  assert.equal(root.querySelector("[data-vize-ui='toast-title']")?.textContent, "custom 1/1");
  assert.equal(root.querySelector("[data-vize-ui='toast-description']")?.textContent, "details");
  const dismiss = root.querySelector<HTMLButtonElement>("[data-custom='dismiss']");
  assert.ok(dismiss);
  await handle.click(dismiss);
  await settle();
  assert.equal(root.querySelectorAll("[data-vize-ui='toast']").length, 0);
  handle.unmount();
});

test("ToastAction requires alt text through the root action contract", async () => {
  const { handle, store, items } = mountToaster({}, {}, ({ toast }) =>
    h(ToastRoot, { toast }, () => h(ToastAction, { altText: "Open the inbox" }, () => "Open")),
  );
  store.toast("Message");
  await settle();
  const action = items()[0]?.querySelector("[data-vize-ui='toast-action']");
  assert.equal(action?.textContent, "Open");
  assert.equal(action?.getAttribute("data-alt-text"), "Open the inbox");
  handle.unmount();
});

test("useToast and parts require a provider", () => {
  const Orphan = defineComponent({
    name: "ToastOrphan",
    setup() {
      useToast();
      return () => null;
    },
  });
  assert.throws(() => mountInteraction(Orphan), /VIZE_UI_CONTEXT_MISSING/);
  assert.throws(() => mountInteraction(ToastViewport), /VIZE_UI_CONTEXT_MISSING/);
  assert.throws(() => mountInteraction(ToastTitle), /VIZE_UI_CONTEXT_MISSING/);
});
