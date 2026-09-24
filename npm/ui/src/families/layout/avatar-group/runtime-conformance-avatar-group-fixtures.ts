import assert from "node:assert/strict";

import { h } from "vue";

import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";
import Avatar from "../avatar/avatar.vue";
import { AvatarGroup, AvatarGroupOverflow } from "./avatar-group.ts";

export const avatarGroupRuntimeFixtures: readonly RuntimeFixture[] = [
  {
    name: "avatar-group",
    sourceFile: "families/layout/avatar-group/avatar-group.vue",
    render: () =>
      h(
        AvatarGroup,
        { items: ["Ada", "Grace", "Linus"], max: 2, ariaLabel: "Owners" },
        {
          item: ({ item }: { item: string }) => h(Avatar, { name: item, fallback: item.charAt(0) }),
        },
      ),
    assertServerMarkup(html) {
      assert.match(html, /^<ul aria-label="Owners"/);
      assert.match(html, /data-vize-ui="avatar-group"/);
      assert.match(html, /data-overflow="2"/);
    },
    assertHydratedDom(host) {
      const root = host.querySelector('[data-vize-ui="avatar-group"]');
      assert.ok(root instanceof HTMLUListElement);
      assert.equal(root.querySelectorAll('[data-vize-ui="avatar-group-item"]').length, 1);
    },
  },
  {
    name: "avatar-group-overflow",
    sourceFile: "families/layout/avatar-group/avatar-group-overflow.vue",
    render: () => h(AvatarGroupOverflow, { count: 7 }),
    assertServerMarkup(html) {
      assert.match(html, /^<span role="img" aria-label="7 more"/);
      assert.match(html, /\+7/);
    },
    assertHydratedDom(host) {
      const tile = host.querySelector('[data-vize-ui="avatar-group-overflow"]');
      assert.ok(tile instanceof HTMLSpanElement);
      assert.equal(tile.getAttribute("data-count"), "7");
    },
  },
];
