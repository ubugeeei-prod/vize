import assert from "node:assert/strict";
import { mkdir, mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { test } from "node:test";

import { formatSfcLintResults, lintSfcFiles } from "./lint-sfc.ts";

void test("discovers overlapping SFC roots once in deterministic order", async (context) => {
  const root = await mkdtemp(path.join(tmpdir(), "vize-ui-sfc-"));
  context.after(() => rm(root, { force: true, recursive: true }));
  await mkdir(path.join(root, "a"));
  await mkdir(path.join(root, "z"));
  await writeFile(path.join(root, "z", "Second.vue"), "<template><p>Second</p></template>");
  await writeFile(path.join(root, "a", "First.vue"), "<template><p>First</p></template>");

  const requestedFiles: string[] = [];
  const results = await lintSfcFiles(
    (_, options) => {
      requestedFiles.push(options.filename);
      assert.equal(options.preset, "opinionated");
      assert.equal(options.typeAware, true);
      assert.equal(options.helpLevel, "short");
      return { diagnostics: [] };
    },
    [root, root],
  );

  assert.deepEqual(
    requestedFiles.map((filename) => path.basename(filename)),
    ["First.vue", "Second.vue"],
  );
  assert.equal(results.length, 2);
});

void test("formats source locations and preserves warning severity", () => {
  const report = formatSfcLintResults([
    {
      filename: "src/Control.vue",
      diagnostics: [
        {
          rule: "a11y/control-name",
          severity: "warning",
          message: "Control requires an accessible name",
          location: { start: { line: 4, column: 7 } },
        },
      ],
    },
  ]);

  assert.equal(
    report,
    "src/Control.vue:4:7 warning a11y/control-name Control requires an accessible name",
  );
});
