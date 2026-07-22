import assert from "node:assert/strict";
import { test } from "node:test";

import { firstChild, mountComponent, toTreeSnapshot } from "../testing/mount.js";
import { ErrorText, InfoText, MutedText, SuccessText, Text, WarningText } from "./Text.js";

// Text declares Boolean props, so Vue defaults them to false and the render
// function forwards them onto the host node; they are inert downstream
// (treeToRenderNodes drops falsy appearance) but part of the output tree.
const textDefaults = {
  wrap: false,
  bold: false,
  dim: false,
  italic: false,
  underline: false,
  strikethrough: false,
  inverse: false,
  "aria-hidden": false,
};

void test("renders the content prop as a single text node", () => {
  const mounted = mountComponent(Text, { content: "hello" });

  assert.deepEqual(toTreeSnapshot(firstChild(mounted)), {
    type: "text",
    props: { text: "hello", ...textDefaults },
  });
  mounted.unmount();
});

void test("stringifies slot children and prefers content over the slot", () => {
  const slotted = mountComponent(Text, {}, () => ["count: ", 42]);
  assert.equal(firstChild(slotted).props.text, "count: 42");
  slotted.unmount();

  const both = mountComponent(Text, { content: "wins" }, () => "loses");
  assert.equal(firstChild(both).props.text, "wins");
  both.unmount();
});

void test("passes styling flags and wrap mode through to the node", () => {
  const mounted = mountComponent(Text, {
    content: "styled",
    bold: true,
    italic: true,
    underline: true,
    strikethrough: true,
    inverse: true,
    wrap: "truncate-middle",
  });

  assert.deepEqual(toTreeSnapshot(firstChild(mounted)), {
    type: "text",
    props: {
      ...textDefaults,
      text: "styled",
      bold: true,
      italic: true,
      underline: true,
      strikethrough: true,
      inverse: true,
      wrap: "truncate-middle",
    },
  });
  mounted.unmount();
});

void test("resolves color and dim aliases", () => {
  const propsOf = (props: Record<string, unknown>) => {
    const mounted = mountComponent(Text, { content: "x", ...props });
    const node = firstChild(mounted).props;
    mounted.unmount();
    return { fg: node.fg, bg: node.bg, dim: node.dim };
  };

  assert.deepEqual(propsOf({ color: "red", backgroundColor: "blue" }), {
    fg: "red",
    bg: "blue",
    dim: false,
  });
  assert.deepEqual(propsOf({ fg: "cyan", color: "red", bg: "black", backgroundColor: "blue" }), {
    fg: "cyan",
    bg: "black",
    dim: false,
  });
  assert.deepEqual(propsOf({ dimColor: true }), { fg: undefined, bg: undefined, dim: true });
});

void test("honors aria props in screen reader mode", () => {
  const hidden = mountComponent(Text, { content: "x", "aria-hidden": true }, undefined, {
    screenReader: true,
  });
  assert.deepEqual(toTreeSnapshot(firstChild(hidden)), { type: "text" });
  hidden.unmount();

  const labeled = mountComponent(Text, { content: "42%", "aria-label": "progress" }, undefined, {
    screenReader: true,
  });
  assert.equal(firstChild(labeled).props.text, "progress");
  labeled.unmount();

  const plain = mountComponent(Text, { content: "42%", "aria-label": "progress" });
  assert.equal(firstChild(plain).props.text, "42%");
  plain.unmount();
});

void test("convenience variants preset color and dim styling", () => {
  const nodeFor = (component: typeof ErrorText) => {
    const mounted = mountComponent(component, { content: "msg" });
    const { fg, dim, text } = firstChild(mounted).props;
    mounted.unmount();
    return { fg, dim, text };
  };

  assert.deepEqual(nodeFor(ErrorText), { fg: "red", dim: false, text: "msg" });
  assert.deepEqual(nodeFor(WarningText), { fg: "yellow", dim: false, text: "msg" });
  assert.deepEqual(nodeFor(SuccessText), { fg: "green", dim: false, text: "msg" });
  assert.deepEqual(nodeFor(InfoText), { fg: "blue", dim: false, text: "msg" });
  assert.deepEqual(nodeFor(MutedText), { fg: undefined, dim: true, text: "msg" });
});
