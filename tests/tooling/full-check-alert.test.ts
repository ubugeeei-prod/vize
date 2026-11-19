import assert from "node:assert/strict";
import test from "node:test";
import { parse } from "yaml";
import {
  type CompletedRun,
  decide,
  failedProducers,
  issueTitle,
  marker,
  reportFullCheck,
} from "../../tools/support/compat/github/full-check-alert.ts";
import { readRepoFile } from "./support/github-workflows.ts";

const repository = "owner/repo";
const run: CompletedRun = {
  id: 44465592435,
  event: "schedule",
  conclusion: "failure",
  head_branch: "main",
  head_sha: "e0e4d5a656".padEnd(40, "0"),
  html_url: "https://github.com/owner/repo/actions/runs/44465592435",
  created_at: "2026-11-19T05:30:09Z",
};
const jobs = [
  { name: "coverage", conclusion: "failure", html_url: "https://example.test/coverage" },
  { name: "vue-parity", conclusion: "success", html_url: "https://example.test/vue-parity" },
  {
    name: "Full qualification / rust-davinci-allocations",
    conclusion: "failure",
    html_url: "https://example.test/allocations",
  },
  {
    name: "Full qualification / clippy-and-test",
    conclusion: "failure",
    html_url: "https://example.test/aggregate",
  },
  { name: "full-js-report", conclusion: "failure", html_url: "https://example.test/report" },
  { name: "Nuxt 3 build and SSR", conclusion: "cancelled", html_url: "https://example.test/nuxt" },
];

function fakeApi(openIssues: { number: number; body: string | null; pull_request?: unknown }[]) {
  const calls: { endpoint: string; method: string; body?: unknown }[] = [];
  const api = async (endpoint: string, options: { method?: string; body?: unknown } = {}) => {
    const method = options.method ?? "GET";
    calls.push({ endpoint, method, body: options.body });
    if (method === "GET" && endpoint.startsWith(`repos/${repository}/issues?`)) return openIssues;
    if (method === "GET" && endpoint.includes("/jobs?")) return { jobs };
    if (method === "POST" && endpoint === `repos/${repository}/issues`) return { number: 99 };
    return {};
  };
  return { api, calls };
}

test("only full runs on the default branch are judged", () => {
  assert.equal(decide(run, "main"), "report-red");
  assert.equal(decide({ ...run, conclusion: "timed_out" }, "main"), "report-red");
  assert.equal(decide({ ...run, conclusion: "success" }, "main"), "report-green");
  assert.equal(decide({ ...run, conclusion: "cancelled" }, "main"), "ignore");
  assert.equal(decide({ ...run, event: "workflow_dispatch" }, "main"), "report-red");
  assert.equal(decide({ ...run, event: "pull_request" }, "main"), "ignore");
  assert.equal(decide({ ...run, event: "merge_group" }, "main"), "ignore");
  assert.equal(decide({ ...run, event: "push" }, "main"), "ignore");
  assert.equal(decide({ ...run, head_branch: "refactor/remove-codenames" }, "main"), "ignore");
});

test("failing producers exclude aggregate gates and cancelled jobs", () => {
  assert.deepEqual(
    failedProducers(jobs).map((job) => job.name),
    ["coverage", "Full qualification / rust-davinci-allocations"],
  );
});

test("a red full run opens one marked tracking issue listing its producers", async () => {
  const { api, calls } = fakeApi([]);
  const outcome = await reportFullCheck(api, repository, run, "main", ["maintainer"]);
  assert.deepEqual(outcome, { decision: "report-red", action: "opened", issue: 99 });
  const created = calls.find((call) => call.method === "POST");
  assert.equal(created?.endpoint, `repos/${repository}/issues`);
  const body = created?.body as { title: string; body: string; assignees: string[] };
  assert.equal(body.title, issueTitle);
  assert.deepEqual(body.assignees, ["maintainer"]);
  assert.equal(body.body.split("\n")[0], marker);
  assert.match(body.body, /\[coverage\]\(https:\/\/example\.test\/coverage\) \| failure \|/u);
  assert.doesNotMatch(body.body, /clippy-and-test|full-js-report|vue-parity|Nuxt 3/u);
  assert.ok(
    calls.some(
      (call) =>
        call.endpoint ===
        `repos/${repository}/actions/runs/${run.id}/jobs?filter=latest&per_page=100&page=1`,
    ),
  );
});

test("a further red run comments on the open issue instead of opening another", async () => {
  const { api, calls } = fakeApi([
    { number: 7, body: "unrelated" },
    { number: 8, body: `${marker}\nold`, pull_request: {} },
    { number: 12, body: `${marker}\nold` },
  ]);
  const outcome = await reportFullCheck(api, repository, run, "main");
  assert.deepEqual(outcome, { decision: "report-red", action: "commented", issue: 12 });
  assert.deepEqual(
    calls.filter((call) => call.method !== "GET").map((call) => call.endpoint),
    [`repos/${repository}/issues/12/comments`],
  );
});

test("a green full run closes the open issue and is silent without one", async () => {
  const green = { ...run, conclusion: "success" };
  const open = fakeApi([{ number: 12, body: `${marker}\nold` }]);
  assert.deepEqual(await reportFullCheck(open.api, repository, green, "main"), {
    decision: "report-green",
    action: "closed",
    issue: 12,
  });
  assert.deepEqual(
    open.calls.filter((call) => call.method !== "GET").map((call) => [call.method, call.endpoint]),
    [
      ["POST", `repos/${repository}/issues/12/comments`],
      ["PATCH", `repos/${repository}/issues/12`],
    ],
  );
  const none = fakeApi([]);
  assert.deepEqual(await reportFullCheck(none.api, repository, green, "main"), {
    decision: "report-green",
    action: "none",
  });
  assert.equal(none.calls.filter((call) => call.method !== "GET").length, 0);
});

test("ignored runs make no API calls", async () => {
  const { api, calls } = fakeApi([]);
  const outcome = await reportFullCheck(api, repository, { ...run, event: "pull_request" }, "main");
  assert.deepEqual(outcome, { decision: "ignore", action: "none" });
  assert.equal(calls.length, 0);
});

test("the listener watches Check completions with least privilege and trusted code", () => {
  const workflow = parse(readRepoFile(".github/workflows/check-full-alert.yml")) as {
    on: { workflow_run: { workflows: string[]; types: string[] } };
    permissions: Record<string, string>;
    jobs: { alert: { permissions: Record<string, string>; steps: { with?: { ref?: string } }[] } };
  };
  const check = parse(readRepoFile(".github/workflows/check.yml")) as { name: string };
  assert.deepEqual(workflow.on.workflow_run.workflows, [check.name]);
  assert.deepEqual(workflow.on.workflow_run.types, ["completed"]);
  assert.deepEqual(workflow.permissions, { contents: "read" });
  assert.deepEqual(workflow.jobs.alert.permissions, {
    contents: "read",
    actions: "read",
    issues: "write",
  });
  assert.equal(
    workflow.jobs.alert.steps[0]?.with?.ref,
    "${{ github.event.repository.default_branch }}",
  );
});
