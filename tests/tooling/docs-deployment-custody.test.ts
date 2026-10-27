import assert from "node:assert/strict";
import { test } from "node:test";
import { qualifyDeployment } from "../../tools/support/compat/github/docs-deployment-custody.ts";
import { artifactIdentity } from "../../tools/support/compat/github/docs-deployment-policy.ts";
import { repositoryId, run, sourceArtifacts, sourceSha } from "./support/docs-deployment.ts";

import { verifySolePagesWriter } from "../../tools/support/compat/github/docs-deployment-writers.ts";
import { custodyFixture } from "./support/docs-deployment-custody.ts";

test("durable custody is qualified by its exact primary writer log, source run, native artifact and actual Pages step", async () => {
  const { api, publisher, job, deployment, receipt, values, pages } = custodyFixture();
  assert.equal(
    (await qualifyDeployment(api, repositoryId, deployment, publisher, job)).sourceSha,
    sourceSha,
  );
  assert.equal(
    (
      await qualifyDeployment(
        api,
        repositoryId,
        deployment,
        { ...publisher, conclusion: "failure" },
        job,
      )
    ).sourceSha,
    sourceSha,
    "Later cleanup failure does not erase real Pages success",
  );
  for (const value of sourceArtifacts())
    values.set("/actions/artifacts/" + value.id, {
      ...value,
      expired: true,
      updated_at: "2026-11-01T00:00:00Z",
    });
  values.set("/actions/artifacts/" + pages.id, {
    ...pages,
    expired: true,
    updated_at: "2026-11-01T00:00:00Z",
  });
  assert.equal(
    (await qualifyDeployment(api, repositoryId, deployment, publisher, job)).sourceSha,
    sourceSha,
  );
  assert.deepEqual(artifactIdentity({ ...pages, expired: true }), receipt.pagesArtifact);
});

test("deleted historical artifacts retain the floor authenticated by the original publisher log", async () => {
  const { api, publisher, job, deployment, values, pages } = custodyFixture();
  for (const artifact of [...sourceArtifacts(), pages])
    values.delete("/actions/artifacts/" + artifact.id);
  assert.equal(
    (await qualifyDeployment(api, repositoryId, deployment, publisher, job)).sourceSha,
    sourceSha,
  );
  api.jobLog = async () => "The receipt is absent from the original publisher log";
  await assert.rejects(
    () => qualifyDeployment(api, repositoryId, deployment, publisher, job),
    /actual publisher log/,
  );
});

test("historical artifact authentication still rejects authorization and service failures", async () => {
  for (const status of [401, 403, 429, 500]) {
    const { api, publisher, job, deployment } = custodyFixture();
    api.response = async () => new Response(null, { status });
    await assert.rejects(
      () => qualifyDeployment(api, repositoryId, deployment, publisher, job),
      new RegExp("GitHub metadata request failed: " + status),
    );
  }
});

test("available historical metadata must still agree with the whole authenticated receipt", async () => {
  for (const kind of ["build", "pages"]) {
    const { api, publisher, job, deployment, values, pages } = custodyFixture();
    const artifact = kind === "pages" ? pages : sourceArtifacts()[0];
    values.set("/actions/artifacts/" + artifact.id, {
      ...artifact,
      digest: "sha256:" + "e".repeat(64),
    });
    await assert.rejects(() => qualifyDeployment(api, repositoryId, deployment, publisher, job));
  }
});

test("forged or missing receipt custody and unexecuted candidates cannot establish a publication floor", async () => {
  for (const mutation of [
    { creator: { login: "maintainer", type: "User" } },
    { sha: "e".repeat(40) },
    { task: "deploy" },
    { environment: "github-pages" },
    { created_at: "2026-10-10T01:36:00Z" },
  ]) {
    const { api, publisher, job, deployment } = custodyFixture();
    await assert.rejects(() =>
      qualifyDeployment(api, repositoryId, { ...deployment, ...mutation }, publisher, job),
    );
  }
  const { api, publisher, job, deployment, receipt, values } = custodyFixture();
  const originalLog = api.jobLog.bind(api);
  api.jobLog = async () => "Generic successful environment deployment";
  await assert.rejects(
    () => qualifyDeployment(api, repositoryId, deployment, publisher, job),
    /actual publisher log/,
  );
  api.jobLog = originalLog;
  await assert.rejects(() =>
    qualifyDeployment(api, repositoryId, deployment, publisher, {
      ...job,
      steps: job.steps.filter((step) => step.name !== "Record Pages artifact custody"),
    }),
  );
  await assert.rejects(() =>
    qualifyDeployment(api, repositoryId, deployment, publisher, {
      ...job,
      steps: job.steps.map((step) =>
        step.name === "Record Pages artifact custody" ? { ...step, conclusion: "skipped" } : step,
      ),
    }),
  );
  await assert.rejects(
    () =>
      qualifyDeployment(api, repositoryId, deployment, publisher, {
        ...job,
        steps: job.steps.map((step) =>
          step.name === "Deploy to GitHub Pages" ? { ...step, conclusion: "skipped" } : step,
        ),
      }),
    /unexecuted candidate/,
  );
  values.set(
    "/actions/runs/" + receipt.build.runId + "/attempts/1",
    run({ head_repository: { id: 999 } }),
  );
  await assert.rejects(
    () => qualifyDeployment(api, repositoryId, deployment, publisher, job),
    /fork source/,
  );
});

test("generic Pages environment records cannot introduce an unknown writer or success without a real job", async () => {
  const { api, publisher, job, deployment, values } = custodyFixture();
  let links = [
    {
      state: "success",
      log_url:
        "https://github.com/ubugeeei-prod/vize/actions/runs/" + publisher.id + "/job/" + job.id,
    },
  ];
  values.set("/actions/jobs/" + job.id, job);
  values.set("/deployments?environment=github-pages&per_page=100&page=1", [
    { ...deployment, environment: "github-pages" },
  ]);
  api.list = async <T>(path: string): Promise<T[]> => {
    assert.equal(path, "/deployments/" + deployment.id + "/statuses");
    return links as T[];
  };
  await verifySolePagesWriter(api, publisher.run_started_at, new Set([publisher.id]), 0);
  await assert.rejects(
    () => verifySolePagesWriter(api, publisher.run_started_at, new Set(), 0),
    /unknown Pages writer/,
  );
  links = [{ state: "success", log_url: "" }];
  await assert.rejects(
    () => verifySolePagesWriter(api, publisher.run_started_at, new Set([publisher.id]), 0),
    /without primary job/,
  );
  links = [{ state: "waiting", log_url: "" }];
  await assert.rejects(
    () => verifySolePagesWriter(api, publisher.run_started_at, new Set(), 0),
    /pending Pages writer without primary job/,
  );
  links = [{ state: "success", log_url: "https://github.com/other/repo/actions/runs/1/job/2" }];
  await assert.rejects(
    () => verifySolePagesWriter(api, publisher.run_started_at, new Set([publisher.id]), 0),
    /Repository-owned/,
  );
});
