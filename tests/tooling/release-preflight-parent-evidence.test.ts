import assert from "node:assert/strict";
import { test } from "node:test";

import {
  bootstrapRequiredWorkflowRuns,
  createReleaseGateDispatchPlans,
} from "../../legacy-tools/github/release-preflight-bootstrap.mjs";
import { isVersionMetadataOnlyRelease } from "../../legacy-tools/github/release-preflight-core.mjs";
import { requiredReleaseWorkflows } from "../../legacy-tools/github/release-preflight-evidence.mjs";
import { releaseEvidenceShas } from "../../legacy-tools/github/release-preflight.mjs";

/** The workspace lint budget is zero warnings, and `.sort()` needs a comparator. */
const byCodeUnit = (left: string, right: string) => (left < right ? -1 : left > right ? 1 : 0);

const tagSha = "a".repeat(40);
const parentSha = "b".repeat(40);

/**
 * The release commit the script writes: the same version into 27 tracked files
 * plus the two regenerated lockfiles, and nothing else.
 */
const versionOnlyPaths = [
  "Cargo.lock",
  "Cargo.toml",
  "README.md",
  "editors/vscode/package.json",
  "editors/zed/extension.toml",
  "npm/cli/package.json",
  "pnpm-lock.yaml",
  "pnpm-workspace.yaml",
];

test("version metadata paths are recognised and source paths are not", () => {
  assert.equal(isVersionMetadataOnlyRelease(versionOnlyPaths), true);
  assert.equal(isVersionMetadataOnlyRelease([]), false);
  for (const source of [
    "crates/vize_canon/src/lib.rs",
    "editors/vscode/test/suite/extension-host.cjs",
    "tools/commands/ci/github/release-preflight.rs",
    ".github/workflows/release.yml",
  ]) {
    assert.equal(
      isVersionMetadataOnlyRelease([...versionOnlyPaths, source]),
      false,
      `${source} must defeat the reuse`,
    );
  }
});

test("every required gate reuses the parent; artifact gates never do", () => {
  const shas = releaseEvidenceShas({ sha: tagSha, baseSha: parentSha, versionOnly: true });
  for (const gate of requiredReleaseWorkflows) {
    assert.deepEqual(shas.get(gate), [tagSha, parentSha], gate);
  }
  // Its subject is the artifact the tag built, so it must never be reused.
  assert.equal(shas.get("Native Smoke"), undefined);
  assert.equal(
    releaseEvidenceShas({ sha: tagSha, baseSha: parentSha, versionOnly: false }).size,
    0,
  );
});

function greenRun(
  workflowPath: string,
  headSha: string,
  displayTitle: string,
  event = "workflow_dispatch",
) {
  return {
    conclusion: "success",
    created_at: "2026-08-19T00:00:00Z",
    display_title: displayTitle,
    event,
    head_branch: "main",
    head_sha: headSha,
    html_url: `https://example.test/${workflowPath}`,
    id: 1,
    path: workflowPath,
    status: "completed",
  };
}

test("a version-only release never dispatches a gate the parent already proved", async () => {
  const plans = createReleaseGateDispatchPlans({
    ref: "v0.350.1",
    headSha: tagSha,
    baseSha: parentSha,
  });
  const runs = [
    greenRun(".github/workflows/check.yml", parentSha, "Check", "push"),
    greenRun(".github/workflows/miri.yml", parentSha, "Miri", "push"),
    greenRun(".github/workflows/build-docs.yml", parentSha, "Docs build", "push"),
    greenRun(".github/workflows/benchmark.yml", parentSha, `Benchmark ${parentSha}...${tagSha}`),
    greenRun(".github/workflows/fuzz.yml", parentSha, `Fuzz replay @ ${tagSha}`),
    greenRun(
      ".github/workflows/real-project-matrix.yml",
      parentSha,
      `Real Project Matrix @ ${tagSha}`,
    ),
  ];
  const dispatched: string[] = [];
  const selected = await bootstrapRequiredWorkflowRuns({
    sha: tagSha,
    dispatchPlans: plans,
    listRuns: async () => runs,
    dispatchWorkflow: async (plan: { workflowName: string }) =>
      void dispatched.push(plan.workflowName),
    evidenceShas: releaseEvidenceShas({ sha: tagSha, baseSha: parentSha, versionOnly: true }),
    now: () => 0,
    timeoutMs: 0,
    sleep: async () => {},
  });
  assert.deepEqual(dispatched, [], "no gate should be dispatched");
  assert.deepEqual(
    [...selected.keys()].sort(byCodeUnit),
    [...requiredReleaseWorkflows].sort(byCodeUnit),
  );
  for (const gate of requiredReleaseWorkflows) {
    assert.equal(selected.get(gate)?.head_sha, parentSha, gate);
  }
});

test("without the reuse the same parent evidence does not satisfy the gates", async () => {
  const plans = createReleaseGateDispatchPlans({
    ref: "v0.350.1",
    headSha: tagSha,
    baseSha: parentSha,
  });
  const dispatched: string[] = [];
  await assert.rejects(
    bootstrapRequiredWorkflowRuns({
      sha: tagSha,
      dispatchPlans: plans,
      listRuns: async () => [greenRun(".github/workflows/check.yml", parentSha, "Check", "push")],
      dispatchWorkflow: async (plan: { workflowName: string }) =>
        void dispatched.push(plan.workflowName),
      now: () => 0,
      timeoutMs: 0,
      sleep: async () => {},
    }),
    /Required release gates are not green/,
  );
  assert.deepEqual(dispatched.sort(byCodeUnit), ["Benchmark", "Fuzz", "Real Project Matrix"]);
});
