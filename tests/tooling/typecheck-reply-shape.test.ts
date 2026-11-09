import assert from "node:assert/strict";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";

import {
  familyShapes,
  renderReplyShape,
  shapeSidecar,
} from "../../tools/benchmarks/scripts/typecheck-reply-shape.ts";

const TEXT = [
  "// ========== Template Scope (inherits from setup) ==========",
  "  const __vize_prop_check_0_rows: __Child_0_prop_rows = rows;",
  "  void ((...__vize_args: Parameters<__Child_1_select_listener>) => {",
  "  void (count);",
  "",
].join("\n");

void test("a reply's tuples become starts; ignore gaps and authored directives split", () => {
  assert.deepEqual(
    shapeSidecar({
      text: TEXT,
      mappings: [
        [10, 3, 0, 3, 0],
        [70, 4, 4, 4, 1, 0],
      ],
      diagnosticDirectives: {
        directives: [
          [0, 0, 5, 9, 0],
          [3, 2, 20, 30, 1, 0],
          [3, 2, 40, 45, 0, 0],
        ],
        unusedExpectDirectiveDiagnostics: [],
      },
    }),
    { spans: [10, 70], ignoreGaps: [5], directives: [20, 40] },
  );
  assert.deepEqual(shapeSidecar({ text: "" }), { spans: [], ignoreGaps: [], directives: [] });
});

void test("tuples count toward the family of the statement they start in", (t) => {
  const dir = mkdtempSync(join(tmpdir(), "reply-shape-"));
  t.after(() => rmSync(dir, { recursive: true, force: true }));
  mkdirSync(join(dir, "src"));
  writeFileSync(join(dir, "src", "A.vue.tsx"), TEXT);
  const line = (index: number) =>
    TEXT.split("\n")
      .slice(0, index)
      .reduce((offset, text) => offset + Buffer.byteLength(text) + 1, 0);
  writeFileSync(
    join(dir, "src", "A.vue.tsx.shape.json"),
    JSON.stringify({
      spans: [line(1) + 40, line(2) + 30, line(3) + 8],
      ignoreGaps: [line(1) - 1, line(2)],
      directives: [],
    }),
  );
  // A sidecar without its text is skipped.
  writeFileSync(
    join(dir, "B.vue.tsx.shape.json"),
    JSON.stringify({ spans: [0], ignoreGaps: [], directives: [] }),
  );
  const shapes = familyShapes(dir);
  assert.deepEqual(Object.fromEntries(shapes), {
    "template/prop-check": { spans: 1, ignoreGaps: 1, directives: 0 },
    "template/listener": { spans: 1, ignoreGaps: 1, directives: 0 },
    "template/template-expression": { spans: 1, ignoreGaps: 0, directives: 0 },
  });
  const head = new Map(shapes);
  head.set("template/listener", { spans: 4, ignoreGaps: 3, directives: 0 });
  assert.equal(
    renderReplyShape("demo", shapes, head),
    [
      "#### demo: reply tuples by family (tsc probe)",
      "",
      "| family | spans base | spans head | Δ | ignore gaps base | ignore gaps head | Δ | directives head |",
      "| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |",
      "| template/listener | 1 | 4 | +3 | 1 | 3 | +2 | 0 |",
      "| template/prop-check | 1 | 1 | 0 | 1 | 1 | 0 | 0 |",
      "| template/template-expression | 1 | 1 | 0 | 0 | 0 | 0 | 0 |",
      "",
    ].join("\n"),
  );
  assert.equal(familyShapes(join(dir, "missing")).size, 0);
});
