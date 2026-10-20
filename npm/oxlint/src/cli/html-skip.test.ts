import assert from "node:assert/strict";
import test from "node:test";

import { prepareHtmlContext } from "./html-operation.ts";
import { skippedHtml, unavailableHtml } from "./html-project-output.ts";

void test("unsupported standalone HTML hosts are skipped", () => {
  const context = prepareHtmlContext(
    process.cwd(),
    ["clean.html"],
    process.execPath,
    "1.75.0",
    new Set(),
  );

  assert.equal(context.failure, undefined);
  assert.match(context.skipReason ?? "", /qualified Oxlint/u);
});

void test("standalone HTML skips preserve the host exit status", () => {
  const clean = {
    stdout: "",
    stderr: "host warning\n",
    status: 0,
  };
  const skipped = skippedHtml(clean, new Error("unsupported host"));

  assert.equal(skipped.status, 0);
  assert.match(skipped.stderr, /Skipped standalone HTML lint: unsupported host/u);
  assert.equal(skipped.rawStderr?.toString(), skipped.stderr);

  assert.equal(skippedHtml({ ...clean, status: 1 }, "unsupported host").status, 1);
  assert.equal(unavailableHtml(clean, "runtime failure").status, 1);
});
