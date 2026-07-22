import assert from "node:assert/strict";
import { test } from "node:test";
import { h } from "@vue/runtime-core";

import { firstChild, mountComponent, toTreeSnapshot } from "../testing/mount.js";
import { Box } from "./Box.js";

// Box declares Boolean props, so Vue defaults them to false and the render
// function forwards them onto the host node; they are inert downstream
// (treeToRenderNodes drops falsy appearance) but part of the output tree.
const boxDefaults = { borderDimColor: false, "aria-hidden": false };

void test("renders a box node and forwards children", () => {
  const mounted = mountComponent(Box, {}, () => h("text", { text: "inside" }));

  assert.deepEqual(toTreeSnapshot(firstChild(mounted)), {
    type: "box",
    props: { style: {}, ...boxDefaults },
    children: [{ type: "text", props: { text: "inside" } }],
  });
  mounted.unmount();
});

void test("maps layout props onto the style object", () => {
  const mounted = mountComponent(Box, {
    flexDirection: "column",
    justifyContent: "center",
    alignItems: "flex-end",
    flexGrow: 1,
    flexShrink: 0,
    width: 40,
    height: "50%",
    minWidth: 5,
    gap: 2,
    overflow: "hidden",
  });

  assert.deepEqual(firstChild(mounted).props.style, {
    flexDirection: "column",
    justifyContent: "center",
    alignItems: "flex-end",
    flexGrow: 1,
    flexShrink: 0,
    width: "40",
    height: "50%",
    minWidth: "5",
    gap: 2,
    overflow: "hidden",
  });
  mounted.unmount();
});

void test("expands padding and margin axis shorthands per side", () => {
  const mounted = mountComponent(Box, {
    padding: 1,
    paddingX: 2,
    paddingTop: 3,
    marginY: 4,
    marginLeft: 5,
  });

  assert.deepEqual(firstChild(mounted).props.style, {
    padding: 1,
    paddingTop: 3,
    paddingRight: 2,
    // paddingBottom stays unset: per-side keys materialize only when that
    // side or its axis shorthand is given; the base padding covers the rest.
    paddingLeft: 2,
    marginTop: 4,
    marginBottom: 4,
    marginLeft: 5,
  });
  mounted.unmount();
});

void test("normalizes border style names and prefers the Ink alias", () => {
  const withBorder = (props: Record<string, unknown>) => {
    const mounted = mountComponent(Box, props);
    const border = firstChild(mounted).props.border;
    mounted.unmount();
    return border;
  };

  assert.equal(withBorder({ border: "single" }), "single");
  assert.equal(withBorder({ border: "round" }), "rounded");
  assert.equal(withBorder({ border: "bold" }), "heavy");
  assert.equal(withBorder({ borderStyle: "double", border: "single" }), "double");
  assert.equal(withBorder({}), undefined);
});

void test("resolves color aliases with Fresco names winning", () => {
  const mounted = mountComponent(Box, {
    fg: "cyan",
    color: "red",
    backgroundColor: "blue",
    borderColor: "green",
  });

  const props = firstChild(mounted).props;
  assert.equal(props.fg, "cyan");
  assert.equal(props.bg, "blue");
  assert.equal(props.borderColor, "green");
  mounted.unmount();
});

void test("hides aria-hidden subtrees only in screen reader mode", () => {
  const visible = mountComponent(Box, { "aria-hidden": true }, () => h("text", { text: "x" }));
  assert.equal(firstChild(visible).type, "box");
  assert.equal(firstChild(visible).props["aria-hidden"], true);
  visible.unmount();

  const hidden = mountComponent(Box, { "aria-hidden": true }, () => h("text", { text: "x" }), {
    screenReader: true,
  });
  assert.deepEqual(toTreeSnapshot(firstChild(hidden)), { type: "text" });
  hidden.unmount();
});

void test("replaces children with the aria-label in screen reader mode", () => {
  const mounted = mountComponent(
    Box,
    { "aria-label": "status panel" },
    () => h("text", { text: "42%" }),
    { screenReader: true },
  );

  assert.deepEqual(toTreeSnapshot(firstChild(mounted)), {
    type: "box",
    props: { style: {}, "aria-label": "status panel", ...boxDefaults },
    children: [{ type: "text", props: { text: "status panel" } }],
  });
  mounted.unmount();
});
