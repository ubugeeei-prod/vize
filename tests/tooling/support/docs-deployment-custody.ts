import assert from "node:assert/strict";
import {
  GitHub,
  sha256,
  stableJson,
  type Deployment,
} from "../../../tools/support/compat/github/docs-deployment-api.ts";
import { artifactIdentity } from "../../../tools/support/compat/github/docs-deployment-policy.ts";
import { publisherFixture, run, sourceArtifacts } from "./docs-deployment.ts";

export function custodyFixture(publisherId = 38013719817) {
  const { publisher, job, pages, receipt } = publisherFixture();
  publisher.id = publisherId;
  job.run_id = publisherId;
  receipt.publisher.runId = publisherId;
  pages.workflow_run.id = publisherId;
  receipt.pagesArtifact = artifactIdentity(pages);
  job.steps.unshift(
    {
      name: "Validate completed build and actual Pages publication floor",
      conclusion: "success",
      started_at: "2026-10-10T01:34:58Z",
      completed_at: "2026-10-10T01:35:10Z",
    },
    {
      name: "Record Pages artifact custody",
      conclusion: "success",
      started_at: "2026-10-10T01:35:28Z",
      completed_at: "2026-10-10T01:35:29Z",
    },
  );
  const deployment: Deployment = {
    id: 99,
    created_at: "2026-10-10T01:35:28Z",
    sha: publisher.head_sha,
    task: receipt.schema,
    environment: "vize-docs-source-custody",
    creator: { login: "github-actions[bot]", type: "Bot" },
    payload: receipt,
  };
  const values = new Map<string, unknown>([
    ["/actions/runs/" + publisher.id + "/attempts/1", publisher],
    ["/actions/jobs/" + job.id, job],
    ["/actions/runs/" + receipt.build.runId + "/attempts/1", run()],
    ...sourceArtifacts().map((item): [string, unknown] => ["/actions/artifacts/" + item.id, item]),
    ["/actions/artifacts/" + pages.id, pages],
  ]);
  const api = new GitHub("ubugeeei-prod/vize", "fixture-credential");
  api.json = async <T>(path: string): Promise<T> => {
    assert(values.has(path), "Only exact primary identity endpoints: " + path);
    return values.get(path) as T;
  };
  api.jobLog = async (id: number) => {
    assert.equal(id, job.id);
    return "vize-docs-receipt:" + deployment.id + ":" + sha256(stableJson(receipt));
  };
  return { api, publisher, job, deployment, receipt, values, pages };
}
