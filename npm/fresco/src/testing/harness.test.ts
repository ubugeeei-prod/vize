import assert from "node:assert/strict";
import { test } from "node:test";
import { defineComponent, h, nextTick, ref } from "@vue/runtime-core";

import { Static, TextInput } from "../components/index.js";
import { useWindowSize } from "../composables/index.js";
import {
  getByRole,
  getByTestId,
  getByText,
  queryAllByRole,
  queryAllByTestId,
  queryAllByText,
  renderTui,
} from "./index.js";

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

void test("semantic queries find role, name, state, text, and test-id nodes", () => {
  const rendered = renderTui(() =>
    h("box", { style: { flexDirection: "column" } }, [
      h(
        "box",
        {
          "aria-role": "button",
          "aria-label": "Save changes",
          "aria-state": { disabled: true },
          "test-id": "save-action",
        },
        [h("text", { text: "Ignored label" })],
      ),
      h("box", { "aria-role": "button" }, [h("text", { text: "Cancel" })]),
      h("input", { "aria-role": "textbox", value: "draft" }),
      h("text", { "data-testid": "status-line", text: "Status: ready" }),
    ]),
  );

  assert.equal(queryAllByRole(rendered.root, "button").length, 2);
  assert.equal(
    getByRole(rendered.root, "button", { name: "Save changes" }).props["aria-role"],
    "button",
  );
  assert.equal(
    getByRole(rendered.root, "button", { state: { disabled: true } }).props["aria-label"],
    "Save changes",
  );
  assert.equal(
    getByRole(rendered.root, "button", { name: "Cancel" }).children[0]?.props.text,
    "Cancel",
  );
  assert.equal(getByText(rendered.root, "draft").type, "input");
  assert.equal(queryAllByText(rendered.root, /ready/u).length, 1);
  assert.equal(getByTestId(rendered.root, "save-action").props["aria-label"], "Save changes");
  assert.equal(queryAllByTestId(rendered.root, "status-line")[0]?.props.text, "Status: ready");
  assert.throws(() => getByTestId(rendered.root, "missing"), /Unable to find Fresco node/);
  assert.throws(() => getByRole(rendered.root, "checkbox"), /Unable to find Fresco node/);
  assert.throws(() => getByRole(rendered.root, "button"), /Found 2 Fresco nodes/);
  rendered.unmount();
});
