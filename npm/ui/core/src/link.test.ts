import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h } from "vue";

import IdProvider from "./deterministic-id-provider.vue";
import LinkAnchor from "./link-anchor.vue";
import { mountInteraction } from "./testing/mount.ts";

test("renders a native link with navigation and accessibility attributes", async () => {
  const handle = mountInteraction(LinkAnchor, {
    props: {
      id: "docs-link",
      href: "/docs",
      target: "_blank",
      rel: "noopener",
      download: "guide.pdf",
      ariaCurrent: "page",
    },
    slots: { default: "Docs" },
  });
  const link = handle.getByRole("link", { name: "Docs" }) as HTMLAnchorElement;

  assert.equal(link.tagName, "A");
  assert.equal(link.id, "docs-link");
  assert.equal(link.getAttribute("href"), "/docs");
  assert.equal(link.getAttribute("target"), "_blank");
  assert.equal(link.getAttribute("rel"), "noopener");
  assert.equal(link.getAttribute("download"), "guide.pdf");
  assert.equal(link.getAttribute("aria-current"), "page");
  assert.equal(link.getAttribute("aria-disabled"), null);
  assert.equal(link.getAttribute("data-vize-ui"), "link");
  assert.equal(link.getAttribute("data-state"), "idle");

  await handle.wrapper.setProps({ ariaCurrent: false });
  assert.equal(link.getAttribute("aria-current"), null);
  handle.unmount();
});

test("uses the deterministic id scope when no id is supplied", () => {
  const ScopedLink = defineComponent({
    setup: () => () =>
      h(
        IdProvider,
        { prefix: "nav", seed: "request" },
        { default: () => h(LinkAnchor, { href: "/docs" }, { default: () => "Docs" }) },
      ),
  });
  const handle = mountInteraction(ScopedLink);
  const link = handle.getByRole("link", { name: "Docs" }) as HTMLAnchorElement;

  assert.equal(link.id, "nav-request-link-0");
  handle.unmount();
});

test("joins the tab order and focuses programmatically", async () => {
  const handle = mountInteraction(LinkAnchor, {
    props: { href: "/docs" },
    slots: { default: "Docs" },
  });
  const link = handle.getByRole("link");

  assert.ok((await handle.tab()) === link, "Tab must move focus to the link");
  assert.ok(handle.activeElement() === link);

  link.blur();
  handle.exposes<{ focus: (options?: FocusOptions) => void }>().focus();
  assert.ok(handle.activeElement() === link, "exposed focus() must focus the anchor");
  handle.unmount();
});

test("click emits navigate and preserves consumer click listeners", async () => {
  let clicks = 0;
  const handle = mountInteraction(LinkAnchor, {
    props: { href: "/docs" },
    attrs: { onClick: () => clicks++ },
    slots: { default: "Docs" },
    record: ["navigate"],
  });
  const link = handle.getByRole("link");

  await handle.click(link);

  const navigations = handle.wrapper.emitted("navigate");
  assert.equal(navigations?.length, 1);
  assert.ok(navigations?.[0]?.[0] instanceof MouseEvent);
  assert.equal(clicks, 1);
  assert.deepEqual(
    handle.recorded().map((emit) => emit.event),
    ["navigate"],
  );
  handle.unmount();
});

test("unsafe href values do not render navigable links", async () => {
  const handle = mountInteraction(LinkAnchor, {
    props: { href: " javascript:alert(1) " },
    slots: { default: "Docs" },
  });
  const link = handle.root() as HTMLAnchorElement;

  assert.equal(handle.queryByRole("link"), null);
  assert.equal(link.getAttribute("href"), null);

  await handle.wrapper.setProps({ href: "https://vize.dev/docs" });
  assert.equal(handle.getByRole("link", { name: "Docs" }), link);
  assert.equal(link.getAttribute("href"), "https://vize.dev/docs");

  await handle.wrapper.setProps({ href: "java\nscript:alert(1)" });
  assert.equal(handle.queryByRole("link"), null);
  assert.equal(link.getAttribute("href"), null);
  handle.unmount();
});

test("Enter activates native links while Space remains non-activating", async () => {
  const handle = mountInteraction(LinkAnchor, {
    props: { href: "/docs" },
    slots: { default: "Docs" },
  });
  const link = handle.getByRole("link");
  link.focus();

  const enter = await handle.press(link, "Enter");
  assert.equal(enter.activated, true);
  assert.equal(handle.wrapper.emitted("navigate")?.length, 1);

  const space = await handle.press(link, " ");
  assert.equal(space.activated, false);
  assert.equal(space.keydownPrevented, false);
  assert.equal(handle.wrapper.emitted("navigate")?.length, 1);
  handle.unmount();
});

test("disabled links remove navigation, tab focus, and activation", async () => {
  let clicks = 0;
  const handle = mountInteraction(LinkAnchor, {
    props: { href: "/danger", target: "_blank", rel: "noopener", download: true, disabled: true },
    attrs: { onClick: () => clicks++ },
    slots: { default: "Delete" },
  });
  const link = handle.root() as HTMLAnchorElement;

  assert.equal(link.tagName, "A");
  assert.equal(handle.queryByRole("link"), null);
  assert.equal(link.getAttribute("href"), null);
  assert.equal(link.getAttribute("target"), null);
  assert.equal(link.getAttribute("rel"), null);
  assert.equal(link.getAttribute("download"), null);
  assert.equal(link.getAttribute("aria-disabled"), "true");
  assert.equal(link.getAttribute("tabindex"), "-1");
  assert.equal(link.getAttribute("data-state"), "disabled");

  await handle.click(link);
  await handle.press(link, "Enter");
  await handle.press(link, " ");
  assert.equal(handle.wrapper.emitted("navigate"), undefined);
  assert.equal(clicks, 0);
  assert.ok((await handle.tab()) === null, "a disabled link must leave the tab order");
  handle.unmount();
});

test("inert links expose native inertness and suppress fallback activation", async () => {
  const handle = mountInteraction(LinkAnchor, {
    props: { href: "/archive", inert: true },
    slots: { default: "Archive" },
  });
  const link = handle.root() as HTMLAnchorElement;

  assert.equal(link.getAttribute("href"), null);
  assert.ok(link.hasAttribute("inert"));
  assert.equal(link.getAttribute("aria-disabled"), "true");
  assert.equal(link.getAttribute("tabindex"), "-1");
  assert.equal(link.getAttribute("data-state"), "inert");

  await handle.click(link);
  await handle.press(link, "Enter");
  await handle.press(link, " ");
  assert.equal(handle.wrapper.emitted("navigate"), undefined);
  assert.ok((await handle.tab()) === null, "an inert link must leave the tab order");
  handle.unmount();
});

test("exposes disabled, inert, and unavailable to the default slot", async () => {
  const handle = mountInteraction(LinkAnchor, {
    props: { href: "/docs" },
    slots: {
      default: (state: { disabled: boolean; inert: boolean; unavailable: boolean }) =>
        `disabled:${state.disabled} inert:${state.inert} unavailable:${state.unavailable}`,
    },
  });

  assert.equal(handle.root().textContent, "disabled:false inert:false unavailable:false");
  await handle.wrapper.setProps({ inert: true });
  assert.equal(handle.root().textContent, "disabled:false inert:true unavailable:true");
  await handle.wrapper.setProps({ disabled: true, inert: false });
  assert.equal(handle.root().textContent, "disabled:true inert:false unavailable:true");
  handle.unmount();
});
