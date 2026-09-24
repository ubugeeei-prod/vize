import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { h, nextTick } from "vue";

import Avatar from "../avatar/avatar.vue";
import type { AvatarGroupExpose, AvatarGroupOverflowSlotState } from "./avatar-group.ts";
import AvatarGroup from "./avatar-group.vue";
import AvatarGroupOverflow from "./avatar-group-overflow.vue";
import {
  avatarGroupSpacing,
  resolveAvatarGroupMessages,
  splitAvatarGroup,
} from "./avatar-group-state.ts";
import { mountInteraction } from "../../../testing/mount.ts";

interface Person {
  readonly id: string;
  readonly name: string;
}

const people: readonly Person[] = ["Ada", "Grace", "Linus", "Barbara", "Ken"].map((name) => ({
  id: name.toLowerCase(),
  name,
}));

function mountGroup(props: Record<string, unknown> = {}, slots: Record<string, unknown> = {}) {
  return mountInteraction(AvatarGroup, {
    props: { items: people, ariaLabel: "Project members", ...props },
    slots: {
      item: ({ item }: { item: Person }) => h(Avatar, { name: item.name, fallback: item.name[0] }),
      ...slots,
    },
  });
}

test("renders a labelled list of avatars without collapsing by default", () => {
  const handle = mountGroup();
  const root = handle.root();
  assert.equal(root.tagName, "UL");
  assert.equal(root.getAttribute("aria-label"), "Project members");
  assert.equal(root.getAttribute("data-vize-ui"), "avatar-group");
  assert.equal(root.getAttribute("data-state"), "expanded");
  assert.equal(root.getAttribute("data-count"), "5");
  assert.equal(root.hasAttribute("data-overflow"), false);
  const items = root.querySelectorAll('[data-vize-ui="avatar-group-item"]');
  assert.equal(items.length, 5);
  assert.equal(items[0]?.tagName, "LI");
  assert.equal(items[0]?.getAttribute("data-index"), "0");
  assert.equal(items[0]?.querySelector('[data-vize-ui="avatar"]')?.textContent?.trim(), "A");
  assert.equal(root.querySelector('[data-vize-ui="avatar-group-overflow"]'), null);
  handle.unmount();
});

test("max counts the overflow tile and announces hidden people", () => {
  const handle = mountGroup({ max: 3 });
  const root = handle.root();
  assert.equal(root.getAttribute("data-state"), "collapsed");
  assert.equal(root.querySelectorAll('[data-vize-ui="avatar-group-item"]').length, 2);
  assert.equal(root.getAttribute("data-overflow"), "3");
  const overflow = handle.getByRole("img", { name: "3 more" });
  assert.equal(overflow.getAttribute("data-vize-ui"), "avatar-group-overflow");
  assert.equal(overflow.getAttribute("data-count"), "3");
  assert.equal(overflow.textContent?.trim(), "+3");
  assert.equal(
    overflow.querySelector('[data-vize-ui="avatar"]')?.getAttribute("aria-hidden"),
    "true",
  );
  assert.equal(root.children.length, 3, "max bounds every tile");
  handle.unmount();
});

test("total adds server-side people to the overflow count", () => {
  const handle = mountGroup({ items: people.slice(0, 2), total: 40 });
  assert.equal(handle.getByRole("img").getAttribute("aria-label"), "38 more");
  assert.equal(handle.root().querySelectorAll('[data-vize-ui="avatar-group-item"]').length, 2);
  handle.unmount();

  const bounded = mountGroup({ items: people.slice(0, 2), total: 40, max: 2 });
  assert.equal(bounded.root().querySelectorAll('[data-vize-ui="avatar-group-item"]').length, 1);
  assert.equal(bounded.getByRole("img").getAttribute("aria-label"), "39 more");
  bounded.unmount();
});

test("localized messages, spacing, and empty groups", () => {
  const handle = mountGroup({
    max: 2,
    spacing: -8,
    messages: {
      overflow: (count: number) => `他 ${count} 人`,
      overflowText: (count: number) => `${count}+`,
    },
  });
  assert.equal(handle.getByRole("img").getAttribute("aria-label"), "他 4 人");
  assert.equal(handle.getByRole("img").textContent?.trim(), "4+");
  assert.equal(handle.root().style.getPropertyValue("--vize-ui-avatar-group-spacing"), "-8px");
  handle.unmount();

  const empty = mountGroup({ items: [], spacing: "0.5rem" });
  assert.equal(empty.root().getAttribute("data-state"), "empty");
  assert.equal(empty.root().children.length, 0);
  assert.equal(empty.root().style.getPropertyValue("--vize-ui-avatar-group-spacing"), "0.5rem");
  empty.unmount();
});

test("the overflow slot receives hidden items for custom tiles", () => {
  const handle = mountGroup(
    { max: 2 },
    {
      overflow: (state: AvatarGroupOverflowSlotState<Person>) =>
        h(
          AvatarGroupOverflow,
          { count: state.count, items: state.hiddenItems },
          {
            default: (inner: AvatarGroupOverflowSlotState<Person>) =>
              h(
                "span",
                { title: inner.hiddenItems.map((person) => person.name).join(", ") },
                inner.text,
              ),
          },
        ),
    },
  );
  const tile = handle.getByRole("img", { name: "4 more" });
  assert.equal(tile.querySelector("[title]")?.getAttribute("title"), "Grace, Linus, Barbara, Ken");
  handle.unmount();
});

test("items and max are reactive and the instance exposes the split", async () => {
  const handle = mountGroup({ max: 4 });
  const exposed = handle.exposes<AvatarGroupExpose<Person>>();
  assert.equal(exposed.state, "collapsed");
  assert.equal(exposed.overflowCount, 2);
  assert.deepEqual(
    exposed.hiddenItems.map((person) => person.id),
    ["barbara", "ken"],
  );
  assert.ok(exposed.element === handle.root());
  await handle.wrapper.setProps({ max: 10 });
  await nextTick();
  assert.equal(exposed.state, "expanded");
  assert.equal(exposed.visibleItems.length, 5);
  handle.unmount();
});

test("splits groups, resolves messages, and sanitizes spacing", () => {
  assert.deepEqual(splitAvatarGroup([], 3), {
    visibleItems: [],
    hiddenItems: [],
    overflowCount: 0,
    state: "empty",
  });
  assert.equal(splitAvatarGroup([1, 2, 3], 3).state, "expanded");
  assert.deepEqual(splitAvatarGroup([1, 2, 3, 4], 3).visibleItems, [1, 2]);
  assert.equal(splitAvatarGroup([1, 2, 3, 4], 1).visibleItems.length, 0);
  assert.equal(splitAvatarGroup([1, 2, 3, 4], 1).overflowCount, 4);
  assert.equal(splitAvatarGroup([1, 2], 0).state, "expanded", "invalid max never collapses");
  assert.equal(splitAvatarGroup([1, 2], Number.NaN).state, "expanded");
  assert.equal(
    splitAvatarGroup([1, 2], undefined, 1).overflowCount,
    0,
    "total below items is ignored",
  );
  assert.equal(splitAvatarGroup([1], undefined, Number.POSITIVE_INFINITY).overflowCount, 0);
  assert.equal(resolveAvatarGroupMessages(undefined).overflow(2), "2 more");
  assert.equal(resolveAvatarGroupMessages({ overflowText: () => "…" }).overflowText(2), "…");
  assert.equal(avatarGroupSpacing(4), "4px");
  assert.equal(avatarGroupSpacing(Number.NaN), undefined);
  assert.equal(avatarGroupSpacing(" -0.25rem "), "-0.25rem");
  assert.equal(avatarGroupSpacing("1px; color: red"), undefined);
  assert.equal(avatarGroupSpacing(""), undefined);
  assert.equal(avatarGroupSpacing(undefined), undefined);
});
