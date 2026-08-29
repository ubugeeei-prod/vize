import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import { resolveStackLayout } from "./stack-runtime.ts";
import type { StackExpose, StackSlotState } from "./stack.ts";
import Stack from "./stack.vue";
import { mountInteraction } from "./testing/mount.ts";

test("resolves a logical block stack without authored wrapping behavior", () => {
  assert.deepEqual(resolveStackLayout({}), {
    align: "stretch",
    axis: "block",
    direction: "column",
    gap: "1rem",
    justify: "start",
    reversed: false,
    state: "stacked",
    style: {
      "--vize-ui-stack-align": "stretch",
      "--vize-ui-stack-gap": "1rem",
      "--vize-ui-stack-justify": "start",
      alignItems: "var(--vize-ui-stack-align)",
      display: "flex",
      flexDirection: "column",
      gap: "var(--vize-ui-stack-gap)",
      justifyContent: "var(--vize-ui-stack-justify)",
    },
  });
  assert.equal("flexWrap" in resolveStackLayout({}).style, false);
});

test("resolves reversed inline flow with native logical alignment values", () => {
  assert.deepEqual(
    resolveStackLayout({
      align: "center",
      axis: "inline",
      gap: "clamp(0.5rem, 2vi, 2rem)",
      justify: "space-between",
      reversed: true,
    }),
    {
      align: "center",
      axis: "inline",
      direction: "row-reverse",
      gap: "clamp(0.5rem, 2vi, 2rem)",
      justify: "space-between",
      reversed: true,
      state: "stacked",
      style: {
        "--vize-ui-stack-align": "center",
        "--vize-ui-stack-gap": "clamp(0.5rem, 2vi, 2rem)",
        "--vize-ui-stack-justify": "space-between",
        alignItems: "var(--vize-ui-stack-align)",
        display: "flex",
        flexDirection: "row-reverse",
        gap: "var(--vize-ui-stack-gap)",
        justifyContent: "var(--vize-ui-stack-justify)",
      },
    },
  );
});

test("renders a non-focusable block stack by default while preserving child semantics", async () => {
  const handle = mountInteraction(Stack, {
    slots: {
      default: '<button type="button">Continue</button>',
    },
  });
  const root = handle.root();

  assert.equal(root.tagName, "DIV");
  assert.equal(root.getAttribute("role"), null);
  assert.equal(root.getAttribute("aria-hidden"), null);
  assert.equal(root.getAttribute("tabindex"), null);
  assert.equal(root.getAttribute("part"), "root");
  assert.equal(root.getAttribute("data-vize-ui"), "stack");
  assert.equal(root.getAttribute("data-state"), "stacked");
  assert.equal(root.getAttribute("data-axis"), "block");
  assert.equal(root.getAttribute("data-reversed"), "false");
  assert.equal(root.getAttribute("data-vize-stack-direction"), "column");
  assert.equal(root.getAttribute("data-vize-stack-gap"), "1rem");
  assert.equal(root.getAttribute("data-vize-stack-align"), "stretch");
  assert.equal(root.getAttribute("data-vize-stack-justify"), "start");
  assert.equal(root.style.getPropertyValue("--vize-ui-stack-gap"), "1rem");
  assert.equal(root.style.getPropertyValue("--vize-ui-stack-align"), "stretch");
  assert.equal(root.style.getPropertyValue("--vize-ui-stack-justify"), "start");
  assert.equal(root.style.display, "flex");
  assert.equal(root.style.flexDirection, "column");
  assert.equal(root.style.gap, "var(--vize-ui-stack-gap)");
  assert.equal(root.style.alignItems, "var(--vize-ui-stack-align)");
  assert.equal(root.style.justifyContent, "var(--vize-ui-stack-justify)");
  assert.equal(await handle.tab(), handle.getByRole("button", { name: "Continue" }));
  handle.unmount();
});

test("renders an RTL-aware inline stack on a custom host", () => {
  const handle = mountInteraction(Stack, {
    props: {
      align: "center",
      as: "nav",
      axis: "inline",
      gap: "2ch",
      justify: "end",
      reversed: true,
    },
    attrs: {
      "aria-label": "Breadcrumb",
      dir: "rtl",
    },
    slots: {
      default: '<a href="/docs">Docs</a><a href="/api">API</a>',
    },
  });
  const root = handle.root();

  assert.equal(root.tagName, "NAV");
  assert.equal(root.getAttribute("aria-label"), "Breadcrumb");
  assert.equal(root.getAttribute("dir"), "rtl");
  assert.equal(root.getAttribute("data-axis"), "inline");
  assert.equal(root.getAttribute("data-reversed"), "true");
  assert.equal(root.getAttribute("data-vize-stack-direction"), "row-reverse");
  assert.equal(root.getAttribute("data-vize-stack-gap"), "2ch");
  assert.equal(root.getAttribute("data-vize-stack-align"), "center");
  assert.equal(root.getAttribute("data-vize-stack-justify"), "end");
  assert.equal(root.style.flexDirection, "row-reverse");
  assert.equal(root.style.getPropertyValue("--vize-ui-stack-gap"), "2ch");
  assert.equal(root.children.length, 2);
  handle.unmount();
});

test("passes slot state and exposes live resolved layout state", async () => {
  const handle = mountInteraction(Stack, {
    props: {
      gap: "0.5rem",
    },
    slots: {
      default: (state: StackSlotState) =>
        `${state.axis}:${state.reversed}:${state.direction}:${state.gap}:${state.align}:${state.justify}:${state.state}`,
    },
  });
  const exposed = handle.exposes<StackExpose>();

  assert.ok(exposed.element === handle.root());
  assert.equal(exposed.axis, "block");
  assert.equal(exposed.reversed, false);
  assert.equal(exposed.direction, "column");
  assert.equal(exposed.gap, "0.5rem");
  assert.equal(exposed.align, "stretch");
  assert.equal(exposed.justify, "start");
  assert.equal(exposed.state, "stacked");
  assert.equal(handle.root().textContent, "block:false:column:0.5rem:stretch:start:stacked");

  await handle.wrapper.setProps({
    align: "baseline",
    axis: "inline",
    gap: "1lh",
    justify: "space-evenly",
    reversed: true,
  });
  assert.equal(exposed.axis, "inline");
  assert.equal(exposed.reversed, true);
  assert.equal(exposed.direction, "row-reverse");
  assert.equal(exposed.gap, "1lh");
  assert.equal(exposed.align, "baseline");
  assert.equal(exposed.justify, "space-evenly");
  assert.equal(
    handle.root().textContent,
    "inline:true:row-reverse:1lh:baseline:space-evenly:stacked",
  );
  handle.unmount();
});
