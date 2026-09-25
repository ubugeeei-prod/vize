import assert from "node:assert/strict";

import { h } from "vue";

import {
  NotificationCenterEmpty,
  NotificationCenterItem,
  NotificationCenterList,
  NotificationCenterRoot,
  NotificationCenterTrigger,
} from "./notification-center.ts";
import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";

const notificationCenterFamilyRoot = "families/feedback/notification-center/";

function center(children: () => unknown): ReturnType<typeof h> {
  return h(
    NotificationCenterRoot,
    {
      id: "runtime-inbox",
      inline: true,
      now: () => 0,
      initial: [{ id: "build", title: "Build passed", description: "main is green" }],
    },
    children,
  );
}

function part(host: HTMLElement, name: string): HTMLElement {
  const element = host.querySelector(`[data-vize-ui="${name}"]`);
  assert.ok(element instanceof HTMLElement, name);
  return element;
}

export const notificationCenterRuntimeFixtures: readonly RuntimeFixture[] = [
  {
    name: "notification-center-root",
    sourceFile: `${notificationCenterFamilyRoot}notification-center-root.vue`,
    render: () => center(() => "Inbox"),
    assertServerMarkup(html) {
      assert.match(html, /id="runtime-inbox"/);
      assert.match(html, /data-unread-count="1"/);
    },
    assertHydratedDom(host) {
      part(host, "notification-center-root");
    },
  },
  {
    name: "notification-center-trigger",
    sourceFile: `${notificationCenterFamilyRoot}notification-center-trigger.vue`,
    render: () => center(() => h(NotificationCenterTrigger, null, () => "Bell")),
    assertServerMarkup(html) {
      assert.match(html, /aria-label="Notifications, 1 unread"/);
    },
    assertHydratedDom(host) {
      assert.ok(part(host, "notification-center-trigger") instanceof HTMLButtonElement);
    },
  },
  {
    name: "notification-center-list",
    sourceFile: `${notificationCenterFamilyRoot}notification-center-list.vue`,
    render: () => center(() => h(NotificationCenterList)),
    assertServerMarkup(html) {
      assert.match(html, /role="feed"/);
      assert.match(html, /id="runtime-inbox-list"/);
    },
    assertHydratedDom(host) {
      assert.equal(part(host, "notification-center-list").getAttribute("role"), "feed");
    },
  },
  {
    name: "notification-center-item",
    sourceFile: `${notificationCenterFamilyRoot}notification-center-item.vue`,
    render: () =>
      center(() =>
        h(NotificationCenterList, null, {
          item: ({ position, setSize }: { readonly position: number; readonly setSize: number }) =>
            h(NotificationCenterItem, {
              notification: {
                archived: false,
                createdAt: 0,
                data: undefined,
                description: null,
                group: null,
                id: "build",
                read: false,
                title: "Build passed",
                type: "neutral",
                updatedAt: 0,
              },
              position,
              setSize,
            }),
        }),
      ),
    assertServerMarkup(html) {
      assert.match(html, /<article id="runtime-inbox-item-build"/);
      assert.match(html, /aria-posinset="1"/);
    },
    assertHydratedDom(host) {
      assert.equal(part(host, "notification-center-item").tagName, "ARTICLE");
    },
  },
  {
    name: "notification-center-empty",
    sourceFile: `${notificationCenterFamilyRoot}notification-center-empty.vue`,
    render: () => center(() => h(NotificationCenterEmpty)),
    assertServerMarkup(html) {
      assert.match(html, /<p[^>]*\bhidden\b[^>]*data-vize-ui="notification-center-empty"/);
    },
    assertHydratedDom(host) {
      assert.equal(part(host, "notification-center-empty").hidden, true);
    },
  },
];
