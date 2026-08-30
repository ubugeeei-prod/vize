import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h, nextTick } from "vue";

import IdProvider from "../../foundations/id/deterministic-id-provider.vue";
import { mountInteraction } from "../../../testing/mount.ts";
import SkipLink from "./skip-link.vue";
import type { SkipLinkActivation, SkipLinkExpose, SkipLinkSlotState } from "./skip-link.ts";

function appendTarget(id = "main"): HTMLElement {
  const target = document.createElement("main");
  target.id = id;
  target.textContent = "Main content";
  document.body.append(target);
  return target;
}

test("renders a native same-document link with accessibility hooks", async () => {
  const handle = mountInteraction(SkipLink, {
    props: { href: "#content", id: "skip-content" },
    slots: { default: "Skip to content" },
  });
  const link = handle.getByRole("link", { name: "Skip to content" }) as HTMLAnchorElement;

  assert.equal(link.tagName, "A");
  assert.equal(link.id, "skip-content");
  assert.equal(link.getAttribute("href"), "#content");
  assert.equal(link.getAttribute("aria-disabled"), null);
  assert.equal(link.getAttribute("data-vize-ui"), "skip-link");
  assert.equal(link.getAttribute("part"), "root");
  assert.equal(link.getAttribute("data-state"), "idle");
  assert.equal(link.getAttribute("data-target-id"), "content");
  assert.equal(link.getAttribute("data-unavailable"), null);
  assert.ok((await handle.tab()) === link, "skip link must join the tab order");
  handle.unmount();
});

test("uses the deterministic id scope when no id is supplied", () => {
  const ScopedSkipLink = defineComponent({
    setup: () => () =>
      h(
        IdProvider,
        { prefix: "app", seed: "request" },
        { default: () => h(SkipLink, { href: "#main" }, { default: () => "Skip" }) },
      ),
  });
  const handle = mountInteraction(ScopedSkipLink);
  const link = handle.getByRole("link", { name: "Skip" });

  assert.equal(link.id, "app-request-skip-link-0");
  handle.unmount();
});

test("exposes focus state to attributes, slots, and the public instance", async () => {
  const handle = mountInteraction(SkipLink, {
    slots: {
      default: (state: SkipLinkSlotState) =>
        [
          state.href ?? "none",
          state.targetId ?? "none",
          state.state,
          state.focused ? "focused" : "blurred",
          state.unavailable ? "unavailable" : "available",
        ].join(":"),
    },
  });
  const link = handle.getByRole("link") as HTMLAnchorElement;
  const exposed = handle.exposes<SkipLinkExpose>();

  assert.equal(handle.root().textContent, "#main:main:idle:blurred:available");
  assert.equal(exposed.href, "#main");
  assert.equal(exposed.targetId, "main");
  assert.equal(exposed.state, "idle");
  assert.equal(exposed.focused, false);

  exposed.focus();
  await nextTick();
  assert.ok(handle.activeElement() === link);
  assert.equal(link.getAttribute("data-state"), "focused");
  assert.equal(handle.root().textContent, "#main:main:focused:focused:available");
  assert.equal(exposed.state, "focused");
  assert.equal(exposed.focused, true);

  link.blur();
  await nextTick();
  assert.equal(link.getAttribute("data-state"), "idle");
  assert.equal(handle.root().textContent, "#main:main:idle:blurred:available");
  handle.unmount();
});

test("activation emits detail and moves focus to the hash target", async () => {
  const target = appendTarget();
  let clicks = 0;
  const handle = mountInteraction(SkipLink, {
    attrs: { onClick: () => clicks++ },
    record: ["activate"],
    slots: { default: "Skip to main" },
  });
  const link = handle.getByRole("link", { name: "Skip to main" });

  await handle.click(link);

  const emissions = handle.wrapper.emitted("activate") as
    | [[MouseEvent, SkipLinkActivation]]
    | undefined;
  assert.equal(emissions?.length, 1);
  assert.ok(emissions[0]?.[0] instanceof MouseEvent);
  assert.deepEqual(
    handle.recorded().map((emit) => emit.event),
    ["activate"],
  );
  assert.equal(emissions[0]?.[1].href, "#main");
  assert.equal(emissions[0]?.[1].targetId, "main");
  assert.equal(emissions[0]?.[1].target, target);
  assert.equal(emissions[0]?.[1].focused, true);
  assert.equal(handle.activeElement(), target);
  assert.equal(target.getAttribute("tabindex"), "-1");
  assert.equal(clicks, 1);

  target.blur();
  assert.equal(target.getAttribute("tabindex"), null);
  handle.unmount();
  target.remove();
});

test("focusTarget=false preserves activation without moving DOM focus", async () => {
  const target = appendTarget("content");
  const handle = mountInteraction(SkipLink, {
    props: { focusTarget: false, href: "#content" },
    slots: { default: "Skip to content" },
  });
  const link = handle.getByRole("link", { name: "Skip to content" });

  await handle.click(link);

  const emissions = handle.wrapper.emitted("activate") as
    | [[MouseEvent, SkipLinkActivation]]
    | undefined;
  assert.equal(emissions?.length, 1);
  assert.equal(emissions[0]?.[1].target, target);
  assert.equal(emissions[0]?.[1].focused, false);
  assert.notEqual(handle.activeElement(), target);
  assert.equal(target.getAttribute("tabindex"), null);
  handle.unmount();
  target.remove();
});

test("invalid runtime href values remove native navigation and activation", async () => {
  let clicks = 0;
  const handle = mountInteraction(SkipLink, {
    attrs: { onClick: () => clicks++ },
    props: { href: "/main" },
    slots: {
      default: (state: SkipLinkSlotState) =>
        `${state.href ?? "none"}:${state.targetId ?? "none"}:${state.state}:${
          state.unavailable ? "unavailable" : "available"
        }`,
    },
  });
  const root = handle.root() as HTMLAnchorElement;

  assert.equal(handle.queryByRole("link"), null);
  assert.equal(root.getAttribute("href"), null);
  assert.equal(root.getAttribute("aria-disabled"), "true");
  assert.equal(root.getAttribute("tabindex"), "-1");
  assert.equal(root.getAttribute("data-state"), "invalid");
  assert.equal(root.getAttribute("data-unavailable"), "true");
  assert.equal(root.textContent, "none:none:invalid:unavailable");

  await handle.click(root);
  assert.equal(handle.wrapper.emitted("activate"), undefined);
  assert.equal(clicks, 0);

  await handle.wrapper.setProps({ href: "#main" });
  assert.equal(handle.getByRole("link"), root);
  assert.equal(root.getAttribute("href"), "#main");
  assert.equal(root.getAttribute("aria-disabled"), null);
  assert.equal(root.getAttribute("tabindex"), null);
  assert.equal(root.textContent, "#main:main:idle:available");
  handle.unmount();
});

test("exposed target helpers resolve missing and present targets", () => {
  const handle = mountInteraction(SkipLink, {
    props: { href: "#workspace" },
    slots: { default: "Skip to workspace" },
  });
  const exposed = handle.exposes<SkipLinkExpose>();

  assert.equal(exposed.getTarget(), null);
  assert.deepEqual(exposed.focusTarget(), { target: null, focused: false });

  const target = appendTarget("workspace");
  const result = exposed.focusTarget();

  assert.equal(result.target, target);
  assert.equal(result.focused, true);
  assert.equal(handle.activeElement(), target);
  handle.unmount();
  target.remove();
});
