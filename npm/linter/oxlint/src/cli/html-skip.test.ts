import assert from "node:assert/strict";
import test from "node:test";

import { prepareHtmlContexts } from "./html-operation.ts";
import { skippedHtml, unavailableHtml } from "./html-project-output.ts";

void test("unsupported standalone HTML hosts are skipped", () => {
  const contexts = prepareHtmlContexts(
    process.cwd(),
    ["clean.html", "pages"],
    process.execPath,
    "1.94.0",
    new Set(),
    ["clean.html", "pages"],
  );

  assert.equal(contexts.length, 1, "a host-wide skip is reported once, not per target");
  assert.equal(contexts[0].failure, undefined);
  assert.equal(
    contexts[0].skipReason,
    "Oxlint 1.94.0 is not a qualified standalone HTML host (1.78.0, 1.81.0, 1.86.0); .html/.htm files were not linted",
  );
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
