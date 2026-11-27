import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import { parseUnifiedDiff } from "../diff-parse.ts";
import DiffView from "../diff-view.vue";
import { mountInteraction } from "../../../testing/mount.ts";

const patch = [
  "diff --git a/readme.md b/readme.md",
  "index 111..222 100644",
  "--- a/readme.md",
  "+++ b/readme.md",
  "@@ -1,2 +1,2 @@",
  " keep",
  "-old",
  "+new",
  "",
].join("\n");

test("parses a unified diff without treating headers as edits", () => {
  assert.deepEqual(
    parseUnifiedDiff(patch).map((line) => [line.kind, line.text]),
    [
      ["meta", "diff --git a/readme.md b/readme.md"],
      ["meta", "index 111..222 100644"],
      ["meta", "--- a/readme.md"],
      ["meta", "+++ b/readme.md"],
      ["meta", "@@ -1,2 +1,2 @@"],
      ["context", "keep"],
      ["remove", "old"],
      ["add", "new"],
    ],
  );
});

test("renders a patch as a named table", () => {
  const handle = mountInteraction(DiffView, { props: { label: "readme.md", patch } });
  const table = handle.getByRole("table", { name: "readme.md" });

  assert.equal(table.getAttribute("data-vize-ui"), "diff-view");
  assert.equal(
    handle.getByRole("cell", { name: "old" }).closest("tr")?.getAttribute("data-kind"),
    "remove",
  );
  assert.equal(
    handle.getByRole("cell", { name: "new" }).closest("tr")?.getAttribute("data-kind"),
    "add",
  );
  assert.equal(
    handle.getByRole("cell", { name: "keep" }).closest("tr")?.getAttribute("data-kind"),
    "context",
  );
  assert.equal(
    handle.getByRole("cell", { name: "--- a/readme.md" }).closest("tr")?.getAttribute("data-kind"),
    "meta",
  );
  handle.unmount();
});

test("prefers structured lines over a patch", () => {
  const handle = mountInteraction(DiffView, {
    props: {
      label: "notes",
      patch,
      lines: [{ kind: "add", text: "only" }],
    },
  });

  assert.equal(handle.getByRole("cell", { name: "only" }).textContent, "only");
  assert.equal(handle.queryByRole("cell", { name: "old" }), null);
  handle.unmount();
});

test("renders an empty patch as a table without cells", () => {
  const handle = mountInteraction(DiffView, { props: { label: "Empty", patch: "" } });

  assert.equal(handle.getByRole("table", { name: "Empty" }).tagName, "TABLE");
  assert.equal(handle.queryByRole("cell"), null);
  handle.unmount();
});
