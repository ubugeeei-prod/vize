import assert from "node:assert/strict";
import { readRepoFile } from "../support/github-workflows.ts";
import { requiredWorkflowJobNames } from "../../../tools/support/compat/github/release-preflight-evidence.mjs";
import type { Row, ReadApi } from "../../../tools/support/release/qualify_check/check-state.ts";

export const repository = "owner/repo";
export const H = "a".repeat(40),
  C = "b".repeat(40),
  candidate = "c".repeat(40);
export const base = "d".repeat(40),
  integrationHead = "e".repeat(40),
  leaseSha = "f".repeat(40);
export const pinSha = "1".repeat(40),
  tree = "2".repeat(40);
export const created = Date.parse("2026-10-17T00:00:00Z");
const retainedIntegrationBody = JSON.parse(
  readRepoFile("tests/_fixtures/release/integration-body-447/current-pr.json"),
).body as string;
export const integrationBody = retainedIntegrationBody
  .replaceAll("dd188515f8ec3c51662b4f4d69adcd39c86a0907", C)
  .replaceAll("ef34d1a0409fc711a6c5cefcf0708b0cce78f502", H)
  .replaceAll("v0.447.0", "v0.500.0")
  .replaceAll("8739", "42")
  .replaceAll("8740", "99")
  .replaceAll("ubugeeei-prod/codesmith/vize", "owner/codesmith/repo")
  .replaceAll("ubugeeei-prod%2Fvize", "owner%2Frepo");
export const context = {
  integration: 99,
  source: 42,
  H,
  C,
  tag: "v0.500.0",
  candidate,
  base,
  integrationHead,
  integrationBody,
  event: "merge_group",
  catalogEqual: true,
  catalogAuthority: "main-integration",
  metadataExact: true,
};
export const clone = <T>(value: T): T => structuredClone(value);

export function fixture(
  legacy = false,
  tag = context.tag,
  qAuthor = "maintainer",
  baseVersion = "0.499.0",
) {
  const sourceBody = [
    "<!-- vize-release-pin: immutable-v1 -->",
    `<!-- vize-release-pin-head: ${H} -->`,
    `<!-- vize-release-pin-cut: ${C} -->`,
    "<!-- vize-release-integration: 99 -->",
    `<!-- vize-release-base-version: ${baseVersion} -->`,
    "<!-- vize-release-bump: minor -->",
  ].join("\n");
  const currentIntegrationBody = integrationBody.replaceAll(context.tag, tag);
  const author = { login: "maintainer", type: "User" };
  const pr = (number: number, branch: string, head: string, draft: boolean, body: string) => ({
    number,
    state: "open",
    merged: false,
    draft,
    body,
    user: author,
    head: { ref: branch, sha: head, repo: { full_name: repository } },
    base: { ref: "main", sha: base, repo: { full_name: repository } },
  });
  const ref = (branch: string, id: string) => ({
    ref: `refs/heads/${branch}`,
    object: { type: "commit", sha: id },
  });
  const commit = (id: string, parent: string, message = "") => ({
    sha: id,
    tree: { sha: tree },
    parents: [{ sha: parent }],
    message,
  });
  const originalDeadline = created + 60_000;
  const lease = legacy
    ? `Vize release operator\n\nTag: ${tag}\nSource-head: ${H}\nPID: 10\nNonce: 20\n`
    : `Vize release operator v2\n\nTag: ${tag}\nSource-head: ${H}\nOperator-run: 5\nOperator-attempt: 2\nOperator-source: ${C}\nOriginal-deadline-unix-millis: ${originalDeadline}\nPID: 10\nNonce: 20\n`;
  const records: Record<string, Row> = {
    "pulls/42": pr(42, `release/${tag}`, H, true, sourceBody),
    "pulls/99": pr(
      99,
      `release-integration/${tag}`,
      integrationHead,
      false,
      currentIntegrationBody,
    ),
    "collaborators/maintainer/permission": { role_name: "maintain" },
    "collaborators/resumer/permission": { role_name: "maintain" },
    [`git/ref/heads/release/${tag}`]: ref(`release/${tag}`, H),
    [`git/ref/heads/release-integration/${tag}`]: ref(
      `release-integration/${tag}`,
      integrationHead,
    ),
    [`git/ref/heads/release-pin/${tag}`]: ref(`release-pin/${tag}`, pinSha),
    [`git/ref/heads/release-operator/${tag}`]: ref(`release-operator/${tag}`, leaseSha),
    [`git/commits/${H}`]: commit(H, C),
    [`git/commits/${candidate}`]: commit(candidate, base),
    [`git/commits/${pinSha}`]: commit(
      pinSha,
      H,
      `Vize immutable release cut\n\nSource-PR: #42\nSource-head: ${H}\nSource-cut: ${C}\nTag: ${tag}\nBase-version: ${baseVersion}\nIntegration-PR: #99\n`,
    ),
    [`git/commits/${leaseSha}`]: commit(leaseSha, H, lease),
    "actions/runs/5": {
      id: 5,
      run_attempt: 2,
      head_sha: C,
      head_branch: "main",
      event: "workflow_dispatch",
      path: ".github/workflows/release-operator.yml",
      status: "in_progress",
      conclusion: null,
      repository: { full_name: repository },
      head_repository: { full_name: repository },
      actor: author,
      triggering_actor: author,
      workflow_id: 90,
    },
    "actions/workflows/release-operator.yml": {
      id: 90,
      path: ".github/workflows/release-operator.yml",
      name: "Release Operator",
    },
    [`contents/.github/workflows/release-operator.yml?ref=${C}`]: {
      encoding: "base64",
      content: Buffer.from("name: Release Operator\n").toString("base64"),
    },
  };
  let runs: Row[] = [
    {
      id: 7,
      run_attempt: 3,
      status: "completed",
      conclusion: "success",
      head_sha: H,
      head_branch: `release/${tag}`,
      event: "workflow_dispatch",
      path: ".github/workflows/check.yml",
      display_title: `Check full @ ${H}`,
      created_at: new Date(created).toISOString(),
      repository: { full_name: repository },
      head_repository: { full_name: repository },
      actor: { ...author, login: qAuthor },
      triggering_actor: { ...author, login: qAuthor },
    },
  ];
  let jobs: Row[] = requiredWorkflowJobNames("Check", runs[0]).map(
    (name: string, index: number) => ({
      id: 100 + index,
      name,
      run_id: 7,
      run_attempt: 3,
      head_sha: H,
      status: "completed",
      conclusion: "success",
    }),
  );
  const calls: string[] = [],
    snapshots: unknown[] = [],
    sleeps: number[] = [],
    pendingDeadlines: number[] = [];
  let time = created + 1_000;
  const state = {
    records,
    runs,
    jobs,
    calls,
    snapshots,
    sleeps,
    pendingDeadlines,
    time,
    onSleep: () => {},
  };
  const api = new Proxy(
    {
      read: async (resource: string) => {
        calls.push(`GET ${resource}`);
        assert.ok(records[resource], `unexpected GET ${resource}`);
        return clone(records[resource]);
      },
      pages: async (resource: string, collection: string, query?: Record<string, string>) => {
        calls.push(`PAGES ${resource}`);
        if (resource === "actions/runs") {
          assert.equal(collection, "workflow_runs");
          assert.deepEqual(query, { head_sha: H });
          return clone(state.runs);
        }
        assert.equal(resource, "actions/runs/7/jobs");
        assert.equal(collection, "jobs");
        assert.deepEqual(query, { filter: "latest" });
        return clone(state.jobs);
      },
    },
    {
      get(target, property, receiver) {
        assert.ok(
          property === "read" || property === "pages",
          `forbidden write/API capability ${String(property)}`,
        );
        return Reflect.get(target, property, receiver);
      },
    },
  ) as ReadApi;
  const options = {
    repository,
    candidateSha: candidate,
    mode: "wait" as "wait" | "verify",
    api,
    now: () => state.time,
    sleep: async (milliseconds: number) => {
      sleeps.push(milliseconds);
      state.time += milliseconds;
      state.onSleep();
    },
    log: () => {},
    observe: (snapshot: unknown) => snapshots.push(snapshot),
    onPendingDeadline: (deadline: number) => pendingDeadlines.push(deadline),
  };
  return {
    ...state,
    options,
    get time() {
      return state.time;
    },
    set time(value: number) {
      state.time = value;
    },
    get onSleep() {
      return state.onSleep;
    },
    set onSleep(value: () => void) {
      state.onSleep = value;
    },
    originalDeadline,
  };
}
