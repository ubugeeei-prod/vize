import assert from "node:assert/strict";
import { test } from "node:test";
import { h, nextTick, ref } from "@vue/runtime-core";

import { renderToString } from "./app.js";
import { treeToRenderNodes } from "./renderer.js";
import { firstChild, mountFresco, toTreeSnapshot } from "./testing/mount.js";

void test("maps host element tags onto Fresco node types", () => {
  const mounted = mountFresco(() =>
    h("box", [h("div"), h("view"), h("text"), h("span"), h("input"), h("custom-tag")]),
  );

  const root = firstChild(mounted);
  assert.equal(root.type, "box");
  assert.deepEqual(
    root.children.map((child) => child.type),
    ["box", "box", "text", "text", "input", "box"],
  );
  mounted.unmount();
});

void test("mounts a component tree as the expected output tree", () => {
  const mounted = mountFresco(() =>
    h("box", { style: { flexDirection: "column" } }, [
      h("text", { text: "title", bold: true }),
      h("box", { style: { gap: 1 } }, [h("text", { text: "body" })]),
    ]),
  );

  assert.deepEqual(toTreeSnapshot(firstChild(mounted)), {
    type: "box",
    props: { style: { flexDirection: "column" } },
    children: [
      { type: "text", props: { text: "title", bold: true } },
      {
        type: "box",
        props: { style: { gap: 1 } },
        children: [{ type: "text", props: { text: "body" } }],
      },
    ],
  });

  const [title] = firstChild(mounted).children;
  assert.equal(title?.parent, firstChild(mounted));
  mounted.unmount();
});

void test("patches props and text reactively without remounting", async () => {
  const label = ref<string | undefined>("first");
  const message = ref("hello");
  const mounted = mountFresco(() => h("text", { text: message.value, label: label.value }));

  const node = firstChild(mounted);
  assert.deepEqual(node.props, { text: "hello", label: "first" });

  message.value = "goodbye";
  label.value = undefined;
  await nextTick();

  assert.equal(firstChild(mounted), node, "the node is patched in place");
  assert.deepEqual(node.props, { text: "goodbye" }, "nullish props are removed");
  mounted.unmount();
});

void test("inserts, reorders, and removes list children through anchors", async () => {
  const items = ref(["a", "b"]);
  const mounted = mountFresco(() =>
    h(
      "box",
      items.value.map((item) => h("text", { key: item, text: item })),
    ),
  );

  const texts = () => firstChild(mounted).children.map((child) => child.props.text);
  const [a, b] = firstChild(mounted).children;
  assert.deepEqual(texts(), ["a", "b"]);

  items.value = ["b", "a"];
  await nextTick();
  assert.deepEqual(texts(), ["b", "a"], "keyed children swap order");
  assert.equal(firstChild(mounted).children[0], b, "retained node moves with its key");
  assert.equal(firstChild(mounted).children[1], a, "retained node moves with its key");

  items.value = ["c", "a", "b"];
  await nextTick();
  assert.deepEqual(texts(), ["c", "a", "b"], "prepend inserts before the anchor");

  items.value = ["c", "b"];
  await nextTick();
  assert.deepEqual(texts(), ["c", "b"], "removal detaches the middle child");
  mounted.unmount();
});

void test("emits all four discriminated render variants with canonical payload fields", () => {
  const mounted = mountFresco(() =>
    h("box", { border: "rounded", style: { flex_direction: "row", width: 40, padding_left: 1 } }, [
      h("text", {
        content: "hi",
        wrap: "end",
        color: "cyan",
        background_color: "blue",
        dim_color: true,
      }),
      h("input", {
        value: 123,
        placeholder: 456,
        focus: true,
        cursor: 3,
        mask: true,
        mask_char: "#",
        style: { min_width: 10, overflowX: "hidden" },
      }),
    ]),
  );

  const box = firstChild(mounted);
  const nodes = treeToRenderNodes(mounted.root);

  assert.equal(nodes.length, 4);
  const [rootNode, boxNode, textNode, inputNode] = nodes;
  assert.deepEqual(rootNode, {
    id: mounted.root.id,
    nodeType: "root",
    children: [box.id],
  });
  assert.deepEqual(boxNode, {
    id: box.id,
    nodeType: "box",
    style: { flexDirection: "row", width: "40", paddingLeft: 1 },
    border: "rounded",
    children: [textNode?.id, inputNode?.id],
  });
  assert.deepEqual(textNode, {
    id: box.children[0]?.id,
    nodeType: "text",
    text: "hi",
    wrap: true,
    wrapMode: "truncate-end",
    appearance: { fg: "cyan", bg: "blue", dim: true },
  });
  assert.deepEqual(inputNode, {
    id: box.children[1]?.id,
    nodeType: "input",
    value: "123",
    placeholder: "456",
    focused: true,
    cursor: 3,
    mask: true,
    maskChar: "#",
    style: { overflowX: "hidden", minWidth: "10" },
  });
  mounted.unmount();
});

void test("normalizes wrap props into native wrap modes", () => {
  const wrapOf = (wrap: unknown) => {
    const mounted = mountFresco(() => h("text", { text: "x", wrap }));
    const [node] = treeToRenderNodes(firstChild(mounted));
    mounted.unmount();
    return { wrap: node?.wrap, wrapMode: node?.wrapMode };
  };

  assert.deepEqual(wrapOf(true), { wrap: true, wrapMode: "wrap" });
  assert.deepEqual(wrapOf(false), { wrap: false, wrapMode: "none" });
  assert.deepEqual(wrapOf("end"), { wrap: true, wrapMode: "truncate-end" });
  assert.deepEqual(wrapOf("middle"), { wrap: true, wrapMode: "truncate-middle" });
  assert.deepEqual(wrapOf("truncate-start"), { wrap: false, wrapMode: "truncate-start" });
  assert.deepEqual(wrapOf(undefined), { wrap: undefined, wrapMode: undefined });
});

void test("keeps appearance aliases and drops empty style objects", () => {
  const mounted = mountFresco(() =>
    h("text", {
      text: "x",
      color: "red",
      backgroundColor: "blue",
      dimColor: true,
      blink: true,
      hidden: true,
      style: {},
    }),
  );

  const [node] = treeToRenderNodes(firstChild(mounted));
  assert.deepEqual(node, {
    id: firstChild(mounted).id,
    nodeType: "text",
    text: "x",
    appearance: { fg: "red", bg: "blue", dim: true, blink: true, hidden: true },
  });
  mounted.unmount();
});

void test("renderToString renders column and row layouts deterministically", () => {
  const column = renderToString(() =>
    h("box", { style: { flexDirection: "column" } }, [
      h("text", { text: "hello" }),
      h("text", { text: "world" }),
    ]),
  );
  assert.equal(column, "hello\nworld");

  const row = renderToString(() =>
    h("box", { style: { flexDirection: "row" } }, [
      h("text", { text: "hello " }),
      h("text", { text: "world" }),
    ]),
  );
  assert.equal(row, "hello world");
});

void test("renderToString shows input values and falls back to placeholders", () => {
  assert.equal(
    renderToString(() => h("input", { value: "typed", placeholder: "type here" })),
    "typed",
  );
  assert.equal(
    renderToString(() => h("input", { placeholder: "type here" })),
    "type here",
  );
});
