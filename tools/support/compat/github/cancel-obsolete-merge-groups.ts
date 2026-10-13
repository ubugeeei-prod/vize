import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";

type Run = {
  id: number;
  event: string;
  status: string;
  head_sha: string;
  head_branch: string;
};
type Queue = {
  data: {
    repository: {
      mergeQueue: {
        entries: {
          pageInfo: { hasNextPage: boolean };
          nodes: { headCommit: { oid: string } | null }[];
        };
      };
    };
  };
};
type Api = (endpoint: string, options?: { method?: string; body?: unknown }) => Promise<unknown>;
export type Cancellation = { run: Run; reason: string; refSha?: string | null };
const activeStatuses = new Set(["queued", "in_progress", "waiting", "pending", "requested"]);
const queueBranch = /^gh-readonly-queue\/main\/pr-\d+-[a-f0-9]{40}$/u;

export async function cancelObsoleteMergeGroup(
  api: Api,
  repository: string,
  runId: number,
): Promise<Cancellation> {
  const prefix = `repos/${repository}`;
  const run = (await api(`${prefix}/actions/runs/${runId}`)) as Run;
  const result = { run, reason: "retained" };
  if (
    run.event !== "merge_group" ||
    !activeStatuses.has(run.status) ||
    !queueBranch.test(run.head_branch)
  ) {
    return result;
  }
  const ref = (await api(`${prefix}/git/ref/heads/${run.head_branch}`)) as {
    object: { sha: string };
  } | null;
  if (ref?.object.sha === run.head_sha) return { ...result, reason: "current-ref" };

  // Refresh after the ref lookup and immediately before cancellation. A rebuilt
  // candidate can have appeared since the workflow run was first listed.
  const [owner, name] = repository.split("/");
  const queue = (await api("graphql", {
    method: "POST",
    body: {
      query: `query($owner:String!, $name:String!) {
        repository(owner:$owner, name:$name) {
          mergeQueue(branch:"main") {
            entries(first:100) { pageInfo { hasNextPage } nodes { headCommit { oid } } }
          }
        }
      }`,
      variables: { owner, name },
    },
  })) as Queue;
  const entries = queue.data.repository.mergeQueue.entries;
  if (entries.pageInfo.hasNextPage !== false)
    throw new Error("Refusing cancellation with an incomplete queue");
  if (entries.nodes.some((entry) => entry.headCommit?.oid === run.head_sha)) {
    return { ...result, reason: "current-queue-head" };
  }
  const cancellation = await api(`${prefix}/actions/runs/${run.id}/cancel`, { method: "POST" });
  if (cancellation && typeof cancellation === "object" && "alreadyCompleted" in cancellation) {
    return { ...result, reason: "completed-during-check" };
  }
  return {
    run,
    reason: "cancel-requested: absent queue head and obsolete ref",
    refSha: ref?.object.sha ?? null,
  };
}

export async function cleanObsoleteMergeGroups(
  api: Api,
  repository: string,
  record: (results: Cancellation[]) => void = () => {},
): Promise<Cancellation[]> {
  const ids = new Set<number>();
  for (const status of ["queued", "in_progress", "waiting", "pending", "requested"]) {
    for (let page = 1; ; page += 1) {
      const response = (await api(
        `repos/${repository}/actions/runs?event=merge_group&status=${status}&per_page=100&page=${page}`,
      )) as { workflow_runs: Run[] };
      for (const run of response.workflow_runs) ids.add(run.id);
      if (response.workflow_runs.length < 100) break;
    }
  }
  const results: Cancellation[] = [];
  for (const id of ids) {
    results.push(await cancelObsoleteMergeGroup(api, repository, id));
    record(results);
  }
  return results;
}

async function main() {
  const repository = process.env.GITHUB_REPOSITORY;
  const token = process.env.GH_TOKEN;
  if (!repository || !/^[\w.-]+\/[\w.-]+$/u.test(repository) || !token) {
    throw new Error("GITHUB_REPOSITORY and GH_TOKEN are required");
  }
  const api: Api = async (endpoint, options = {}) => {
    const response = await fetch(`https://api.github.com/${endpoint}`, {
      method: options.method ?? "GET",
      signal: AbortSignal.timeout(30_000),
      headers: {
        Authorization: `Bearer ${token}`,
        Accept: "application/vnd.github+json",
        "X-GitHub-Api-Version": "2022-11-28",
      },
      ...(options.body ? { body: JSON.stringify(options.body) } : {}),
    });
    if (response.status === 404 && endpoint.includes("/git/ref/heads/")) return null;
    if (response.status === 409 && endpoint.endsWith("/cancel")) return { alreadyCompleted: true };
    if (!response.ok) throw new Error(`GitHub ${response.status}: ${endpoint}`);
    const body = await response.text();
    const value: unknown = body ? JSON.parse(body) : null;
    if (value && typeof value === "object" && "errors" in value)
      throw new Error("GitHub GraphQL failed");
    return value;
  };
  const evidencePath = path.join(process.env.RUNNER_TEMP ?? ".", "obsolete-merge-groups.json");
  const record = (results: Cancellation[]) => {
    fs.writeFileSync(
      evidencePath,
      JSON.stringify({ observedAt: new Date().toISOString(), repository, results }, null, 2) + "\n",
    );
  };
  record([]);
  const results = await cleanObsoleteMergeGroups(api, repository, record);
  const cancelled = results.filter((result) => result.reason.startsWith("cancel-requested")).length;
  const summary = `Requested cancellation of ${cancelled} obsolete merge-group runs; retained ${results.length - cancelled}.`;
  console.log(summary);
  if (process.env.GITHUB_STEP_SUMMARY)
    fs.appendFileSync(process.env.GITHUB_STEP_SUMMARY, `${summary}\n`);
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) await main();
