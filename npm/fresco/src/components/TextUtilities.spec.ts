import assert from "node:assert/strict";
import { h } from "vue";
import { test } from "vitest";

import { firstChild, mountComponent } from "../testing/index.js";
import Newline from "./FrescoNewline.vue";
import Spacer from "./FrescoSpacer.vue";
import Transform from "./FrescoTransform.vue";

test("Spacer renders flexible empty space", () => {
  const mounted = mountComponent(Spacer);

  assert.deepEqual(firstChild(mounted).props.style, {
    flexGrow: 1,
    flexShrink: 1,
  });
  mounted.unmount();
});

test("Newline renders a normalized number of newline characters", () => {
  for (const [count, expected] of [
    [undefined, "\n"],
    [3, "\n\n\n"],
    [-1, ""],
  ] as const) {
    const mounted = mountComponent(Newline, count === undefined ? {} : { count });
    assert.equal(firstChild(mounted).props.text, expected);
    mounted.unmount();
  }
});

test("Transform maps each rendered line through its typed callback", () => {
  const mounted = mountComponent(
    Transform,
    { transform: (line: string, index: number) => `${index}:${line.toUpperCase()}` },
    () => h("text", { text: "alpha\nbeta" }),
  );

  assert.equal(firstChild(mounted).props.text, "0:ALPHA\n1:BETA");
  mounted.unmount();
});

test("Transform uses its accessibility label in screen-reader mode", () => {
  const mounted = mountComponent(
    Transform,
    {
      accessibilityLabel: "Accessible status",
      transform: (line: string) => line.toUpperCase(),
    },
    () => h("text", { text: "visual status" }),
    { screenReader: true },
  );

  assert.equal(firstChild(mounted).props.text, "Accessible status");
  assert.equal(firstChild(mounted).props["aria-label"], "Accessible status");
  mounted.unmount();
});
