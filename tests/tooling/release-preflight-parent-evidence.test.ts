import assert from "node:assert/strict";
import { test } from "node:test";

import {
  bootstrapRequiredWorkflowRuns,
  createReleaseGateDispatchPlans,
} from "../../tools/github/release-preflight-bootstrap.mjs";
import { isVersionMetadataOnlyRelease } from "../../tools/github/release-preflight-core.mjs";
import { releaseEvidenceShas } from "../../tools/github/release-preflight.mjs";

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
    "tools/github/release-preflight.mjs",
    ".github/workflows/release.yml",
  ]) {
    assert.equal(
      isVersionMetadataOnlyRelease([...versionOnlyPaths, source]),
      false,
      `${source} must defeat the reuse`,
    );
  }
});

test("only the code gates reuse the parent; Native Smoke stays pinned to the tag", () => {
  const shas = releaseEvidenceShas({ sha: tagSha, baseSha: parentSha, versionOnly: true });
  assert.deepEqual(shas.get("Real Project Matrix"), [tagSha, parentSha]);
  assert.deepEqual(shas.get("Check"), [tagSha, parentSha]);
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
    greenRun(".github/workflows/e2e.yml", parentSha, `App E2E all @ ${tagSha}`),
    greenRun(".github/workflows/fuzz.yml", parentSha, `Fuzz replay @ ${tagSha}`),
    greenRun(
      ".github/workflows/real-project-matrix.yml",
      parentSha,
      `Real Project Matrix @ ${tagSha}`,
    ),
    greenRun(".github/workflows/native-smoke.yml", tagSha, "Native Smoke"),
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
  assert.equal(selected.get("Real Project Matrix")?.head_sha, parentSha);
  assert.equal(selected.get("Native Smoke")?.head_sha, tagSha);
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
  assert.ok(dispatched.includes("Real Project Matrix"), "the matrix must be dispatched");
});
