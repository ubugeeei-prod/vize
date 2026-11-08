import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { h, ref } from "vue";

import { TextInput } from "../../npm/fresco/src/components/TextInput.js";
import { getByRole, getByText, renderTui } from "../../npm/fresco/src/testing/index.js";

const read = (relativePath: string) =>
  readFileSync(new URL(`../../${relativePath}`, import.meta.url), "utf8");

void test("the public harness runs under the plain Node test runner", async () => {
  const value = ref("");
  const rendered = renderTui(() =>
    h("box", { style: { flexDirection: "column" } }, [
      h("box", { "aria-role": "button", "aria-label": "Save draft" }, [
        h("text", { text: "Save" }),
      ]),
      h(TextInput, {
        modelValue: value.value,
        focused: true,
        "onUpdate:modelValue": (next: string) => {
          value.value = next;
        },
      }),
    ]),
  );

  assert.equal(getByRole(rendered.root, "button", { name: "Save draft" }).type, "box");
  assert.equal(getByText(rendered.root, "Save").type, "text");

  const frame = await rendered.input.text("node");
  assert.equal(value.value, "node");
  assert.equal(frame.output, "Save\nnode");
  assert.equal(frame.protocolNodes[0]?.nodeType, "root");
  assert.ok(frame.protocolNodes.some((node) => node.nodeType === "input"));
  rendered.unmount();
});

void test("the testing guide stays on the supported package boundary", () => {
  const guide = read("docs/fresco/testing.md");
  const matrix = read("docs/fresco/compatibility.md");

  assert.match(guide, /from "@vizejs\/fresco\/testing"/u);
  assert.match(guide, /node --import tsx --test/u);
  assert.match(guide, /import `test` from\s+`vitest`/u);
  assert.doesNotMatch(guide, /@vizejs\/fresco\/src\//u);
  assert.match(matrix, /\.\/testing\.md/u);
});
