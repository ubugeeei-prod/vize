import assert from "node:assert/strict";
import { h } from "vue";
import { test } from "vitest";

import { firstChild, mountComponent } from "../testing/index.js";
import Divider from "./FrescoDivider.vue";
import HStack from "./FrescoHStack.vue";
import Stack from "./FrescoStack.vue";
import VStack from "./FrescoVStack.vue";

test("Divider preserves horizontal, titled, and vertical rendering", () => {
  const plain = mountComponent(Divider, { char: "=" });
  assert.equal(firstChild(plain).text, "=".repeat(40));
  assert.deepEqual(firstChild(plain).props, { fg: "gray" });
  plain.unmount();

  const titled = mountComponent(Divider, {
    char: "-",
    title: "Summary",
    fg: "white",
    titleFg: "cyan",
  });
  const titleRoot = firstChild(titled);
  assert.deepEqual(titleRoot.props.style, { flexDirection: "row", alignItems: "center" });
  assert.deepEqual(
    titleRoot.children.map((node) => ({ text: node.text, props: node.props })),
    [
      { text: "---", props: { fg: "white" } },
      { text: " Summary ", props: { fg: "cyan", bold: true } },
      { text: "---", props: { fg: "white" } },
    ],
  );
  titled.unmount();

  const vertical = mountComponent(Divider, { direction: "vertical", fg: "blue" });
  assert.equal(firstChild(vertical).text, "│");
  assert.deepEqual(firstChild(vertical).props, { fg: "blue" });
  vertical.unmount();
});

test("Stack maps its typed layout props and forwards slot children", () => {
  const mounted = mountComponent(
    Stack,
    {
      direction: "horizontal",
      gap: 2,
      align: "center",
      justify: "between",
      wrap: true,
    },
    () => [h("text", { text: "left" }), h("text", { text: "right" })],
  );
  const root = firstChild(mounted);

  assert.deepEqual(root.props.style, {
    flexDirection: "row",
    gap: 2,
    alignItems: "center",
    justifyContent: "space-between",
    flexWrap: "wrap",
  });
  assert.deepEqual(
    root.children.flatMap((node) => (typeof node.props.text === "string" ? [node.props.text] : [])),
    ["left", "right"],
  );
  mounted.unmount();
});

test("HStack and VStack preserve their convenience defaults", () => {
  const horizontal = mountComponent(HStack);
  assert.deepEqual(firstChild(horizontal).props.style, {
    flexDirection: "row",
    gap: 1,
    alignItems: "stretch",
    justifyContent: "flex-start",
    flexWrap: "nowrap",
  });
  horizontal.unmount();

  const vertical = mountComponent(VStack, { gap: 3, align: "end", justify: "around" });
  assert.deepEqual(firstChild(vertical).props.style, {
    flexDirection: "column",
    gap: 3,
    alignItems: "flex-end",
    justifyContent: "space-around",
    flexWrap: "nowrap",
  });
  vertical.unmount();
});
