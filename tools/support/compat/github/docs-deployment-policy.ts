import assert from "node:assert/strict";

export const sourceArtifactNames = ["docs", "docs-render-evidence", "playground", "musea-examples"];
const sha = /^[0-9a-f]{40}$/;
const digest = /^sha256:[0-9a-f]{64}$/;
const hash = /^[0-9a-f]{64}$/;

export type WorkflowRun = {
  id: number;
  run_attempt: number;
  workflow_id: number;
  path: string;
  event: string;
  head_branch: string;
  head_sha: string;
  status: string;
  conclusion: string | null;
  repository: { id: number };
  head_repository: { id: number };
  created_at: string;
  run_started_at: string;
  updated_at: string;
};
export type Artifact = {
  id: number;
  name: string;
  expired: boolean;
  digest: string;
  created_at: string;
  workflow_run: {
    id: number;
    repository_id: number;
    head_repository_id: number;
    head_branch: string;
    head_sha: string;
  };
};
export type ManifestIdentity = { sourceSha: string; assetFingerprint: string; sha256: string };
export type ArtifactIdentity = Pick<
  Artifact,
  "id" | "name" | "digest" | "created_at" | "workflow_run"
>;
export type BuildIdentity = {
  repositoryId: number;
  runId: number;
  attempt: number;
  sourceSha: string;
  artifacts: ArtifactIdentity[];
  manifest: ManifestIdentity;
};
export type OutstandingWriters = {
  since: string;
  publisherAttempts: { runId: number; attempt: number }[];
  environmentIds: number[];
};
export type Receipt = {
  schema: "vize-docs-pages-v1";
  build: BuildIdentity;
  publisher: { runId: number; attempt: number; jobId: number; contextSha: string };
  pagesArtifact: ArtifactIdentity;
  outstanding: OutstandingWriters;
};
export type PublisherJob = {
  id: number;
  run_id: number;
  run_attempt: number;
  head_sha: string;
  status: string;
  steps: {
    name: string;
    conclusion: string | null;
    started_at: string | null;
    completed_at: string | null;
  }[];
};
export type Comparison = {
  base_commit: { sha: string };
  merge_base_commit: { sha: string };
  status: string;
  ahead_by: number;
  behind_by: number;
};

function positive(value: number, label: string) {
  assert(Number.isSafeInteger(value) && value > 0, label);
}
export function validateRun(run: WorkflowRun, repositoryId: number, path: string) {
  positive(repositoryId, "Trusted repository ID");
  positive(run.id, "Workflow run ID");
  positive(run.run_attempt, "Workflow attempt");
  positive(run.workflow_id, "Workflow identity");
  assert.equal(run.path, path, "Trusted workflow path");
  assert.equal(run.repository.id, repositoryId, "Trusted repository");
  assert.equal(run.head_repository.id, repositoryId, "No fork source");
  assert.equal(run.head_branch, "main", "Actual main branch only");
  assert.match(run.head_sha, sha, "Whole source SHA");
}
export function artifactIdentity(artifact: Artifact): ArtifactIdentity {
  return {
    id: artifact.id,
    name: artifact.name,
    digest: artifact.digest,
    created_at: artifact.created_at,
    workflow_run: {
      id: artifact.workflow_run.id,
      repository_id: artifact.workflow_run.repository_id,
      head_repository_id: artifact.workflow_run.head_repository_id,
      head_branch: artifact.workflow_run.head_branch,
      head_sha: artifact.workflow_run.head_sha,
    },
  };
}
export function validateArtifact(
  artifact: Artifact,
  run: WorkflowRun,
  name: string,
  available = true,
) {
  positive(artifact.id, "Artifact ID");
  assert.equal(artifact.name, name, "Exact artifact name");
  assert.equal(typeof artifact.expired, "boolean", "Explicit artifact availability");
  if (available) assert.equal(artifact.expired, false, "Available immutable artifact");
  assert.match(artifact.digest, digest, "Whole artifact digest");
  assert.equal(artifact.workflow_run.id, run.id, "Exact originating workflow run");
  assert.equal(artifact.workflow_run.repository_id, run.repository.id, "Artifact repository");
  assert.equal(
    artifact.workflow_run.head_repository_id,
    run.head_repository.id,
    "Artifact source repository",
  );
  assert.equal(artifact.workflow_run.head_branch, "main", "Artifact source branch");
  assert.equal(artifact.workflow_run.head_sha, run.head_sha, "Artifact source SHA");
  const created = Date.parse(artifact.created_at);
  assert(Number.isFinite(created), "Artifact timestamp");
  assert(created >= Date.parse(run.run_started_at), "Artifact belongs to this attempt");
  if (run.status === "completed")
    assert(created <= Date.parse(run.updated_at), "Artifact existed in the completed build");
}
export function validateBuild(
  run: WorkflowRun,
  repositoryId: number,
  expected: { runId: number; attempt: number; sourceSha: string },
  artifacts: Artifact[],
  manifest: ManifestIdentity,
  available = true,
): BuildIdentity {
  validateRun(run, repositoryId, ".github/workflows/build-docs.yml");
  assert(["push", "schedule", "workflow_dispatch"].includes(run.event), "Trusted Docs trigger");
  assert.equal(run.id, expected.runId, "Triggered Docs run");
  assert.equal(run.run_attempt, expected.attempt, "Triggered Docs attempt");
  assert.equal(run.head_sha, expected.sourceSha, "Triggered Docs source");
  assert.equal(run.status, "completed", "Complete Docs build");
  assert.equal(run.conclusion, "success", "All Docs gates succeeded");
  const selected = sourceArtifactNames.map((name) => {
    const matches = artifacts.filter((artifact) => artifact.name === name);
    assert.equal(matches.length, 1, "Exactly one source artifact: " + name);
    validateArtifact(matches[0], run, name, available);
    return artifactIdentity(matches[0]);
  });
  assert.equal(new Set(selected.map((artifact) => artifact.id)).size, selected.length);
  assert.equal(manifest.sourceSha, run.head_sha, "Native manifest source SHA");
  assert.match(manifest.assetFingerprint, hash, "Native asset fingerprint");
  assert.match(manifest.sha256, hash, "Whole native manifest hash");
  return {
    repositoryId,
    runId: run.id,
    attempt: run.run_attempt,
    sourceSha: run.head_sha,
    artifacts: selected,
    manifest,
  };
}
export function sourceRelation(base: string, head: string, comparison: Comparison) {
  assert.match(base, sha);
  assert.match(head, sha);
  assert.equal(comparison.base_commit.sha, base, "Exact comparison base");
  assert(Number.isSafeInteger(comparison.ahead_by) && comparison.ahead_by >= 0);
  assert(Number.isSafeInteger(comparison.behind_by) && comparison.behind_by >= 0);
  if (base === head) {
    assert.equal(comparison.status, "identical");
    assert.equal(comparison.merge_base_commit.sha, base);
    assert.equal(comparison.ahead_by + comparison.behind_by, 0);
    return "equal";
  }
  if (comparison.status === "ahead") {
    assert.equal(comparison.merge_base_commit.sha, base, "Complete ancestor relation");
    assert.equal(comparison.behind_by, 0);
    assert(comparison.ahead_by > 0);
    return "newer";
  }
  if (comparison.status === "behind") {
    assert.equal(comparison.merge_base_commit.sha, head, "Complete descendant relation");
    assert.equal(comparison.ahead_by, 0);
    assert(comparison.behind_by > 0);
    return "older";
  }
  assert.equal(comparison.status, "diverged", "Known primary comparison status");
  return "diverged";
}
export function publicationDecision(
  incoming: string,
  main: string,
  mainComparison: Comparison,
  floor: string | null,
  floorComparison: Comparison | null,
  pagesConfigured: boolean,
) {
  const mainRelation = sourceRelation(incoming, main, mainComparison);
  assert(["equal", "newer"].includes(mainRelation), "Source must be an actual main ancestor");
  if (floor === null) {
    assert.equal(floorComparison, null);
    assert.equal(
      pagesConfigured,
      false,
      "Configured Pages requires an authoritative publication floor",
    );
    return { eligible: true, reason: "First publication to an unconfigured Pages site" };
  }
  assert(floorComparison, "Primary deployed-source comparison");
  const relation = sourceRelation(floor, incoming, floorComparison);
  assert.notEqual(relation, "diverged", "Published source must share the protected main lineage");
  return {
    eligible: relation === "newer",
    reason:
      relation === "newer" ? "Newer completed main build" : "Already published or older source",
  };
}
export function publishedReceipt(
  receipt: Receipt | null,
  publisher: WorkflowRun,
  job: PublisherJob,
  build: BuildIdentity | null,
  pagesArtifact: Artifact | null,
) {
  assert.equal(job.run_id, publisher.id, "Actual publisher job run");
  assert.equal(job.run_attempt, publisher.run_attempt, "Actual publisher attempt");
  assert.equal(job.head_sha, publisher.head_sha, "Actual publisher context");
  const steps = job.steps.filter((step) => step.name === "Deploy to GitHub Pages");
  assert.equal(steps.length, 1, "One actual Pages deployment step");
  if (steps[0].conclusion === "skipped") return null;
  assert.equal(steps[0].conclusion, "success", "An uncertain Pages effect cannot permit rollback");
  assert(steps[0].completed_at && Number.isFinite(Date.parse(steps[0].completed_at)));
  assert(receipt, "Actual publication requires its durable receipt");
  assert(build && pagesArtifact, "Whole primary source and Pages artifact custody");
  validateRun(publisher, build.repositoryId, ".github/workflows/deploy-docs.yml");
  assert.equal(publisher.event, "workflow_run", "Trusted publisher trigger");
  assert.equal(receipt.schema, "vize-docs-pages-v1");
  assert(Number.isFinite(Date.parse(receipt.outstanding.since)), "Primary writer journal cutoff");
  const pending = receipt.outstanding.publisherAttempts.map(({ runId, attempt }) => {
    positive(runId, "Outstanding publisher");
    positive(attempt, "Outstanding attempt");
    return runId + ":" + attempt;
  });
  assert.equal(new Set(pending).size, pending.length, "Unique outstanding publisher attempts");
  receipt.outstanding.environmentIds.forEach((id) => positive(id, "Outstanding Pages environment"));
  assert.equal(
    new Set(receipt.outstanding.environmentIds).size,
    receipt.outstanding.environmentIds.length,
  );
  assert.deepEqual(receipt.build, build, "No forged source receipt");
  assert.deepEqual(receipt.publisher, {
    runId: publisher.id,
    attempt: publisher.run_attempt,
    jobId: job.id,
    contextSha: publisher.head_sha,
  });
  validateArtifact(pagesArtifact, publisher, "github-pages-" + publisher.run_attempt, false);
  assert.deepEqual(
    receipt.pagesArtifact,
    artifactIdentity(pagesArtifact),
    "No forged Pages artifact receipt",
  );
  return { sourceSha: build.sourceSha, completedAt: steps[0].completed_at, receipt };
}
