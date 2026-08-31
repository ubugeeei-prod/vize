import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { h, nextTick } from "vue";

import type { AvatarExpose, AvatarSlotState } from "./avatar.ts";
import Avatar from "./avatar.vue";
import { mountInteraction } from "../../../testing/mount.ts";

test("renders fallback content by default without adding semantics or styling", async () => {
  const handle = mountInteraction(Avatar, {
    props: {
      fallback: "AK",
      name: "Aki Kimura",
    },
  });
  const root = handle.root();
  const fallback = root.querySelector('[data-vize-ui="avatar-fallback"]');

  assert.equal(root.tagName, "SPAN");
  assert.equal(root.getAttribute("part"), "root");
  assert.equal(root.getAttribute("data-vize-ui"), "avatar");
  assert.equal(root.getAttribute("data-state"), "fallback");
  assert.equal(root.getAttribute("data-status"), "none");
  assert.equal(root.getAttribute("data-image"), "missing");
  assert.equal(root.getAttribute("data-name"), "present");
  assert.equal(root.getAttribute("data-fallback"), "present");
  assert.equal(root.getAttribute("role"), null);
  assert.equal(root.getAttribute("tabindex"), null);
  assert.equal(root.getAttribute("aria-hidden"), null);
  assert.equal(root.getAttribute("aria-live"), null);
  assert.equal(root.getAttribute("style"), null);
  assert.equal(root.querySelector('[data-vize-ui="avatar-image"]'), null);
  assert.ok(fallback instanceof HTMLSpanElement);
  assert.equal(fallback.getAttribute("part"), "fallback");
  assert.equal(fallback.getAttribute("data-status"), "none");
  assert.equal(fallback.textContent, "AK");
  assert.equal(await handle.tab(), null);
  handle.unmount();
});

test("renders native image semantics and forwards load events", async () => {
  const handle = mountInteraction(Avatar, {
    props: {
      alt: "Aki Kimura",
      crossOrigin: "anonymous",
      decoding: "async",
      fetchPriority: "low",
      loading: "lazy",
      name: "Aki Kimura",
      referrerPolicy: "no-referrer",
      src: "/avatars/aki.png",
      status: "online",
    },
    record: ["load"],
  });
  const root = handle.root();
  const image = root.querySelector('[data-vize-ui="avatar-image"]');

  assert.equal(root.getAttribute("data-state"), "image");
  assert.equal(root.getAttribute("data-status"), "online");
  assert.equal(root.getAttribute("data-image"), "present");
  assert.equal(root.getAttribute("data-name"), "present");
  assert.equal(root.getAttribute("data-fallback"), "missing");
  assert.equal(root.getAttribute("role"), null);
  assert.equal(root.getAttribute("tabindex"), null);
  assert.equal(root.getAttribute("style"), null);
  assert.equal(root.querySelector('[data-vize-ui="avatar-fallback"]'), null);
  assert.ok(image instanceof HTMLImageElement);
  assert.equal(image.getAttribute("part"), "image");
  assert.equal(image.getAttribute("src"), "/avatars/aki.png");
  assert.equal(image.getAttribute("alt"), "Aki Kimura");
  assert.equal(image.getAttribute("loading"), "lazy");
  assert.equal(image.getAttribute("decoding"), "async");
  assert.equal(image.getAttribute("fetchpriority"), "low");
  assert.equal(image.getAttribute("crossorigin"), "anonymous");
  assert.equal(image.getAttribute("referrerpolicy"), "no-referrer");

  image.dispatchEvent(new Event("load"));
  await nextTick();
  assert.equal(root.getAttribute("data-state"), "image");
  assert.deepEqual(
    handle.recorded().map((record) => record.event),
    ["load"],
  );
  handle.unmount();
});

test("renders fallback for unsafe image sources without forwarding src", () => {
  for (const src of ["javascript:alert(1)", "data:text/html;base64,PHNjcmlwdD4="]) {
    const handle = mountInteraction(Avatar, {
      props: {
        fallback: "AK",
        src,
        status: "offline",
      },
    });
    const root = handle.root();

    assert.equal(root.getAttribute("data-state"), "fallback");
    assert.equal(root.getAttribute("data-status"), "offline");
    assert.equal(root.getAttribute("data-image"), "missing");
    assert.equal(root.querySelector('[data-vize-ui="avatar-image"]'), null);
    assert.equal(root.textContent, "AK");
    handle.unmount();
  }
});

test("switches failed images to fallback while keeping consumer attrs on the root", async () => {
  const handle = mountInteraction(Avatar, {
    attrs: {
      "aria-label": "Aki Kimura profile",
      role: "group",
      tabindex: "0",
    },
    props: {
      alt: "Aki Kimura",
      fallback: "AK",
      name: "Aki Kimura",
      src: "/avatars/missing.png",
      status: "busy",
    },
    record: ["error"],
  });
  const root = handle.getByRole("group", { name: "Aki Kimura profile" });
  const image = root.querySelector('[data-vize-ui="avatar-image"]');

  assert.ok(image instanceof HTMLImageElement);
  image.dispatchEvent(new Event("error"));
  await nextTick();

  const fallback = root.querySelector('[data-vize-ui="avatar-fallback"]');
  assert.equal(root.getAttribute("data-state"), "fallback");
  assert.equal(root.getAttribute("data-status"), "busy");
  assert.equal(root.getAttribute("data-image"), "present");
  assert.equal(root.getAttribute("data-name"), "present");
  assert.equal(root.getAttribute("data-fallback"), "present");
  assert.equal(root.getAttribute("tabindex"), "0");
  assert.equal(root.querySelector('[data-vize-ui="avatar-image"]'), null);
  assert.ok(fallback instanceof HTMLSpanElement);
  assert.equal(fallback.textContent, "AK");
  assert.deepEqual(
    handle.recorded().map((record) => record.event),
    ["error"],
  );
  assert.equal(await handle.tab(), root);

  await handle.wrapper.setProps({
    src: "/avatars/recovered.png",
    status: "online",
  });
  assert.equal(root.getAttribute("data-state"), "image");
  assert.equal(root.getAttribute("data-status"), "online");
  const recovered = root.querySelector('[data-vize-ui="avatar-image"]');
  assert.ok(recovered instanceof HTMLImageElement);
  assert.equal(recovered.getAttribute("src"), "/avatars/recovered.png");
  handle.unmount();
});

test("passes slot state and exposes live avatar state", async () => {
  const handle = mountInteraction(Avatar, {
    props: {
      fallback: "AK",
      name: "Aki Kimura",
      status: "away",
    },
    slots: {
      fallback: (state: AvatarSlotState) =>
        h(
          "span",
          `${state.state}:${state.image}:${state.nameState}:${state.fallbackState}:${state.status}:${state.name}:${state.fallback}`,
        ),
    },
  });
  const exposed = handle.exposes<AvatarExpose>();

  assert.ok(exposed.element === handle.root());
  assert.equal(exposed.imageElement, null);
  assert.ok(exposed.fallbackElement instanceof HTMLSpanElement);
  assert.equal(exposed.state, "fallback");
  assert.equal(exposed.status, "away");
  assert.equal(exposed.src, undefined);
  assert.equal(exposed.alt, "");
  assert.equal(exposed.name, "Aki Kimura");
  assert.equal(exposed.fallback, "AK");
  assert.equal(exposed.image, "missing");
  assert.equal(exposed.nameState, "present");
  assert.equal(exposed.fallbackState, "present");
  assert.equal(handle.root().textContent, "fallback:missing:present:present:away:Aki Kimura:AK");

  await handle.wrapper.setProps({
    alt: "Aki Kimura portrait",
    src: "/avatars/aki.png",
    status: "online",
  });

  assert.equal(exposed.state, "image");
  assert.equal(exposed.status, "online");
  assert.equal(exposed.src, "/avatars/aki.png");
  assert.equal(exposed.alt, "Aki Kimura portrait");
  assert.equal(exposed.image, "present");
  assert.ok(exposed.imageElement instanceof HTMLImageElement);
  assert.equal(exposed.fallbackElement, null);
  assert.equal(handle.root().textContent, "");
  handle.unmount();
});
