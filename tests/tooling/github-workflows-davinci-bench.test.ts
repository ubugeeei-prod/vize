import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";

import { readRepoFile } from "./support/github-workflows.ts";

interface WorkflowStep {
  name?: string;
  if?: string;
  uses?: string;
  run?: string;
  with?: Record<string, unknown>;
}

interface WorkflowJob {
  steps?: WorkflowStep[];
}

test("check workflow uploads Davinci compact-storage bench reports", () => {
  const workflow = readRepoFile(".github", "workflows", "check.yml");
  const action = readRepoFile(
    ".github",
    "actions",
    "upload-davinci-compact-storage-bench-reports",
    "action.yml",
  );
  const jobs = (parse(workflow) as { jobs?: Record<string, WorkflowJob> }).jobs ?? {};
  const actionSteps = (parse(action) as { runs?: { steps?: WorkflowStep[] } }).runs?.steps ?? [];
  const steps = jobs["clippy-and-test"]?.steps ?? [];
  const gateIndex = steps.findIndex(
    (step) => step.name === "Davinci compact-storage allocation gate",
  );

  assert.notEqual(gateIndex, -1);
  assert.match(
    steps[gateIndex].run ?? "",
    /cargo bench -p vize_ricalco --bench davinci_storage -- --quick/,
  );
  assert.match(
    steps[gateIndex].run ?? "",
    /node tools\/davinci\/bench-compare\.mjs --bench ricalco_lower_vfor_three_aliases --bench ricalco_emit_von_two_per_bucket/,
  );
  assert.deepEqual(steps[gateIndex + 1], {
    name: "Upload Davinci compact-storage bench reports",
    if: "${{ always() }}",
    uses: "./.github/actions/upload-davinci-compact-storage-bench-reports",
  });
  assert.deepEqual(actionSteps, [
    {
      name: "Upload reports",
      uses: "actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a",
      with: {
        name: "davinci-compact-storage-bench-reports",
        path: "bench/results/davinci/*.json",
        "if-no-files-found": "error",
        "retention-days": 14,
      },
    },
  ]);
  assert.match(
    action,
    /actions\/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a\s*# v7\.0\.1/,
  );
});
