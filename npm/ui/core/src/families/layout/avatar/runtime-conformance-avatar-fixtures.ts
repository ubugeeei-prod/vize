import assert from "node:assert/strict";

import { h } from "vue";

import Avatar from "./avatar.vue";
import type { RuntimeFixture } from "../../../runtime-conformance-fixtures.ts";

export const avatarRuntimeFixture: RuntimeFixture = {
  name: "avatar",
  sourceFile: "families/layout/avatar/avatar.vue",
  render: () =>
    h(Avatar, {
      alt: "Aki Kimura",
      decoding: "async",
      fallback: "AK",
      loading: "lazy",
      name: "Aki Kimura",
      src: "/avatars/aki.png",
      status: "online",
    }),
  assertServerMarkup(html) {
    assert.match(html, /^<span/);
    assert.match(html, /part="root"/);
    assert.match(html, /data-vize-ui="avatar"/);
    assert.match(html, /data-state="image"/);
    assert.match(html, /data-status="online"/);
    assert.match(html, /data-image="present"/);
    assert.match(html, /data-name="present"/);
    assert.match(html, /data-fallback="present"/);
    assert.match(html, /data-vize-ui="avatar-image"/);
    assert.match(html, /src="\/avatars\/aki\.png"/);
    assert.match(html, /alt="Aki Kimura"/);
    assert.match(html, /loading="lazy"/);
    assert.match(html, /decoding="async"/);
    assert.doesNotMatch(html, /data-vize-ui="avatar-fallback"/);
    assert.doesNotMatch(html, /role=/);
    assert.doesNotMatch(html, /tabindex=/);
    assert.doesNotMatch(html, /aria-hidden=/);
    assert.doesNotMatch(html, /aria-live=/);
    assert.doesNotMatch(html, /style=/);
  },
  assertHydratedDom(host) {
    const avatar = host.querySelector('[data-vize-ui="avatar"]');
    const image = host.querySelector('[data-vize-ui="avatar-image"]');
    assert.ok(avatar instanceof HTMLSpanElement);
    assert.ok(image instanceof HTMLImageElement);
    assert.equal(avatar.getAttribute("part"), "root");
    assert.equal(avatar.getAttribute("data-state"), "image");
    assert.equal(avatar.getAttribute("data-status"), "online");
    assert.equal(avatar.getAttribute("data-image"), "present");
    assert.equal(avatar.getAttribute("data-name"), "present");
    assert.equal(avatar.getAttribute("data-fallback"), "present");
    assert.equal(avatar.getAttribute("role"), null);
    assert.equal(avatar.getAttribute("tabindex"), null);
    assert.equal(avatar.getAttribute("aria-hidden"), null);
    assert.equal(avatar.getAttribute("aria-live"), null);
    assert.equal(avatar.getAttribute("style"), null);
    assert.equal(image.getAttribute("src"), "/avatars/aki.png");
    assert.equal(image.getAttribute("alt"), "Aki Kimura");
    assert.equal(image.getAttribute("loading"), "lazy");
    assert.equal(image.getAttribute("decoding"), "async");
  },
};
