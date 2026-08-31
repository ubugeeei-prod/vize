import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import { resolveContainerLayout } from "./container-runtime.ts";
import type { ContainerExpose, ContainerSlotState } from "./container.ts";
import Container from "./container.vue";
import { mountInteraction } from "../../../testing/mount.ts";

test("resolves a centered default container with no authored CSS classes", () => {
  assert.deepEqual(resolveContainerLayout({}), {
    centered: true,
    maxInlineSize: "64rem",
    paddingInline: "0",
    size: "md",
    style: {
      "--vize-ui-container-max-inline-size": "64rem",
      "--vize-ui-container-padding-inline": "0",
      marginInline: "auto",
      maxInlineSize: "var(--vize-ui-container-max-inline-size)",
      paddingInline: "var(--vize-ui-container-padding-inline)",
    },
  });
});

test("resolves preset and numeric overrides into native logical CSS values", () => {
  assert.deepEqual(
    resolveContainerLayout({
      centered: false,
      maxInlineSize: 960,
      paddingInline: 24,
      size: "xl",
    }),
    {
      centered: false,
      maxInlineSize: "960px",
      paddingInline: "24px",
      size: "xl",
      style: {
        "--vize-ui-container-max-inline-size": "960px",
        "--vize-ui-container-padding-inline": "24px",
        maxInlineSize: "var(--vize-ui-container-max-inline-size)",
        paddingInline: "var(--vize-ui-container-padding-inline)",
      },
    },
  );
});

test("renders a non-focusable container by default while preserving child semantics", async () => {
  const handle = mountInteraction(Container, {
    slots: {
      default: '<button type="button">Filter</button><a href="/docs">Docs</a>',
    },
  });
  const root = handle.root();

  assert.equal(root.tagName, "DIV");
  assert.equal(root.getAttribute("class"), null);
  assert.equal(root.getAttribute("role"), null);
  assert.equal(root.getAttribute("aria-hidden"), null);
  assert.equal(root.getAttribute("tabindex"), null);
  assert.equal(root.getAttribute("part"), "root");
  assert.equal(root.getAttribute("data-vize-ui"), "container");
  assert.equal(root.getAttribute("data-size"), "md");
  assert.equal(root.getAttribute("data-centered"), "true");
  assert.equal(root.style.getPropertyValue("--vize-ui-container-max-inline-size"), "64rem");
  assert.equal(root.style.getPropertyValue("--vize-ui-container-padding-inline"), "0");
  assert.equal(root.style.marginInline, "auto");
  assert.equal(root.style.maxInlineSize, "var(--vize-ui-container-max-inline-size)");
  assert.equal(root.style.paddingInline, "var(--vize-ui-container-padding-inline)");
  assert.equal(await handle.tab(), handle.getByRole("button", { name: "Filter" }));
  handle.unmount();
});

test("renders an uncentered custom semantic host with forwarded attributes", () => {
  const handle = mountInteraction(Container, {
    props: {
      as: "main",
      centered: false,
      maxInlineSize: "72ch",
      paddingInline: "clamp(1rem, 2vw, 2rem)",
      size: "full",
    },
    attrs: {
      "aria-label": "Primary content",
    },
    slots: {
      default: "<section>Dashboard</section>",
    },
  });
  const root = handle.root();

  assert.equal(root.tagName, "MAIN");
  assert.equal(root.getAttribute("aria-label"), "Primary content");
  assert.equal(root.getAttribute("data-size"), "full");
  assert.equal(root.getAttribute("data-centered"), "false");
  assert.equal(root.style.marginInline ?? "", "");
  assert.equal(root.style.getPropertyValue("--vize-ui-container-max-inline-size"), "72ch");
  assert.equal(
    root.style.getPropertyValue("--vize-ui-container-padding-inline"),
    "clamp(1rem, 2vw, 2rem)",
  );
  assert.equal(root.children.length, 1);
  handle.unmount();
});

test("passes slot state and exposes live resolved logical sizing state", async () => {
  const handle = mountInteraction(Container, {
    props: {
      paddingInline: 8,
    },
    slots: {
      default: (state: ContainerSlotState) =>
        `${state.size}:${state.maxInlineSize}:${state.paddingInline}:${state.centered}:${state.style.marginInline ?? ""}`,
    },
  });
  const exposed = handle.exposes<ContainerExpose>();

  assert.ok(exposed.element === handle.root());
  assert.equal(exposed.size, "md");
  assert.equal(exposed.maxInlineSize, "64rem");
  assert.equal(exposed.paddingInline, "8px");
  assert.equal(exposed.centered, true);
  assert.equal(exposed.style.marginInline, "auto");
  assert.equal(handle.root().textContent, "md:64rem:8px:true:auto");

  await handle.wrapper.setProps({
    centered: false,
    maxInlineSize: "min(100%, 70rem)",
    paddingInline: "2rem",
    size: "lg",
  });
  assert.equal(exposed.size, "lg");
  assert.equal(exposed.maxInlineSize, "min(100%, 70rem)");
  assert.equal(exposed.paddingInline, "2rem");
  assert.equal(exposed.centered, false);
  assert.equal(exposed.style.marginInline, undefined);
  assert.equal(handle.root().textContent, "lg:min(100%, 70rem):2rem:false:");
  handle.unmount();
});
