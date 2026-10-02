import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";

export function validateCampaignAuthority(
  context: any,
  approvedPlanPath: string,
  expectedHead: string,
  expectedPlanSha256: string,
  root: string,
) {
  const { plan, source, git, sha256 } = context;
  assert.match(expectedHead, /^[a-f0-9]{40}$/);
  assert.match(expectedPlanSha256, /^[a-f0-9]{64}$/);
  const bytes = fs.readFileSync(approvedPlanPath);
  assert.equal(sha256(bytes), expectedPlanSha256, "reviewed plan bytes changed");
  const approved = JSON.parse(bytes.toString());
  assert.equal(source.revision, expectedHead, "dispatch source does not equal approved HEAD");
  assert.equal(
    git("rev-parse", "HEAD^"),
    approved.source.revision,
    "diagnostic child has wrong parent",
  );
  assert.equal(
    git("rev-parse", "HEAD^^{tree}"),
    approved.source.tree,
    "reviewed parent tree changed",
  );
  const paths = git("diff", "--name-only", "HEAD^", "HEAD").split("\n").filter(Boolean);
  const prefix = "tools/campaigns/formatter-history/";
  const allowed = new Set([
    ...approved.runnerFiles.map((entry: any) => prefix + entry.name),
    prefix + "reviewed-plan.json",
    prefix + "formatter-history-diagnostic.check.yml",
    ".github/workflows/check.yml",
    "docs/davinci/decisions/2026-10-02-formatter-history-campaign.md",
    "docs/davinci/decisions/2026-09-27-level-restructure.md",
  ]);
  assert(
    paths.every((file: string) => allowed.has(file)),
    "diagnostic child changes unrelated source",
  );
  for (const field of ["formatterTree", "cargoLockSha256", "observerSha256"])
    assert.equal(source[field], approved.source[field], `reviewed source changed: ${field}`);
  const { source: _currentSource, ...currentContract } = plan;
  const { source: _approvedSource, ...approvedContract } = approved;
  assert.deepEqual(
    currentContract,
    approvedContract,
    "reviewed manifests/laws/controls/runner changed",
  );
  assert.equal(
    sha256(fs.readFileSync(path.join(root, ".github/workflows/check.yml"))),
    approved.diagnosticWorkflowSha256,
    "diagnostic workflow does not equal reviewed workflow",
  );
  return {
    reviewedParent: approved.source,
    observedHead: source,
    reviewedPlanSha256: expectedPlanSha256,
    diagnosticChangedPaths: paths,
    parentExecutionCredit: false,
  };
}
