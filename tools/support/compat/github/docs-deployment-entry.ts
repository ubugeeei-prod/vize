import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { appendFileSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import {
  GitHub,
  sha256,
  stableJson,
  terminalPagesMetadata,
  type Deployment,
} from "./docs-deployment-api.ts";
import {
  artifacts,
  buildFromRun,
  custodyEnvironment,
  custodyTask,
} from "./docs-deployment-custody.ts";
import { publicationState } from "./docs-deployment-history.ts";
import {
  artifactIdentity,
  publicationDecision,
  publishedReceipt,
  validateArtifact,
  validateBuild,
  validateRun,
  type Artifact,
  type BuildIdentity,
  type Comparison,
  type PublisherJob,
  type Receipt,
  type OutstandingWriters,
  type WorkflowRun,
} from "./docs-deployment-policy.ts";

const repository = process.env.GITHUB_REPOSITORY ?? "";
const api = new GitHub(repository, process.env.GITHUB_TOKEN ?? "");
const directory = process.env.RUNNER_TEMP ?? "";
assert(directory, "Runner-owned custody directory");
const statePath = join(directory, "vize-docs-deployment-state.json");
type State = {
  build: BuildIdentity;
  repositoryId: number;
  validatorSha: string;
  outstanding: OutstandingWriters;
  preparationStartedAt: string;
  receipt?: Receipt;
  receiptId?: number;
};
const state = () => JSON.parse(readFileSync(statePath, "utf8")) as State;
const save = (value: State) => writeFileSync(statePath, JSON.stringify(value));
const output = (key: string, value: string) => {
  assert(process.env.GITHUB_OUTPUT);
  assert.match(key, /^[a-z_]+$/);
  assert(!value.includes("\n") && !value.includes("\r"));
  appendFileSync(process.env.GITHUB_OUTPUT, key + "=" + value + "\n");
};
const compare = (base: string, head: string) =>
  api.json<Comparison>("/compare/" + base + "..." + head);
async function ownPublisher(repositoryId: number) {
  const runId = Number(process.env.GITHUB_RUN_ID);
  const attempt = Number(process.env.GITHUB_RUN_ATTEMPT);
  const publisher = await api.json<WorkflowRun>("/actions/runs/" + runId + "/attempts/" + attempt);
  validateRun(publisher, repositoryId, ".github/workflows/deploy-docs.yml");
  assert.equal(publisher.id, runId);
  assert.equal(publisher.run_attempt, attempt);
  assert.equal(publisher.head_sha, process.env.GITHUB_SHA);
  assert.equal(publisher.event, "workflow_run");
  const jobs = await api.list<PublisherJob>(
    "/actions/runs/" + runId + "/attempts/" + attempt + "/jobs",
    "jobs",
  );
  const matches = jobs.filter((job) =>
    job.steps.some((step) => step.name === "Record Pages artifact custody"),
  );
  assert.equal(matches.length, 1, "Exact current Pages writer job");
  return { publisher, job: matches[0] };
}
async function prepare() {
  assert(process.env.GITHUB_EVENT_PATH);
  const event = JSON.parse(readFileSync(process.env.GITHUB_EVENT_PATH, "utf8")) as {
    repository: { id: number };
    workflow_run: WorkflowRun;
  };
  const metadata = await api.json<{ id: number }>("");
  assert.equal(metadata.id, event.repository.id, "Trusted event repository");
  const current = await ownPublisher(metadata.id);
  const preparation = current.job.steps.filter(
    (step) => step.name === "Validate completed build and actual Pages publication floor",
  );
  assert.equal(preparation.length, 1);
  const preparationStartedAt = preparation[0].started_at;
  assert(preparationStartedAt && Number.isFinite(Date.parse(preparationStartedAt)));
  validateRun(event.workflow_run, metadata.id, ".github/workflows/build-docs.yml");
  const owner = await api.json<WorkflowRun>(
    "/actions/runs/" + event.workflow_run.id + "/attempts/" + event.workflow_run.run_attempt,
  );
  assert.equal(owner.id, event.workflow_run.id);
  assert.equal(owner.head_sha, event.workflow_run.head_sha);
  assert.equal(owner.run_attempt, event.workflow_run.run_attempt);
  const build = await buildFromRun(api, metadata.id, owner);
  const main = await api.json<{ commit: { sha: string } }>("/branches/main");
  const validatorSha = execFileSync("git", ["rev-parse", "HEAD"], { encoding: "utf8" }).trim();
  const site = await api.response("/pages");
  assert(site.ok || site.status === 404, "Primary Pages configuration is provable");
  const observed = await publicationState(
    api,
    metadata.id,
    Number(process.env.GITHUB_RUN_ID),
    await api.workflowHash(validatorSha),
    site.ok,
  );
  const floor = observed.floor;
  const decision = publicationDecision(
    build.sourceSha,
    main.commit.sha,
    await compare(build.sourceSha, main.commit.sha),
    floor?.sourceSha ?? null,
    floor ? await compare(floor.sourceSha, build.sourceSha) : null,
    site.ok,
  );
  save({
    build,
    repositoryId: metadata.id,
    validatorSha,
    outstanding: observed.outstanding,
    preparationStartedAt,
  });
  output("eligible", String(decision.eligible));
  output("artifact_ids", build.artifacts.map((item) => item.id).join(","));
  console.log(
    decision.reason +
      ": source=" +
      build.sourceSha +
      " floor=" +
      (floor?.sourceSha ?? "unconfigured") +
      " main=" +
      main.commit.sha,
  );
}
async function record() {
  const saved = state();
  const native = readFileSync("site/_og/manifest.json");
  const parsed = JSON.parse(native.toString()) as { sourceSha: string; assetFingerprint: string };
  assert.deepEqual(
    {
      sourceSha: parsed.sourceSha,
      assetFingerprint: parsed.assetFingerprint,
      sha256: sha256(native),
    },
    saved.build.manifest,
    "Whole constructed native site matches its immutable Docs build",
  );
  const owner = await api.json<WorkflowRun>(
    "/actions/runs/" + saved.build.runId + "/attempts/" + saved.build.attempt,
  );
  const sourceArtifacts = await Promise.all(
    saved.build.artifacts.map((item) => api.json<Artifact>("/actions/artifacts/" + item.id)),
  );
  assert.deepEqual(
    validateBuild(
      owner,
      saved.repositoryId,
      { runId: saved.build.runId, attempt: saved.build.attempt, sourceSha: saved.build.sourceSha },
      sourceArtifacts,
      saved.build.manifest,
    ),
    saved.build,
  );
  const { publisher, job } = await ownPublisher(saved.repositoryId);
  const values = await artifacts(api, publisher);
  const pages = values.filter((item) => item.name === "github-pages-" + publisher.run_attempt);
  assert.equal(pages.length, 1);
  validateArtifact(pages[0], publisher, pages[0].name);
  assert.deepEqual(
    await api.manifest(pages[0], true),
    saved.build.manifest,
    "Actual immutable Pages archive contains the same native output",
  );
  const receipt: Receipt = {
    schema: custodyTask,
    build: saved.build,
    publisher: {
      runId: publisher.id,
      attempt: publisher.run_attempt,
      jobId: job.id,
      contextSha: publisher.head_sha,
    },
    pagesArtifact: artifactIdentity(pages[0]),
    outstanding: { ...saved.outstanding, since: saved.preparationStartedAt },
  };
  const deployment = await api.json<Deployment>("/deployments", {
    ref: publisher.head_sha,
    task: custodyTask,
    environment: custodyEnvironment,
    auto_merge: false,
    required_contexts: [],
    description: "Exact Docs source and immutable Pages artifact custody",
    payload: receipt,
    production_environment: false,
  });
  assert(Number.isSafeInteger(deployment.id) && deployment.id > 0);
  assert.equal(deployment.sha, publisher.head_sha);
  assert.deepEqual(deployment.payload, receipt);
  save({ ...saved, receipt, receiptId: deployment.id });
  console.log("vize-docs-receipt:" + deployment.id + ":" + sha256(stableJson(receipt)));
  await api.json("/deployments/" + deployment.id + "/statuses", {
    state: "in_progress",
    auto_inactive: false,
    log_url:
      "https://github.com/" + repository + "/actions/runs/" + publisher.id + "/job/" + job.id,
  });
}
async function finish() {
  const saved = state();
  assert(saved.receipt && saved.receiptId, "Recorded candidate custody");
  const { publisher, job } = await terminalPagesMetadata(() => ownPublisher(saved.repositoryId));
  const pages = await api.json<Artifact>("/actions/artifacts/" + saved.receipt.pagesArtifact.id);
  assert(
    publishedReceipt(saved.receipt, publisher, job, saved.build, pages),
    "Only real Pages success qualifies publication",
  );
  await api.json("/deployments/" + saved.receiptId + "/statuses", {
    state: "success",
    auto_inactive: false,
    log_url:
      "https://github.com/" + repository + "/actions/runs/" + publisher.id + "/job/" + job.id,
    environment_url: process.env.PAGES_URL ?? "",
  });
}
const mode = process.argv[2];
if (mode === "prepare") await prepare();
else if (mode === "record") await record();
else {
  assert.equal(mode, "finish", "Known custody operation");
  await finish();
}
