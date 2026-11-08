import assert from "node:assert/strict";
import { test } from "vitest";
import { h, nextTick, ref } from "vue";

import { renderToString } from "./app.js";
import { treeToRenderNodes } from "./renderer.js";
import { firstChild, mountFresco, toTreeSnapshot } from "./testing/mount.js";

test("maps host element tags onto Fresco node types", () => {
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

test("mounts a component tree as the expected output tree", () => {
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

test("patches props and text reactively without remounting", async () => {
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

test("reports only host-tree mutations as native frame work", async () => {
  const message = ref("first");
  let mutations = 0;
  const mounted = mountFresco(() => h("text", { text: message.value }), {
    onMutation: () => mutations++,
  });

  assert.ok(mutations > 0, "the initial host tree requires a frame");
  const mountedMutations = mutations;
  await nextTick();
  assert.equal(mutations, mountedMutations, "an idle Vue tree schedules no frame");

  message.value = "second";
  await nextTick();
  assert.ok(mutations > mountedMutations, "a host prop change requires a frame");
  const changedMutations = mutations;

  message.value = "second";
  await nextTick();
  assert.equal(mutations, changedMutations, "an identical assignment schedules no frame");

  mounted.unmount();
  assert.ok(mutations > changedMutations, "detaching the host tree requires a frame");
});

test("inserts, reorders, and removes list children through anchors", async () => {
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

test("commits insert state and schedules native work before observer failures escape", () => {
  let mutations = 0;
  let observations = 0;

  assert.throws(
    () =>
      mountFresco(() => h("box", [h("text")]), {
        onMutation: () => mutations++,
        lifecycle: {
          onUpdate(node, kind) {
            if (kind !== "children") return;
            observations += 1;
            const child = node.children.at(-1);
            assert.ok(child);
            assert.equal(child.parent, node, "observers see the completed insertion");
            throw new Error("observer failed");
          },
        },
      }),
    /observer failed/,
  );

  assert.equal(observations, 1);
  assert.equal(mutations, 1, "native work remains scheduled when an observer throws");
});

test("keeps stable ids addressable across keyed moves", async () => {
  const items = ref(["a", "b"]);
  const mounted = mountFresco(() =>
    h(
      "box",
      items.value.map((item) => h("text", { key: item, text: item })),
    ),
  );
  const root = firstChild(mounted);
  const [a, b] = root.children;
  assert.ok(a && b);

  assert.equal(mounted.renderer.getNode(a.id), a);
  assert.equal(mounted.renderer.findDescendantById(root, b.id), b);
  assert.equal(mounted.renderer.findDescendantById(a, b.id), undefined);

  items.value = ["b", "a"];
  await nextTick();
  assert.deepEqual(
    root.children.map((node) => node.id),
    [b.id, a.id],
  );
  assert.equal(mounted.renderer.getNode(a.id), a);
  assert.equal(mounted.renderer.getNode(b.id), b);
  mounted.unmount();
});

test("reports lifecycle updates and recursively destroys lookup entries", async () => {
  const width = ref(10);
  const visible = ref(true);
  const created: number[] = [];
  const updated: Array<[number, string]> = [];
  const resized: Array<[number, unknown, unknown]> = [];
  const removed: number[] = [];
  const mounted = mountFresco(
    () =>
      h("box", { style: { width: width.value } }, [
        visible.value ? h("box", { key: "branch" }, [h("text", { text: "leaf" })]) : null,
      ]),
    {
      lifecycle: {
        onCreate: (node) => created.push(node.id),
        onUpdate: (node, kind) => updated.push([node.id, kind]),
        onResize: (node, previous, current) =>
          resized.push([node.id, previous.width, current.width]),
        onRemove: (node) => removed.push(node.id),
      },
    },
  );

  const root = firstChild(mounted);
  const branch = root.children[0];
  const leaf = branch?.children[0];
  assert.ok(branch && leaf);
  assert.ok(created.includes(root.id));
  assert.ok(created.includes(branch.id));
  assert.ok(created.includes(leaf.id));

  width.value = 20;
  await nextTick();
  assert.deepEqual(resized.at(-1), [root.id, 10, 20]);
  assert.ok(updated.some(([id, kind]) => id === root.id && kind === "props"));

  visible.value = false;
  await nextTick();
  assert.equal(mounted.renderer.getNode(branch.id), undefined);
  assert.equal(mounted.renderer.getNode(leaf.id), undefined);
  assert.deepEqual(removed.slice(-2), [leaf.id, branch.id]);
  assert.equal(mounted.renderer.destroyRecursively(branch.id), false, "disposal is idempotent");

  const rootId = root.id;
  assert.equal(mounted.renderer.destroyRecursively(rootId), true);
  assert.equal(mounted.renderer.getNode(rootId), undefined);
  mounted.unmount();
  assert.equal(
    removed.filter((id) => id === rootId).length,
    1,
    "Vue unmount cannot report an already disposed node twice",
  );
});

test("emits all four discriminated render variants with canonical payload fields", () => {
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
    wrap: false,
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

test("normalizes wrap props into native wrap modes", () => {
  const wrapOf = (wrap: unknown) => {
    const mounted = mountFresco(() => h("text", { text: "x", wrap }));
    const [node] = treeToRenderNodes(firstChild(mounted));
    mounted.unmount();
    return { wrap: node?.wrap, wrapMode: node?.wrapMode };
  };

  assert.deepEqual(wrapOf(true), { wrap: true, wrapMode: "wrap" });
  assert.deepEqual(wrapOf(false), { wrap: false, wrapMode: "none" });
  assert.deepEqual(wrapOf("end"), wrapOf("truncate-end"));
  assert.deepEqual(wrapOf("middle"), wrapOf("truncate-middle"));
  assert.deepEqual(wrapOf("truncate-start"), { wrap: false, wrapMode: "truncate-start" });
  assert.deepEqual(wrapOf("truncate-end"), { wrap: false, wrapMode: "truncate-end" });
  assert.deepEqual(wrapOf("truncate-middle"), { wrap: false, wrapMode: "truncate-middle" });
  assert.deepEqual(wrapOf("hard"), { wrap: true, wrapMode: "hard" });
  assert.deepEqual(wrapOf("invalid"), { wrap: true, wrapMode: undefined });
  assert.deepEqual(wrapOf(undefined), { wrap: undefined, wrapMode: undefined });
});

test("keeps appearance aliases and drops empty style objects", () => {
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

test("renderToString renders column and row layouts deterministically", () => {
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

test("renderToString shows input values and falls back to placeholders", () => {
  assert.equal(
    renderToString(() => h("input", { value: "typed", placeholder: "type here" })),
    "typed",
  );
  assert.equal(
    renderToString(() => h("input", { placeholder: "type here" })),
    "type here",
  );
});
