import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import { resolveClusterLayout } from "./cluster-runtime.ts";
import type { ClusterExpose, ClusterSlotState } from "./cluster.ts";
import Cluster from "./cluster.vue";
import { mountInteraction } from "../../../testing/mount.ts";

test("resolves a wrapping inline cluster with no authored CSS classes", () => {
  assert.deepEqual(resolveClusterLayout({}), {
    align: "stretch",
    direction: "row",
    gap: "0",
    justify: "start",
    reversed: false,
    state: "clustered",
    wrap: true,
    wrapMode: "wrap",
    style: {
      "--vize-ui-cluster-align": "stretch",
      "--vize-ui-cluster-gap": "0",
      "--vize-ui-cluster-justify": "start",
      alignItems: "var(--vize-ui-cluster-align)",
      display: "flex",
      flexDirection: "row",
      flexWrap: "wrap",
      gap: "var(--vize-ui-cluster-gap)",
      justifyContent: "var(--vize-ui-cluster-justify)",
    },
  });
});

test("resolves reversed nowrap flow with native logical alignment values", () => {
  assert.deepEqual(
    resolveClusterLayout({
      align: "baseline",
      gap: 8,
      justify: "space-evenly",
      reversed: true,
      wrap: false,
    }),
    {
      align: "baseline",
      direction: "row-reverse",
      gap: "8px",
      justify: "space-evenly",
      reversed: true,
      state: "clustered",
      wrap: false,
      wrapMode: "nowrap",
      style: {
        "--vize-ui-cluster-align": "baseline",
        "--vize-ui-cluster-gap": "8px",
        "--vize-ui-cluster-justify": "space-evenly",
        alignItems: "var(--vize-ui-cluster-align)",
        display: "flex",
        flexDirection: "row-reverse",
        flexWrap: "nowrap",
        gap: "var(--vize-ui-cluster-gap)",
        justifyContent: "var(--vize-ui-cluster-justify)",
      },
    },
  );
});

test("renders a non-focusable cluster by default while preserving child semantics", async () => {
  const handle = mountInteraction(Cluster, {
    slots: {
      default: '<button type="button">Filter</button><a href="/docs">Docs</a>',
    },
  });
  const root = handle.root();

  assert.equal(root.tagName, "DIV");
  assert.equal(root.getAttribute("role"), null);
  assert.equal(root.getAttribute("aria-hidden"), null);
  assert.equal(root.getAttribute("tabindex"), null);
  assert.equal(root.getAttribute("part"), "root");
  assert.equal(root.getAttribute("data-vize-ui"), "cluster");
  assert.equal(root.getAttribute("data-state"), "clustered");
  assert.equal(root.getAttribute("data-wrap"), "true");
  assert.equal(root.getAttribute("data-reversed"), "false");
  assert.equal(root.getAttribute("data-align"), "stretch");
  assert.equal(root.getAttribute("data-justify"), "start");
  assert.equal(root.getAttribute("data-vize-cluster-direction"), "row");
  assert.equal(root.getAttribute("data-vize-cluster-gap"), "0");
  assert.equal(root.style.getPropertyValue("--vize-ui-cluster-gap"), "0");
  assert.equal(root.style.getPropertyValue("--vize-ui-cluster-align"), "stretch");
  assert.equal(root.style.getPropertyValue("--vize-ui-cluster-justify"), "start");
  assert.equal(root.style.display, "flex");
  assert.equal(root.style.flexDirection, "row");
  assert.equal(root.style.flexWrap, "wrap");
  assert.equal(root.style.gap, "var(--vize-ui-cluster-gap)");
  assert.equal(root.style.alignItems, "var(--vize-ui-cluster-align)");
  assert.equal(root.style.justifyContent, "var(--vize-ui-cluster-justify)");
  assert.equal(await handle.tab(), handle.getByRole("button", { name: "Filter" }));
  handle.unmount();
});

test("renders nowrap reversed flow on a custom semantic host", () => {
  const handle = mountInteraction(Cluster, {
    props: {
      align: "center",
      as: "nav",
      gap: "0.5rem",
      justify: "space-between",
      reversed: true,
      wrap: false,
    },
    attrs: {
      "aria-label": "Actions",
    },
    slots: {
      default: '<a href="/one">One</a><a href="/two">Two</a>',
    },
  });
  const root = handle.root();

  assert.equal(root.tagName, "NAV");
  assert.equal(root.getAttribute("aria-label"), "Actions");
  assert.equal(root.getAttribute("data-wrap"), "false");
  assert.equal(root.getAttribute("data-reversed"), "true");
  assert.equal(root.getAttribute("data-align"), "center");
  assert.equal(root.getAttribute("data-justify"), "space-between");
  assert.equal(root.getAttribute("data-vize-cluster-direction"), "row-reverse");
  assert.equal(root.getAttribute("data-vize-cluster-gap"), "0.5rem");
  assert.equal(root.style.flexDirection, "row-reverse");
  assert.equal(root.style.flexWrap, "nowrap");
  assert.equal(root.style.getPropertyValue("--vize-ui-cluster-gap"), "0.5rem");
  assert.equal(root.children.length, 2);
  handle.unmount();
});

test("passes slot state and exposes live resolved layout state", async () => {
  const handle = mountInteraction(Cluster, {
    props: {
      gap: 4,
    },
    slots: {
      default: (state: ClusterSlotState) =>
        `${state.wrap}:${state.reversed}:${state.direction}:${state.wrapMode}:${state.gap}:${state.align}:${state.justify}:${state.state}`,
    },
  });
  const exposed = handle.exposes<ClusterExpose>();

  assert.ok(exposed.element === handle.root());
  assert.equal(exposed.wrap, true);
  assert.equal(exposed.reversed, false);
  assert.equal(exposed.direction, "row");
  assert.equal(exposed.wrapMode, "wrap");
  assert.equal(exposed.gap, "4px");
  assert.equal(exposed.align, "stretch");
  assert.equal(exposed.justify, "start");
  assert.equal(exposed.state, "clustered");
  assert.equal(handle.root().textContent, "true:false:row:wrap:4px:stretch:start:clustered");

  await handle.wrapper.setProps({
    align: "end",
    gap: "1lh",
    justify: "space-around",
    reversed: true,
    wrap: false,
  });
  assert.equal(exposed.wrap, false);
  assert.equal(exposed.reversed, true);
  assert.equal(exposed.direction, "row-reverse");
  assert.equal(exposed.wrapMode, "nowrap");
  assert.equal(exposed.gap, "1lh");
  assert.equal(exposed.align, "end");
  assert.equal(exposed.justify, "space-around");
  assert.equal(
    handle.root().textContent,
    "false:true:row-reverse:nowrap:1lh:end:space-around:clustered",
  );
  handle.unmount();
});
