import assert from "node:assert/strict";
import test from "node:test";
import { parse } from "yaml";
import {
  cancelObsoleteMergeGroup,
  cleanObsoleteMergeGroups,
} from "../../tools/support/compat/github/cancel-obsolete-merge-groups.ts";
import { readRepoFile } from "./support/github-workflows.ts";

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

void test("coalesces overlapping cleanup runs", () => {
  const workflow = parse(
    readRepoFile(".github", "workflows", "cleanup-obsolete-merge-groups.yml"),
  ) as { concurrency: { group: string; "cancel-in-progress": boolean } };
  assert.deepEqual(workflow.concurrency, {
    group: "cleanup-obsolete-merge-groups",
    "cancel-in-progress": true,
  });
});

void test("uses one complete queue snapshot for every candidate in a batch", async () => {
  const retained = {
    ...run,
    id: 2,
    head_sha: "c".repeat(40),
    head_branch: `gh-readonly-queue/main/pr-8-${"d".repeat(40)}`,
  };
  const calls: string[] = [];
  const api = async (endpoint: string) => {
    calls.push(endpoint);
    if (endpoint === "graphql")
      return {
        data: {
          repository: {
            mergeQueue: {
              entries: {
                pageInfo: { hasNextPage: false },
                nodes: [{ headCommit: { oid: retained.head_sha } }],
              },
            },
          },
        },
      };
    if (endpoint.includes("/git/ref/")) return null;
    if (endpoint.endsWith("/cancel")) return {};
    if (endpoint.includes("?event="))
      return { workflow_runs: endpoint.includes("status=queued") ? [run, retained] : [] };
    if (endpoint.endsWith("/1")) return run;
    if (endpoint.endsWith("/2")) return retained;
    throw new Error(`Unexpected API call: ${endpoint}`);
  };
  const results = await cleanObsoleteMergeGroups(api, "owner/repo");
  assert.deepEqual(
    results.map(({ run: current, reason }) => [current.id, reason]),
    [
      [1, "cancel-requested: absent queue head and obsolete ref"],
      [2, "current-queue-head"],
    ],
  );
  assert.equal(calls.filter((call) => call === "graphql").length, 1);
  assert.deepEqual(
    calls.filter((call) => call.endsWith("/cancel")),
    ["repos/owner/repo/actions/runs/1/cancel"],
  );
  assert.ok(calls.indexOf("graphql") < calls.indexOf("repos/owner/repo/actions/runs/1"));
});

void test("retains a live queue candidate even after its temporary ref disappears", async () => {
  const { api, calls } = fixture();
  assert.equal(
    (await cancelObsoleteMergeGroup(api, "owner/repo", 1, new Set([sha]))).reason,
    "current-queue-head",
  );
  assert.equal(calls.includes("graphql"), false);
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
    await cancelObsoleteMergeGroup(api, "owner/repo", 1, new Set());
    assert.equal(calls.length, 1);
  }
  const { api, calls } = fixture({ ref: { object: { sha } } });
  assert.equal(
    (await cancelObsoleteMergeGroup(api, "owner/repo", 1, new Set())).reason,
    "current-ref",
  );
  assert.equal(calls.length, 2);
});

void test("fails closed on incomplete or unavailable queue evidence", async () => {
  const { api, calls } = fixture({ incomplete: true });
  await assert.rejects(cleanObsoleteMergeGroups(api, "owner/repo"), /incomplete queue/u);
  assert.equal(
    calls.some((call) => call.endsWith("/cancel")),
    false,
  );
  await assert.rejects(
    cleanObsoleteMergeGroups(async (endpoint) => {
      if (endpoint === "graphql") throw new Error("API unavailable");
      return api(endpoint);
    }, "owner/repo"),
    /API unavailable/u,
  );

  const unresolved = fixture();
  await assert.rejects(
    cleanObsoleteMergeGroups(async (endpoint) => {
      if (endpoint !== "graphql") return unresolved.api(endpoint);
      return {
        data: {
          repository: {
            mergeQueue: {
              entries: { pageInfo: { hasNextPage: false }, nodes: [{ headCommit: null }] },
            },
          },
        },
      };
    }, "owner/repo"),
    /unresolved queue head/u,
  );
  assert.equal(
    unresolved.calls.some((call) => call.endsWith("/cancel")),
    false,
  );
});

void test("lists only merge-group statuses and revalidates each candidate before cancellation", async () => {
  const { api, calls } = fixture();
  assert.equal((await cleanObsoleteMergeGroups(api, "owner/repo")).length, 1);
  assert.equal(calls.filter((call) => call.includes("?event=merge_group&status=")).length, 5);
  assert.equal(calls.filter((call) => call === "graphql").length, 1);
  assert.equal(calls.at(-1), "repos/owner/repo/actions/runs/1/cancel");
});

void test("reports a completed cancellation race without treating it as accepted", async () => {
  const { api } = fixture();
  const result = await cancelObsoleteMergeGroup(
    async (endpoint) => (endpoint.endsWith("/cancel") ? { alreadyCompleted: true } : api(endpoint)),
    "owner/repo",
    1,
    new Set(),
  );
  assert.equal(result.reason, "completed-during-check");
});
