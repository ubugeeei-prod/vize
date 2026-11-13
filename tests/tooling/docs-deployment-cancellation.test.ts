import assert from "node:assert/strict";
import { test } from "node:test";
import { publicationState } from "../../tools/support/compat/github/docs-deployment-history.ts";
import type { PublisherJob } from "../../tools/support/compat/github/docs-deployment-policy.ts";
import { history } from "./support/docs-deployment-history.ts";
import { repositoryId, run, sourceSha } from "./support/docs-deployment.ts";

const id = sourceSha;
const lateFailure = "/later Pages effect lacks authenticated/";
// The actions/deploy-pages timeout path, as run 42220058904 logged it on 2026-11-06.
const timedOut = [
  `2026-10-10T01:40:01.9680366Z Created deployment for ${sourceSha}, ID: ${id}`,
  "2026-10-10T01:40:02.4333441Z Current status: syncing_files",
  "2026-10-10T01:40:09.0919073Z ##[error]Timeout reached, aborting!",
  "2026-10-10T01:40:09.0924261Z Canceling Pages deployment...",
  `2026-10-10T01:40:09.6585551Z Canceled deployment with ID ${id}`,
].join("\n");

/** A completed publisher whose Pages step failed after the authenticated floor. */
function timedOutPublisher() {
  const value = history();
  const publisher = run({
    id: 88001,
    path: ".github/workflows/deploy-docs.yml",
    event: "workflow_run",
    conclusion: "failure",
  });
  const job: PublisherJob = {
    id: 77001,
    run_id: publisher.id,
    run_attempt: 1,
    head_sha: publisher.head_sha,
    status: "completed",
    steps: [
      {
        name: "Deploy to GitHub Pages",
        conclusion: "failure",
        started_at: "2026-10-10T01:40:00Z",
        completed_at: "2026-10-10T01:40:09Z",
      },
    ],
  };
  value.receipt.outstanding.publisherAttempts.push({ runId: publisher.id, attempt: 1 });
  value.values.set("/actions/runs/" + publisher.id + "/attempts/1", publisher);
  value.values.set("/actions/jobs/" + job.id, job);
  value.values.set("/pages/deployments/" + id, { status: "" });
  value.lists.set("/actions/runs/" + publisher.id + "/attempts/1/jobs", [job]);
  value.logs.set(job.id, timedOut);
  // The environment record Actions keeps for that job also postdates the floor.
  value.values.set("/deployments?environment=github-pages&per_page=100&page=1", [
    {
      ...value.deployment,
      id: 556,
      environment: "github-pages",
      created_at: "2026-10-10T01:39:00Z",
    },
  ]);
  value.lists.set("/deployments/556/statuses", [
    {
      state: "failure",
      log_url: "https://github.com/ubugeeei-prod/vize/actions/runs/88001/job/77001",
    },
  ]);
  return { ...value, timedOutJob: job };
}

test("a cancelled timed-out Pages deployment leaves the authenticated floor in place", async () => {
  const { api, publisher } = timedOutPublisher();
  const state = await publicationState(api, repositoryId, 0);
  assert.equal(state.floor?.receipt.publisher.runId, publisher.id);
  assert.equal(state.floor?.sourceSha, sourceSha);
  assert.deepEqual(state.outstanding.publisherAttempts, []);
});

test("a failed Pages step stays uncertain without whole cancellation evidence", async () => {
  const mutations: Array<[string, (f: ReturnType<typeof timedOutPublisher>) => void]> = [
    ["no cancellation", (f) => f.logs.set(77001, timedOut.split("\n").slice(0, 3).join("\n"))],
    [
      "other deployment",
      (f) => f.logs.set(77001, timedOut.replace(`with ID ${id}`, "with ID " + "c".repeat(40))),
    ],
    [
      "other source",
      (f) => f.logs.set(77001, timedOut.replace(`for ${sourceSha}`, "for " + "d".repeat(40))),
    ],
    [
      "reported success",
      (f) => f.logs.set(77001, timedOut + "\n2026-10-10T01:40:09.7Z Reported success!"),
    ],
    ["two deployments", (f) => f.logs.set(77001, timedOut + "\n" + timedOut.split("\n")[0])],
    [
      "cancel after the step",
      (f) => f.logs.set(77001, timedOut.replace("01:40:09.6585551Z", "01:40:11.0Z")),
    ],
    [
      "created before the step",
      (f) => f.logs.set(77001, timedOut.replace("01:40:01.9680366Z", "01:39:59.0Z")),
    ],
    [
      "Pages reports success",
      (f) => f.values.set("/pages/deployments/" + id, { status: "succeed" }),
    ],
    ["Pages state unavailable", (f) => f.values.delete("/pages/deployments/" + id)],
    ["cancelled step", (f) => (f.timedOutJob.steps[0]!.conclusion = "cancelled")],
    ["successful step", (f) => (f.timedOutJob.steps[0]!.conclusion = "success")],
  ];
  for (const [name, mutate] of mutations) {
    const f = timedOutPublisher();
    mutate(f);
    await assert.rejects(
      () => publicationState(f.api, repositoryId, 0),
      new RegExp(lateFailure.slice(1, -1)),
      name,
    );
  }
});
