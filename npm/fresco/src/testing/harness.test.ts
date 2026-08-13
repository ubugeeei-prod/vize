import assert from "node:assert/strict";
import { test } from "node:test";
import { defineComponent, h, nextTick, ref } from "@vue/runtime-core";

import { Static, TextInput } from "../components/index.js";
import { useWindowSize } from "../composables/index.js";
import { renderTui } from "./index.js";

void test("renderTui records initial and explicit frame snapshots", async () => {
  const label = ref("ready");
  const rendered = renderTui(() =>
    h("box", { style: { flexDirection: "column" } }, [h("text", { text: label.value })]),
  );

  assert.equal(rendered.lastFrame(), "ready");
  assert.deepEqual(rendered.frameSnapshot(), {
    output: "ready",
    tree: {
      type: "root",
      children: [
        {
          type: "box",
          props: { style: { flexDirection: "column" } },
          children: [{ type: "text", props: { text: "ready" } }],
        },
      ],
    },
  });

  label.value = "done";
  await nextTick();
  assert.equal(rendered.captureFrame().output, "done");
  assert.deepEqual(rendered.frames, ["ready", "done"]);
  rendered.unmount();
});

void test("frame snapshots preserve Static output above the live frame", async () => {
  const items = ref(["one"]);
  const rendered = renderTui(() =>
    h("box", { style: { flexDirection: "column" } }, [
      h(
        Static,
        { items: items.value },
        { default: ({ item }: { item: string }) => h("text", { text: item }) },
      ),
      h("text", { text: "live" }),
    ]),
  );

  assert.equal(rendered.lastFrame(), "one\nlive");

  items.value = ["one", "two"];
  await nextTick();
  assert.equal(rendered.captureFrame().output, "one\ntwo\nlive");
  rendered.unmount();
});

void test("input driver records TextInput key and paste frames", async () => {
  const value = ref("");
  const submitted: string[] = [];
  const rendered = renderTui(() =>
    h(TextInput, {
      modelValue: value.value,
      focused: true,
      "onUpdate:modelValue": (next: string) => {
        value.value = next;
      },
      onSubmit: (text: string) => submitted.push(text),
    }),
  );

  assert.equal(rendered.lastFrame(), "");

  await rendered.input.text("ab");
  assert.equal(value.value, "ab");
  assert.equal(rendered.lastFrame(), "ab");

  await rendered.input.paste("cd");
  assert.equal(value.value, "abcd");
  assert.equal(rendered.lastFrame(), "abcd");

  await rendered.input.key({ key: "enter" });
  assert.deepEqual(submitted, ["abcd"]);
  assert.deepEqual(rendered.frames, ["", "ab", "abcd", "abcd"]);
  rendered.unmount();
});

void test("resize input updates the provided app dimensions", async () => {
  const SizeProbe = defineComponent({
    name: "SizeProbe",
    setup() {
      const size = useWindowSize();
      return () => h("text", { text: `${size.columns}x${size.rows}` });
    },
  });
  const rendered = renderTui(() => h(SizeProbe), { width: 20, height: 8 });

  assert.equal(rendered.lastFrame(), "20x8");
  await rendered.input.resize(120, 40);
  assert.equal(rendered.lastFrame(), "120x40");
  rendered.unmount();
});
