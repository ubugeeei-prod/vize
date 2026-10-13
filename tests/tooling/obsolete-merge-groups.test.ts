import assert from "node:assert/strict";
import test from "node:test";
import {
  cancelObsoleteMergeGroup,
  cleanObsoleteMergeGroups,
} from "../../tools/support/compat/github/cancel-obsolete-merge-groups.ts";

const sha = "a".repeat(40);
const run = {
  id: 1,
  event: "merge_group",
  status: "in_progress",
  head_sha: sha,
  head_branch: `gh-readonly-queue/main/pr-7-${"b".repeat(40)}`,
};
function fixture({
  current = run,
  ref = null,
  heads = [],
  incomplete = false,
}: {
  current?: typeof run;
  ref?: { object: { sha: string } } | null;
  heads?: string[];
  incomplete?: boolean;
} = {}) {
  const calls: string[] = [];
  const api = async (endpoint: string) => {
    calls.push(endpoint);
    if (endpoint.includes("/git/ref/")) return ref;
    if (endpoint === "graphql")
      return {
        data: {
          repository: {
            mergeQueue: {
              entries: {
                pageInfo: { hasNextPage: incomplete },
                nodes: heads.map((oid) => ({ headCommit: { oid } })),
              },
            },
          },
        },
      };
    if (endpoint.endsWith("/cancel")) return {};
    if (endpoint.includes("?event="))
      return { workflow_runs: endpoint.includes("status=queued") ? [run] : [] };
    return current;
  };
  return { api, calls };
}

void test("cancels only after checking the obsolete ref and then refreshing the queue", async () => {
  const { api, calls } = fixture();
  const result = await cancelObsoleteMergeGroup(api, "owner/repo", 1);
  assert.match(result.reason, /^cancel-requested/u);
  assert.deepEqual(calls, [
    "repos/owner/repo/actions/runs/1",
    `repos/owner/repo/git/ref/heads/${run.head_branch}`,
    "graphql",
    "repos/owner/repo/actions/runs/1/cancel",
  ]);
});

void test("retains a live queue candidate even after its temporary ref disappears", async () => {
  const { api, calls } = fixture({ heads: [sha] });
  assert.equal((await cancelObsoleteMergeGroup(api, "owner/repo", 1)).reason, "current-queue-head");
  assert.equal(
    calls.some((call) => call.endsWith("/cancel")),
    false,
  );
});

void test("retains a current ref and never cancels PRs, releases, completed runs, or other queues", async () => {
  for (const current of [
    { ...run, event: "pull_request" },
    { ...run, event: "workflow_dispatch" },
    { ...run, status: "completed" },
    { ...run, head_branch: run.head_branch.replace("/main/", "/other/") },
  ]) {
    const { api, calls } = fixture({ current });
    await cancelObsoleteMergeGroup(api, "owner/repo", 1);
    assert.equal(calls.length, 1);
  }
  const { api, calls } = fixture({ ref: { object: { sha } } });
  assert.equal((await cancelObsoleteMergeGroup(api, "owner/repo", 1)).reason, "current-ref");
  assert.equal(calls.length, 2);
});

void test("fails closed on incomplete or unavailable queue evidence", async () => {
  const { api, calls } = fixture({ incomplete: true });
  await assert.rejects(cancelObsoleteMergeGroup(api, "owner/repo", 1), /incomplete queue/u);
  assert.equal(
    calls.some((call) => call.endsWith("/cancel")),
    false,
  );
  await assert.rejects(
    cancelObsoleteMergeGroup(
      async () => {
        throw new Error("API unavailable");
      },
      "owner/repo",
      1,
    ),
    /API unavailable/u,
  );
});

void test("lists only merge-group statuses and revalidates each candidate before cancellation", async () => {
  const { api, calls } = fixture();
  assert.equal((await cleanObsoleteMergeGroups(api, "owner/repo")).length, 1);
  assert.equal(calls.filter((call) => call.includes("?event=merge_group&status=")).length, 5);
  assert.equal(calls.at(-1), "repos/owner/repo/actions/runs/1/cancel");
});

void test("reports a completed cancellation race without treating it as accepted", async () => {
  const { api } = fixture();
  const result = await cancelObsoleteMergeGroup(
    async (endpoint) => (endpoint.endsWith("/cancel") ? { alreadyCompleted: true } : api(endpoint)),
    "owner/repo",
    1,
  );
  assert.equal(result.reason, "completed-during-check");
});
