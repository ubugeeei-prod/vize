import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h, nextTick } from "vue";

import type { NotificationCenterRootExpose, NotificationStore } from "./notification-center.ts";
import { useNotificationCenter } from "./notification-center-context.ts";
import NotificationCenterEmpty from "./notification-center-empty.vue";
import NotificationCenterItem from "./notification-center-item.vue";
import NotificationCenterList from "./notification-center-list.vue";
import NotificationCenterRoot from "./notification-center-root.vue";
import { createNotificationStore } from "./notification-center-store.ts";
import NotificationCenterTrigger from "./notification-center-trigger.vue";
import { mountInteraction } from "../../../testing/mount.ts";

function seededStore(): NotificationStore {
  let time = 0;
  const store = createNotificationStore({ now: () => ++time });
  store.add({ id: "build", title: "Build passed", description: "main is green", type: "success" });
  store.add({ id: "review", title: "Review requested", group: "reviews" });
  store.add({ id: "deploy", title: "Deploy finished", read: true });
  return store;
}

function mountCenter(
  rootProps: Record<string, unknown> = {},
  listProps: Record<string, unknown> = {},
) {
  return mountInteraction(NotificationCenterRoot, {
    props: { id: "inbox", ...rootProps },
    slots: {
      default: () => [
        h(NotificationCenterTrigger, null, ({ unreadCount }: { readonly unreadCount: number }) =>
          h("span", { "data-badge": unreadCount }, "Bell"),
        ),
        h(NotificationCenterList, listProps, () => h(NotificationCenterEmpty)),
      ],
    },
  });
}

function feed(handle: ReturnType<typeof mountCenter>): HTMLElement {
  return handle.getByRole("feed");
}

function articles(handle: ReturnType<typeof mountCenter>): HTMLElement[] {
  return [...feed(handle).querySelectorAll<HTMLElement>("article")];
}

test("trigger discloses the feed and announces the unread count", async () => {
  const handle = mountCenter({ store: seededStore() });
  const trigger = handle.getByRole("button", { name: "Notifications, 2 unread" });
  const list = feed(handle);

  assert.equal(trigger.getAttribute("aria-expanded"), "false");
  assert.equal(trigger.getAttribute("aria-controls"), "inbox-list");
  assert.equal(trigger.getAttribute("data-unread-count"), "2");
  assert.equal(trigger.getAttribute("data-has-unread"), "true");
  assert.equal(trigger.querySelector("[data-badge]")?.getAttribute("data-badge"), "2");
  assert.equal(list.id, "inbox-list");
  assert.equal(list.hidden, true);

  await handle.click(trigger);
  assert.equal(trigger.getAttribute("aria-expanded"), "true");
  assert.equal(list.hidden, false);
  assert.deepEqual(handle.wrapper.emitted("update:open"), [[true]]);

  handle.unmount();
});

test("feed renders APG articles with position, size, labels, and read state", () => {
  const handle = mountCenter({ store: seededStore(), inline: true }, { busy: true });
  const list = feed(handle);
  const items = articles(handle);

  assert.equal(list.getAttribute("aria-label"), "Notifications");
  assert.equal(list.getAttribute("aria-busy"), "true");
  assert.equal(list.hidden, false);
  assert.equal(items.length, 3);
  const [newest] = items;
  assert.ok(newest);
  assert.equal(newest.getAttribute("aria-posinset"), "1");
  assert.equal(newest.getAttribute("aria-setsize"), "3");
  assert.equal(newest.getAttribute("tabindex"), "0");
  assert.equal(newest.getAttribute("data-read"), "read");
  const build = items[2];
  assert.ok(build);
  assert.equal(build.id, "inbox-item-build");
  assert.equal(build.getAttribute("aria-labelledby"), "inbox-item-build-title");
  assert.equal(build.getAttribute("aria-describedby"), "inbox-item-build-description");
  assert.equal(document.getElementById("inbox-item-build-title")?.textContent, "Build passed");
  assert.equal(build.getAttribute("data-type"), "success");
  assert.equal(items[1]?.getAttribute("data-group"), "reviews");
  assert.equal(
    handle
      .root()
      .querySelector('[data-vize-ui="notification-center-empty"]')
      ?.hasAttribute("hidden"),
    true,
  );
  assert.equal(
    handle.getByRole("button", { name: "Notifications, 2 unread" }).hasAttribute("aria-expanded"),
    false,
    "inline centers are not disclosures",
  );

  handle.unmount();
});

test("PageDown and PageUp move focus between articles", async () => {
  const handle = mountCenter({ store: seededStore(), inline: true });
  const items = articles(handle);
  const [first, second, third] = items;
  assert.ok(first && second && third);

  first.focus();
  const down = new KeyboardEvent("keydown", { key: "PageDown", bubbles: true, cancelable: true });
  first.dispatchEvent(down);
  assert.equal(down.defaultPrevented, true);
  assert.ok(document.activeElement === second);
  second.dispatchEvent(new KeyboardEvent("keydown", { key: "PageDown", bubbles: true }));
  assert.ok(document.activeElement === third);
  const edge = new KeyboardEvent("keydown", { key: "PageDown", bubbles: true, cancelable: true });
  third.dispatchEvent(edge);
  assert.equal(edge.defaultPrevented, false, "the last article keeps focus");
  third.dispatchEvent(new KeyboardEvent("keydown", { key: "PageUp", bubbles: true }));
  assert.ok(document.activeElement === second);
  await nextTick();

  handle.unmount();
});

test("activating an article marks it read unless opted out", async () => {
  const store = seededStore();
  const handle = mountCenter({ store, inline: true });
  const build = articles(handle)[2];
  assert.ok(build);

  build.focus();
  build.dispatchEvent(
    new KeyboardEvent("keydown", { key: "Enter", bubbles: true, cancelable: true }),
  );
  await nextTick();
  assert.equal(store.get("build")?.read, true);
  assert.equal(build.getAttribute("data-read"), "read");
  store.markUnread("build");
  await nextTick();
  build.click();
  await nextTick();
  assert.equal(store.get("build")?.read, true);
  handle.unmount();

  const optOut = seededStore();
  const passive = mountCenter({ store: optOut, inline: true }, { markReadOnActivate: false });
  articles(passive)[2]?.click();
  await nextTick();
  assert.equal(optOut.get("build")?.read, false);
  passive.unmount();
});

test("Escape closes a disclosure feed and returns focus to the trigger", async () => {
  const handle = mountCenter({ store: seededStore(), defaultOpen: true });
  const trigger = handle.getByRole("button", { name: /Notifications/ });
  const first = articles(handle)[0];
  assert.ok(first);

  first.focus();
  first.dispatchEvent(
    new KeyboardEvent("keydown", { key: "Escape", bubbles: true, cancelable: true }),
  );
  await nextTick();
  assert.equal(feed(handle).hidden, true);
  assert.ok(document.activeElement === trigger);

  handle.unmount();
});

test("custom item slots receive typed actions and ids", async () => {
  const store = createNotificationStore<{ readonly href: string }>({ now: () => 1 });
  store.add({ id: "pr", title: "PR merged", data: { href: "/pr/1" } });
  const handle = mountInteraction(NotificationCenterRoot, {
    props: { store, inline: true, id: "custom" },
    slots: {
      default: () =>
        h(NotificationCenterList, null, {
          item: ({
            notification,
            position,
            setSize,
          }: {
            readonly notification: ReturnType<typeof store.get> & object;
            readonly position: number;
            readonly setSize: number;
          }) =>
            h(
              NotificationCenterItem,
              { notification, position, setSize },
              {
                default: (slot: { readonly titleId: string; readonly archive: () => boolean }) => [
                  h("a", { id: slot.titleId, href: notification.data?.href }, notification.title),
                  h("button", { type: "button", onClick: slot.archive }, "Archive"),
                ],
              },
            ),
        }),
    },
  });

  const link = handle.root().querySelector("a");
  assert.equal(link?.getAttribute("href"), "/pr/1");
  assert.equal(link?.id, "custom-item-pr-title");
  await handle.click(handle.getByRole("button", { name: "Archive" }));
  assert.equal(store.archived.value.length, 1);
  assert.equal(store.get("pr")?.read, false, "inner controls do not activate the article");
  assert.equal(handle.root().querySelectorAll("article").length, 0);

  handle.unmount();
});

test("root owns a store, exposes it, and useNotificationCenter reads it", async () => {
  let root: NotificationCenterRootExpose | null = null;
  let injected: NotificationStore | null = null;
  const Reader = defineComponent({
    name: "NotificationCenterReader",
    setup() {
      injected = useNotificationCenter();
      return () => null;
    },
  });
  const Probe = defineComponent({
    name: "NotificationCenterProbe",
    setup: () => () =>
      h(
        NotificationCenterRoot,
        {
          initial: [{ id: "hello", title: "Hello" }],
          now: () => 5,
          ref: (value) => {
            root = value as NotificationCenterRootExpose | null;
          },
        },
        () => [h(Reader), h(NotificationCenterList), h(NotificationCenterEmpty)],
      ),
  });
  const handle = mountInteraction(Probe);
  if (root === null || injected === null) assert.fail("store must be exposed and injectable");
  const exposed: NotificationCenterRootExpose = root;
  const store: NotificationStore = injected;

  assert.ok(exposed.store === store);
  assert.equal(exposed.count, 1);
  assert.equal(exposed.unreadCount, 1);
  assert.equal(store.get("hello")?.createdAt, 5);
  assert.equal(exposed.setOpen(true), true);
  await nextTick();
  assert.equal(exposed.state, "open");
  store.remove("hello");
  await nextTick();
  const empty = handle.root().querySelector('[data-vize-ui="notification-center-empty"]');
  assert.equal(empty?.hasAttribute("hidden"), false);
  assert.equal(empty?.textContent, "No notifications");

  handle.unmount();
});

test("parts and the composable require a NotificationCenterRoot", () => {
  const Orphan = defineComponent({
    name: "NotificationCenterOrphan",
    setup() {
      useNotificationCenter();
      return () => null;
    },
  });
  assert.throws(() => mountInteraction(Orphan), /VIZE_UI_CONTEXT_MISSING/);
  assert.throws(() => mountInteraction(NotificationCenterTrigger), /VIZE_UI_CONTEXT_MISSING/);
  assert.throws(() => mountInteraction(NotificationCenterList), /VIZE_UI_CONTEXT_MISSING/);
  assert.throws(() => mountInteraction(NotificationCenterEmpty), /VIZE_UI_CONTEXT_MISSING/);
});
