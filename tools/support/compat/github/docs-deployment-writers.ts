import assert from "node:assert/strict";
import { GitHub, stableJson, type Deployment } from "./docs-deployment-api.ts";
import { validateRun, type PublisherJob, type WorkflowRun } from "./docs-deployment-policy.ts";
import { legacyWorkflowHash } from "./docs-deployment-custody.ts";

export async function verifyPendingWriter(
  api: GitHub,
  publisher: WorkflowRun,
  repositoryId: number,
  guardedWorkflowHash: string | null,
) {
  validateRun(publisher, repositoryId, ".github/workflows/deploy-docs.yml");
  assert.equal(publisher.event, "workflow_run");
  const hash = await api.workflowHash(publisher.head_sha);
  if (guardedWorkflowHash !== null) assert.match(guardedWorkflowHash, /^[a-f0-9]{64}$/);
  // Both exact writers lock the whole job. The legacy writer checks fresh main
  // after acquiring it; the current writer checks actual authenticated custody.
  assert(
    hash === legacyWorkflowHash || hash === guardedWorkflowHash,
    "A pending writer must prove its whole-job lock and publication guard before admission",
  );
}

export async function* metadataPages<T>(api: GitHub, path: string, key?: string) {
  let previous = "";
  for (let page = 1; ; page++) {
    const response = await api.json<T[] | Record<string, T[]>>(
      path + (path.includes("?") ? "&" : "?") + "per_page=100&page=" + page,
    );
    const values = key ? (response as Record<string, T[]>)[key] : (response as T[]);
    assert(Array.isArray(values));
    const signature = stableJson(values);
    assert(signature !== previous, "Primary metadata pagination advances");
    previous = signature;
    yield values;
    if (values.length < 100) return;
  }
}
export async function recentEnvironments(api: GitHub, since: string, retainedIds: number[]) {
  const selected = new Map<number, Deployment>();
  for await (const values of metadataPages<Deployment>(
    api,
    "/deployments?environment=github-pages",
  )) {
    for (const item of values)
      if (Date.parse(item.created_at) >= Date.parse(since)) selected.set(item.id, item);
    if (values.length && Date.parse(values.at(-1)!.created_at) < Date.parse(since)) break;
  }
  for (const id of retainedIds)
    if (!selected.has(id)) selected.set(id, await api.json<Deployment>("/deployments/" + id));
  return [...selected.values()];
}
export async function verifySolePagesWriter(
  api: GitHub,
  since: string,
  knownRuns: Set<number>,
  currentRunId: number,
  retainedIds: number[] = [],
  floorCompletedAt: string | null = null,
  floorJobId: number | null = null,
  repositoryId: number | null = null,
  guardedWorkflowHash: string | null = null,
) {
  const pending = new Set<number>();
  for (const record of await recentEnvironments(api, since, retainedIds)) {
    const statuses = await api.list<{ state: string; log_url: string }>(
      "/deployments/" + record.id + "/statuses",
    );
    const links = [...new Set(statuses.map((status) => status.log_url).filter(Boolean))];
    if (!links.length) {
      assert.fail("A pending Pages writer without primary job custody refuses publication");
    }
    for (const link of links) {
      const prefix = "https://github.com/" + api.repository + "/actions/runs/";
      assert(link.startsWith(prefix), "Repository-owned Pages publisher job URL");
      const match = /^(\d+)\/job\/(\d+)$/.exec(link.slice(prefix.length));
      assert(match, "Whole primary Pages publisher job URL");
      const job = await api.json<PublisherJob>("/actions/jobs/" + match[2]);
      assert.equal(job.run_id, Number(match[1]));
      assert.equal(
        job.head_sha,
        record.sha,
        "Environment record binds its actual publisher context",
      );
      if (job.run_id === currentRunId) continue;
      const page = job.steps.find((step) => step.name === "Deploy to GitHub Pages");
      if (job.status !== "completed" && (!page || page.started_at === null)) {
        assert(repositoryId !== null, "Primary pending writer repository identity");
        const publisher = await api.json<WorkflowRun>(
          "/actions/runs/" + job.run_id + "/attempts/" + job.run_attempt,
        );
        await verifyPendingWriter(api, publisher, repositoryId, guardedWorkflowHash);
        pending.add(record.id);
        continue;
      }
      let trusted = knownRuns.has(job.run_id);
      if (!trusted && repositoryId !== null) {
        const publisher = await api.json<WorkflowRun>(
          "/actions/runs/" + job.run_id + "/attempts/" + job.run_attempt,
        );
        if (publisher.path === ".github/workflows/deploy-docs.yml") {
          validateRun(publisher, repositoryId, publisher.path);
          assert.equal(publisher.event, "workflow_run");
          trusted = true;
        }
      }
      if (trusted && page?.conclusion === "skipped") continue;
      assert(
        !page || (page.conclusion !== null && page.completed_at),
        "An uncertain actual Pages effect refuses publication even without a configured site",
      );
      if (trusted && floorJobId === job.id && page?.conclusion === "success") continue;
      if (floorCompletedAt !== null)
        assert(
          page?.completed_at &&
            page.conclusion !== null &&
            Date.parse(page.completed_at) < Date.parse(floorCompletedAt),
          "An uncertain later Pages environment effect refuses publication",
        );
      if (!knownRuns.has(job.run_id)) {
        assert(
          trusted ||
            (page &&
              page.conclusion !== null &&
              page.completed_at &&
              floorCompletedAt &&
              Date.parse(page.completed_at) < Date.parse(floorCompletedAt)),
          "An unknown Pages writer cannot bypass source custody",
        );
      }
      if (job.status !== "completed") pending.add(record.id);
    }
  }
  return [...pending].sort((a, b) => a - b);
}
