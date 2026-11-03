import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { parse } from "yaml";
import { deletionReadinessAdapters } from "../differential/deletion-readiness-adapters.ts";

void test("Matrix records the blocked preflight without registering linter deletion credit", () => {
  const workflow = parse(
      fs.readFileSync(
        new URL("../../.github/workflows/real-project-matrix.yml", import.meta.url),
        "utf8",
      ),
    ) as { jobs: Record<string, { steps: Array<Record<string, unknown>> }> },
    steps = workflow.jobs["real-project-matrix"].steps,
    divergenceIndex = steps.findIndex(
      (step) => step.name === "Measure real-project lint divergence",
    ),
    verifierIndex = steps.findIndex(
      (step) => step.name === "Record linter deletion-readiness evidence",
    ),
    uploadIndex = steps.findIndex((step) => step.name === "Upload shard report");
  assert(divergenceIndex >= 0, "lint divergence step is missing");
  assert(verifierIndex > divergenceIndex, "recorder must run after lint divergence");
  assert(uploadIndex > verifierIndex, "artifact upload must run after the recorder");
  const recorder = steps[verifierIndex];
  assert.equal(recorder.if, "${{ !cancelled() }}");
  const upload = steps[uploadIndex];
  assert.equal(
    typeof upload.uses === "string" && upload.uses.startsWith("actions/upload-artifact@"),
    true,
  );
  assert.equal((upload.with as Record<string, unknown>).path, "${{ env.FIXTURE_REPORT_DIR }}");

  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-linter-deletion-wire-"));
  try {
    const bin = path.join(root, "bin"),
      argumentsPath = path.join(root, "recorder-arguments.txt"),
      reportDirectory = "real-project-results/shard-7";
    fs.mkdirSync(bin);
    fs.writeFileSync(
      path.join(bin, "node"),
      '#!/bin/sh\nprintf \'%s\\n\' "$@" > "$RECORDER_ARGUMENTS"\nmkdir -p "$2"\nprintf \'{}\\n\' > "$2/linter-deletion-readiness.json"\n',
      { mode: 0o755 },
    );
    const result = spawnSync("bash", ["-euo", "pipefail", "-c", String(recorder.run)], {
      cwd: root,
      encoding: "utf8",
      env: {
        ...process.env,
        FIXTURE_REPORT_DIR: reportDirectory,
        PATH: `${bin}:${process.env.PATH}`,
        RECORDER_ARGUMENTS: argumentsPath,
      },
    });
    assert.equal(result.status, 0, result.stderr);
    assert.deepEqual(fs.readFileSync(argumentsPath, "utf8").trim().split("\n"), [
      "tests/differential/linter-deletion-recorder.ts",
      reportDirectory,
    ]);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
  assert.deepEqual(Object.keys(deletionReadinessAdapters), ["formatter"]);
});
