import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { loadFormatterCheckEvidenceOrRecord } from "../../legacy-tools/fixtures/glyph-formatter-evidence.mjs";

test("glyph idempotence records unusable formatter check evidence", () => {
  const reportDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-glyph-check-unusable-"));
  const project = { id: "synthetic-report" };
  const baselineUnusable: Array<{ project: string; file: string; detail: string }> = [];
  try {
    fs.writeFileSync(
      path.join(reportDir, `${project.id}-formatter.json`),
      `${JSON.stringify({
        schema: "vize.fixtureToolRun",
        version: 1,
        project: project.id,
        tool: "formatter",
        validationError: "found count 24 does not match 36 inputs",
      })}\n`,
    );
    assert.equal(loadFormatterCheckEvidenceOrRecord(project, baselineUnusable, reportDir), null);
    assert.deepEqual(baselineUnusable, [
      {
        project: project.id,
        file: "formatter-check",
        detail:
          "formatter --check evidence unavailable: invalid synthetic-report formatter --check evidence",
      },
    ]);
  } finally {
    fs.rmSync(reportDir, { recursive: true, force: true });
  }
});
