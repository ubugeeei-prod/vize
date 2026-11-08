import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const isHarnessChild = process.env.VIZE_FRESCO_NODE_HARNESS_CHILD === "1";

const read = (relativePath: string) =>
  readFileSync(new URL(`../../${relativePath}`, import.meta.url), "utf8");

if (isHarnessChild) {
  void test("the public harness runs under the plain Node test runner", async () => {
    const [{ h, ref }, { TextInput }, { getByRole, getByText, renderTui }] = await Promise.all([
      import("vue"),
      import("../../npm/fresco/src/components/TextInput.js"),
      import("../../npm/fresco/src/testing/index.js"),
    ]);
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
} else {
  void test("the documented Node entrypoint runs the public harness", () => {
    const result = spawnSync(
      process.execPath,
      ["--import", "tsx", "--test", fileURLToPath(import.meta.url)],
      {
        cwd: fileURLToPath(new URL("../../npm/fresco/", import.meta.url)),
        encoding: "utf8",
        env: { ...process.env, VIZE_FRESCO_NODE_HARNESS_CHILD: "1" },
      },
    );

    assert.equal(result.status, 0, result.stderr || result.stdout);
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
}
