import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import { parse } from "yaml";
import { deletionReadinessAdapters } from "../differential/deletion-readiness-adapters.mjs";

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
    uploadIndex = steps.findIndex(
      (step) => typeof step.uses === "string" && step.uses.startsWith("actions/upload-artifact@"),
    );
  assert(divergenceIndex >= 0, "lint divergence step is missing");
  assert(verifierIndex > divergenceIndex, "recorder must run after lint divergence");
  assert(uploadIndex > verifierIndex, "artifact upload must run after the recorder");
  const recorder = steps[verifierIndex];
  assert.equal(recorder.if, "${{ !cancelled() }}");
  assert.match(
    String(recorder.run),
    /node tests\/differential\/linter-deletion-recorder\.ts "\$FIXTURE_REPORT_DIR"/,
  );
  assert.deepEqual(Object.keys(deletionReadinessAdapters), ["formatter"]);
});
