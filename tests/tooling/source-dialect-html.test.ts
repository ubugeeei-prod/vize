import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { sourceInspectedDialectCoverage } from "../differential/source-dialect-coverage.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const source = fs.readFileSync(
  path.join(
    root,
    "tests/_fixtures/differential/linter/native-inline-template-6848/Original.html.txt",
  ),
);
const selector = "original authored HTML source";
const expected = {
  state: "source-inspected",
  registryRef: null,
  evidence: [{ kind: "source-inspection", dialects: ["html-template"], selector }],
};

test("the actual original HTML input is recognized through its receipt filename", () => {
  assert.equal(source.toString("utf8"), "<div inline-template>Content</div>");
  assert.equal(
    createHash("sha256").update(source).digest("hex"),
    "eba44a30525a87d2121e058a0a63d59fcb8a6fe0ea8bf4942612e9b88836460c",
  );
  for (const filename of ["Original.html", "Original.html.txt"]) {
    assert.deepEqual(sourceInspectedDialectCoverage([{ filename, source }], selector), expected);
  }
});

test("explicit entry selection and unknown filename refusals retain their contract", () => {
  for (const filename of ["Original.txt", "Original.xml.txt", "Original.htm.txt"]) {
    assert.throws(
      () => sourceInspectedDialectCoverage([{ filename, source }], selector),
      /no authored source dialect could be inspected/,
    );
  }
  assert.throws(
    () =>
      sourceInspectedDialectCoverage(
        [{ filename: "Original.html.txt", source, entry: "script" }],
        selector,
      ),
    /no authored source dialect could be inspected/,
  );
  assert.deepEqual(
    sourceInspectedDialectCoverage(
      [{ filename: "Original.txt", source, entry: "template" }],
      selector,
    ),
    expected,
  );
  assert.deepEqual(
    sourceInspectedDialectCoverage(
      [{ filename: "Original.html.txt", source: "<template><div /></template>" }],
      selector,
    ),
    {
      ...expected,
      evidence: [{ kind: "source-inspection", dialects: ["vue3-sfc", "html-template"], selector }],
    },
  );
});
