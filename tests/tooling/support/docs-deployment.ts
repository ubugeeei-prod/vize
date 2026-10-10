import {
  artifactIdentity,
  sourceArtifactNames,
  validateBuild,
  type Artifact,
  type PublisherJob,
  type Receipt,
  type WorkflowRun,
} from "../../../tools/support/compat/github/docs-deployment-policy.ts";

export const repositoryId = 1123610216;
export const sourceSha = "26031a4fbb30a1e86511919bafc7035b08440ef3";
export const manifest = { sourceSha, assetFingerprint: "a".repeat(64), sha256: "b".repeat(64) };
export function run(overrides: Partial<WorkflowRun> = {}): WorkflowRun {
  return {
    id: 38010819888,
    run_attempt: 1,
    workflow_id: 42,
    path: ".github/workflows/build-docs.yml",
    event: "push",
    head_branch: "main",
    head_sha: sourceSha,
    status: "completed",
    conclusion: "success",
    repository: { id: repositoryId },
    head_repository: { id: repositoryId },
    created_at: "2026-10-10T00:51:18Z",
    run_started_at: "2026-10-10T00:51:20Z",
    updated_at: "2026-10-10T01:34:47Z",
    ...overrides,
  };
}
export function artifact(owner: WorkflowRun, name: string, id = 1): Artifact {
  return {
    id,
    name,
    expired: false,
    digest: "sha256:" + "c".repeat(64),
    created_at: "2026-10-10T01:31:35Z",
    workflow_run: {
      id: owner.id,
      repository_id: repositoryId,
      head_repository_id: repositoryId,
      head_branch: "main",
      head_sha: owner.head_sha,
    },
  };
}
export function sourceArtifacts(owner = run()) {
  return sourceArtifactNames.map((name, i) => artifact(owner, name, i + 1));
}
export function build(owner = run(), artifacts = sourceArtifacts(owner), available = true) {
  return validateBuild(
    owner,
    repositoryId,
    { runId: 38010819888, attempt: 1, sourceSha },
    artifacts,
    manifest,
    available,
  );
}
export function publisherFixture() {
  const publisher = run({
    id: 38013719817,
    path: ".github/workflows/deploy-docs.yml",
    event: "workflow_run",
    created_at: "2026-10-10T01:34:47Z",
    run_started_at: "2026-10-10T01:34:57Z",
    updated_at: "2026-10-10T01:35:41Z",
  });
  const job: PublisherJob = {
    id: 114099286630,
    run_id: publisher.id,
    run_attempt: 1,
    head_sha: publisher.head_sha,
    status: "completed",
    steps: [
      {
        name: "Deploy to GitHub Pages",
        conclusion: "success",
        started_at: "2026-10-10T01:35:30Z",
        completed_at: "2026-10-10T01:35:33Z",
      },
    ],
  };
  const pages = {
    ...artifact(publisher, "github-pages-1", 11655422443),
    created_at: "2026-10-10T01:35:27Z",
  };
  const receipt: Receipt = {
    schema: "vize-docs-pages-v1",
    build: build(),
    publisher: { runId: publisher.id, attempt: 1, jobId: job.id, contextSha: publisher.head_sha },
    pagesArtifact: artifactIdentity(pages),
    outstanding: { since: "2026-10-10T01:34:58Z", publisherAttempts: [], environmentIds: [] },
  };
  return { publisher, job, pages, receipt };
}
