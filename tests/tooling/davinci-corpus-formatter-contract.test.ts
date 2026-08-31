// Davinci corpus baseline hashing for the formatter lane: stderr ordering is
// filed nondeterminism, while formatterCheck is the deterministic evidence.

import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import {
  EXCLUDED_FIELDS,
  HASHED_FIELDS,
  SURFACES,
} from "../../legacy-tools/davinci/lib/corpus-baseline-contract.mjs";
import { reduceShards } from "../../legacy-tools/davinci/lib/corpus-baseline-run.mjs";

const projectId = "synthetic-formatter";

function formatterPayload(overrides: Record<string, unknown> = {}) {
  return {
    schema: "vize.fixtureToolRun",
    version: 1,
    project: projectId,
    tool: "formatter",
    status: "findings",
    exitCode: 1,
    stdout: "",
    stderr: "Would reformat: src/App.vue\nWould reformat: src/Card.vue\n",
    spawnError: null,
    parseError: null,
    validationError: null,
    formatterCheck: {
      checkedFileCount: 2,
      changedFileCount: 2,
      unchangedFileCount: 0,
      changedPathsSha256: "paths-a",
    },
    ...overrides,
  };
}

function writeFormatterShard(payload: Record<string, unknown>) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "davinci-formatter-corpus-"));
  fs.writeFileSync(
    path.join(dir, "summary.json"),
    `${JSON.stringify({
      summary: {
        failedRuns: 0,
        missingFixtureRuns: 0,
        plannedRuns: 0,
        okRuns: 0,
        findingsRuns: 1,
        runCount: 1,
      },
      projects: [{ id: projectId, runs: [{ tool: "formatter", fileCount: 2 }] }],
    })}\n`,
  );
  fs.writeFileSync(path.join(dir, `${projectId}-formatter.json`), `${JSON.stringify(payload)}\n`);
  return dir;
}

function reduceFormatter(payload: Record<string, unknown>) {
  const dir = writeFormatterShard(payload);
  try {
    return reduceShards([dir], ["formatter"])[0];
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
}

test("formatter corpus baseline hashes formatterCheck and excludes stderr ordering", () => {
  assert.ok(SURFACES.includes("formatter"), "formatter must be a corpus surface");
  assert.deepEqual(HASHED_FIELDS.formatter, ["exitCode", "formatterCheck", "stdout"]);
  assert.deepEqual(EXCLUDED_FIELDS.formatter, ["stderr"]);

  const first = reduceFormatter(formatterPayload());
  const reorderedStderr = reduceFormatter(
    formatterPayload({
      stderr: "Would reformat: src/Card.vue\nWould reformat: src/App.vue\n",
    }),
  );
  assert.deepEqual(reorderedStderr, first);

  const changedEvidence = reduceFormatter(
    formatterPayload({
      formatterCheck: {
        checkedFileCount: 2,
        changedFileCount: 2,
        unchangedFileCount: 0,
        changedPathsSha256: "paths-b",
      },
    }),
  );
  assert.equal(changedEvidence.file_count, 2);
  assert.notEqual(changedEvidence.content_hash, first.content_hash);
});

test("formatter corpus baseline requires formatterCheck evidence", () => {
  const payload = formatterPayload();
  delete payload.formatterCheck;
  assert.throws(
    () => reduceFormatter(payload),
    /synthetic-formatter\/formatter payload has no formatterCheck field/u,
  );
});
