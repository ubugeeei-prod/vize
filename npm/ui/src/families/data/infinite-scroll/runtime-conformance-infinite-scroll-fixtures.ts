import assert from "node:assert/strict";

import { h } from "vue";

import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";
import {
  InfiniteScrollItem,
  InfiniteScrollLoadMore,
  InfiniteScrollRoot,
  InfiniteScrollSentinel,
  InfiniteScrollStatus,
} from "./infinite-scroll.ts";

function feed(id: string) {
  return h(InfiniteScrollRoot, { id, feed: true, ariaLabel: "Activity", hasMore: true }, () => [
    h(InfiniteScrollItem, { index: 0 }, () => "Deployed v1"),
    h(InfiniteScrollItem, { index: 1 }, () => "Deployed v2"),
    h(InfiniteScrollSentinel),
    h(InfiniteScrollLoadMore, null, () => "Load older"),
    h(InfiniteScrollStatus, null, () => "Idle"),
  ]);
}

export const infiniteScrollRuntimeFixtures: readonly RuntimeFixture[] = [
  {
    name: "infinite-scroll",
    sourceFile: "families/data/infinite-scroll/infinite-scroll-root.vue",
    render: () => feed("activity-root"),
    assertServerMarkup(html) {
      assert.match(html, /id="activity-root"/);
      assert.match(html, /role="feed"/);
      assert.match(html, /aria-label="Activity"/);
      assert.match(html, /aria-busy="false"/);
      assert.match(html, /data-vize-ui="infinite-scroll-root"/);
      assert.match(html, /data-state="idle"/);
    },
    assertHydratedDom(host) {
      const root = host.querySelector('[data-vize-ui="infinite-scroll-root"]');
      assert.ok(root instanceof HTMLDivElement);
      assert.equal(root.getAttribute("role"), "feed");
      assert.equal(root.getAttribute("aria-busy"), "false");
    },
  },
  {
    name: "infinite-scroll-item",
    sourceFile: "families/data/infinite-scroll/infinite-scroll-item.vue",
    render: () => feed("activity-item"),
    assertServerMarkup(html) {
      assert.match(html, /<article tabindex="0" aria-posinset="1" aria-setsize="-1"/);
      assert.match(html, /data-vize-ui="infinite-scroll-item"/);
    },
    assertHydratedDom(host) {
      const items = host.querySelectorAll('[data-vize-ui="infinite-scroll-item"]');
      assert.equal(items.length, 2);
      assert.equal(items[1]?.getAttribute("aria-posinset"), "2");
    },
  },
  {
    name: "infinite-scroll-load-more",
    sourceFile: "families/data/infinite-scroll/infinite-scroll-load-more.vue",
    render: () => feed("activity-more"),
    assertServerMarkup(html) {
      assert.match(html, /<button type="button" aria-controls="activity-more"/);
      assert.match(html, /Load older/);
    },
    assertHydratedDom(host) {
      const button = host.querySelector('[data-vize-ui="infinite-scroll-load-more"]');
      assert.ok(button instanceof HTMLButtonElement);
      assert.equal(button.disabled, false);
      assert.equal(button.hidden, false);
    },
  },
  {
    name: "infinite-scroll-sentinel",
    sourceFile: "families/data/infinite-scroll/infinite-scroll-sentinel.vue",
    render: () => feed("activity-sentinel"),
    assertServerMarkup(html) {
      assert.match(html, /aria-hidden="true" data-vize-ui="infinite-scroll-sentinel"/);
    },
    assertHydratedDom(host) {
      const sentinel = host.querySelector('[data-vize-ui="infinite-scroll-sentinel"]');
      assert.ok(sentinel instanceof HTMLDivElement);
    },
  },
  {
    name: "infinite-scroll-status",
    sourceFile: "families/data/infinite-scroll/infinite-scroll-status.vue",
    render: () => feed("activity-status"),
    assertServerMarkup(html) {
      assert.match(html, /role="status" aria-live="polite" aria-atomic="true"/);
      assert.match(html, /Idle/);
    },
    assertHydratedDom(host) {
      const status = host.querySelector('[data-vize-ui="infinite-scroll-status"]');
      assert.ok(status instanceof HTMLDivElement);
      assert.equal(status.getAttribute("role"), "status");
    },
  },
];
