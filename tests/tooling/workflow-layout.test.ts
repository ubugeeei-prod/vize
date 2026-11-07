import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";

import {
  readWorkflowEntries,
  renderWorkflowCatalog,
} from "../../tools/commands/ci/workflow-catalog.ts";
import { parseTapCounts } from "../../tools/support/github/docs-preview-readiness.ts";
import { readRepoFile, root } from "./support/github-workflows.ts";

const embeddedProgram =
  /(?:node|python\d*)\s+(?:--input-type=module\s+)?-\s+<<|(?:node|python\d*)\s+--input-type=module\s+<</g;

test("the flat GitHub workflow directory has a generated navigation catalog", () => {
  const directory = path.join(root, ".github/workflows");
  const expected = renderWorkflowCatalog(readWorkflowEntries(directory));
  assert.equal(readRepoFile(".github/workflows/README.md"), expected);
});

test("the extracted docs readiness driver preserves TAP totals", () => {
  assert.deepEqual(
    parseTapCounts("# tests 8\n# pass 8\n# fail 0\n# cancelled 0\n# skipped 0\n# todo 0\n"),
    { tests: 8, pass: 8, fail: 0, cancelled: 0, skipped: 0, todo: 0 },
  );
});

test("embedded workflow programs only shrink", () => {
  const debt = JSON.parse(readRepoFile("tools/config/workflow-inline-script-debt.json")) as Record<
    string,
    number
  >;
  const current: Record<string, number> = {};
  for (const directory of [".github/workflows", ".github/actions"]) {
    const absolute = path.join(root, directory);
    for (const entry of fs.readdirSync(absolute, { recursive: true })) {
      if (typeof entry !== "string" || !/\.ya?ml$/.test(entry)) continue;
      const relative = path.join(directory, entry);
      const count = [...readRepoFile(relative).matchAll(embeddedProgram)].length;
      if (count > 0) current[relative] = count;
    }
  }
  for (const [file, count] of Object.entries(current)) {
    assert.ok(file in debt, `${file} introduced an embedded program; use a typed tools/ driver`);
    assert.ok(
      count <= debt[file]!,
      `${file} increased embedded programs from ${debt[file]} to ${count}`,
    );
  }
});
