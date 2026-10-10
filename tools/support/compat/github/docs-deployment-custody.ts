import assert from "node:assert/strict";
import { GitHub, sha256, stableJson, type Deployment } from "./docs-deployment-api.ts";
import {
  artifactIdentity,
  publishedReceipt,
  validateArtifact,
  validateBuild,
  type Artifact,
  type PublisherJob,
  type Receipt,
  type WorkflowRun,
} from "./docs-deployment-policy.ts";

export const legacyPublisher = 38013719817;
export const legacyWorkflowHash =
  "a2ba0528b06e317a8740b1c324a84a3de3c75c456a983d12d559f54b14e0e100";
export const custodyEnvironment = "vize-docs-source-custody";
export const custodyTask = "vize-docs-pages-v1";
export type Floor = {
  sourceSha: string;
  completedAt: string;
  receipt: Receipt;
  deploymentId: number | null;
};

export async function artifacts(api: GitHub, owner: WorkflowRun) {
  const values = await api.list<Artifact>("/actions/runs/" + owner.id + "/artifacts", "artifacts");
  return values.filter(
    (item) =>
      Date.parse(item.created_at) >= Date.parse(owner.run_started_at) &&
      (owner.status !== "completed" || Date.parse(item.created_at) <= Date.parse(owner.updated_at)),
  );
}
export async function buildFromRun(api: GitHub, repositoryId: number, run: WorkflowRun) {
  const values = await artifacts(api, run);
  const docs = values.filter((item) => item.name === "docs");
  assert.equal(docs.length, 1, "Exactly one immutable native Docs archive");
  validateArtifact(docs[0], run, "docs");
  const manifest = await api.manifest(docs[0]);
  return validateBuild(
    run,
    repositoryId,
    { runId: run.id, attempt: run.run_attempt, sourceSha: run.head_sha },
    values,
    manifest,
  );
}
async function buildFromReceipt(api: GitHub, repositoryId: number, receipt: Receipt) {
  const run = await api.json<WorkflowRun>(
    "/actions/runs/" + receipt.build.runId + "/attempts/" + receipt.build.attempt,
  );
  const values = await Promise.all(
    receipt.build.artifacts.map((item) => api.json<Artifact>("/actions/artifacts/" + item.id)),
  );
  // The immutable receipt is bound to the exact successful custody step's log.
  // Historical archives can expire; primary IDs/digests/run provenance must remain.
  return validateBuild(
    run,
    repositoryId,
    {
      runId: receipt.build.runId,
      attempt: receipt.build.attempt,
      sourceSha: receipt.build.sourceSha,
    },
    values,
    receipt.build.manifest,
    false,
  );
}
export async function qualifyDeployment(
  api: GitHub,
  repositoryId: number,
  deployment: Deployment,
  publisher: WorkflowRun,
  job: PublisherJob,
) {
  assert.equal(deployment.environment, custodyEnvironment);
  assert.equal(deployment.task, custodyTask);
  assert.equal(deployment.creator.login, "github-actions[bot]", "Actual workflow receipt writer");
  assert.equal(deployment.creator.type, "Bot");
  assert.equal(deployment.sha, publisher.head_sha, "Receipt cannot change a Git ref");
  const receipt = deployment.payload as Receipt;
  assert.equal(receipt.schema, custodyTask);
  const custody = job.steps.filter((step) => step.name === "Record Pages artifact custody");
  assert.equal(custody.length, 1);
  assert.equal(custody[0].conclusion, "success", "Receipt writer actually completed");
  const preparation = job.steps.filter(
    (step) => step.name === "Validate completed build and actual Pages publication floor",
  );
  assert.equal(preparation.length, 1);
  assert.equal(preparation[0].conclusion, "success");
  assert.equal(
    receipt.outstanding.since,
    preparation[0].started_at,
    "Journal cutoff is its actual primary preparation start",
  );
  const custodyAt = Date.parse(custody[0].completed_at ?? "");
  const createdAt = Date.parse(deployment.created_at);
  const pagesAt = Date.parse(
    job.steps.find((step) => step.name === "Deploy to GitHub Pages")?.completed_at ?? "",
  );
  assert(Number.isFinite(custodyAt) && Number.isFinite(createdAt) && Number.isFinite(pagesAt));
  assert(
    createdAt >= Date.parse(publisher.run_started_at) &&
      createdAt <= custodyAt &&
      custodyAt <= pagesAt,
    "Candidate receipt precedes its actual Pages publication",
  );
  const log = await api.jobLog(job.id);
  const marker = "vize-docs-receipt:" + deployment.id + ":" + sha256(stableJson(receipt));
  assert.equal(
    log.split(marker).length - 1,
    1,
    "Whole immutable receipt bound to its actual publisher log",
  );
  const [build, pages] = await Promise.all([
    buildFromReceipt(api, repositoryId, receipt),
    api.json<Artifact>("/actions/artifacts/" + receipt.pagesArtifact.id),
  ]);
  const result = publishedReceipt(receipt, publisher, job, build, pages);
  assert(result, "An unexecuted candidate receipt cannot establish a floor");
  return { ...result, deploymentId: deployment.id };
}
export async function qualifyLegacy(
  api: GitHub,
  repositoryId: number,
  publisher: WorkflowRun,
  job: PublisherJob,
): Promise<Floor> {
  // A bounded migration exception for the repository's observed original writer.
  // Unknown or receipt-less replacement writers are always refused.
  assert.equal(
    await api.workflowHash(publisher.head_sha),
    legacyWorkflowHash,
    "Exact preserved legacy Pages writer",
  );
  const log = await api.jobLog(job.id);
  const matches = [...log.matchAll(/\brun-id: (\d+)\b/g)];
  assert.equal(matches.length, 1, "Legacy log binds its one original Docs run");
  const buildRun = await api.json<WorkflowRun>("/actions/runs/" + matches[0][1]);
  assert.equal(buildRun.run_attempt, 1, "Legacy logs do not prove a rerun attempt");
  assert.equal(buildRun.head_sha, publisher.head_sha, "Original current-main writer source");
  const build = await buildFromRun(api, repositoryId, buildRun);
  const pagesValues = await artifacts(api, publisher);
  const pages = pagesValues.filter((item) => item.name === "github-pages-" + publisher.run_attempt);
  assert.equal(pages.length, 1);
  validateArtifact(pages[0], publisher, pages[0].name);
  assert.deepEqual(
    await api.manifest(pages[0], true),
    build.manifest,
    "Native output inside the actual Pages archive",
  );
  for (const item of [...build.artifacts, artifactIdentity(pages[0])]) {
    assert(
      log.includes("ID: " + item.id) || log.includes("Artifact ID " + item.id),
      "Original download/upload log artifact ID",
    );
    assert(
      log.includes(item.digest.slice(7)),
      "Original download/upload log whole artifact digest",
    );
  }
  const receipt: Receipt = {
    schema: custodyTask,
    build,
    publisher: {
      runId: publisher.id,
      attempt: publisher.run_attempt,
      jobId: job.id,
      contextSha: publisher.head_sha,
    },
    pagesArtifact: artifactIdentity(pages[0]),
    outstanding: { since: publisher.run_started_at, publisherAttempts: [], environmentIds: [] },
  };
  const result = publishedReceipt(receipt, publisher, job, build, pages[0]);
  assert(result);
  return { ...result, deploymentId: null };
}
