import assert from "node:assert/strict";
import { test } from "node:test";
import { Script } from "node:vm";
import { parse } from "yaml";

import { readRepoFile } from "./support/github-workflows.ts";

test("zizmor workflow audits GitHub Actions with pinned security scanning", () => {
  const workflow = readRepoFile(".github", "workflows", "zizmor.yml");
  const parsed = parse(workflow) as {
    name?: string;
    on?: {
      push?: { branches?: string[]; paths?: string[] };
      pull_request?: { branches?: string[]; paths?: string[] };
      release?: { types?: string[] };
      schedule?: Array<{ cron?: string }>;
      workflow_dispatch?: unknown;
    };
    concurrency?: {
      group?: string;
      "cancel-in-progress"?: boolean;
    };
    permissions?: Record<string, never>;
    env?: Record<string, unknown>;
    jobs?: {
      zizmor?: {
        name?: string;
        needs?: string;
        if?: string;
        "runs-on"?: string;
        "timeout-minutes"?: number;
        "continue-on-error"?: boolean;
        permissions?: Record<string, string>;
        steps?: Array<{
          name?: string;
          uses?: string;
          "continue-on-error"?: boolean;
          with?: Record<string, unknown>;
        }>;
      };
    };
  };

  assert.equal(parsed.name, "Zizmor");
  assert.deepEqual(parsed.permissions, {});
  assert.equal(parsed.env?.FORCE_JAVASCRIPT_ACTIONS_TO_NODE24, true);
  assert.deepEqual(parsed.on?.push?.branches, ["main", "davinci"]);
  assert.deepEqual(parsed.on?.push?.paths, [".github/**"]);
  assert.deepEqual(parsed.on?.pull_request, {});
  assert.deepEqual(parsed.on?.release?.types, ["published"]);
  assert.deepEqual(parsed.on?.schedule, [{ cron: "17 2 * * 2" }]);
  assert.ok(Object.hasOwn(parsed.on ?? {}, "workflow_dispatch"));
  assert.equal(Object.hasOwn(parsed.on ?? {}, "pull_request_target"), false);
  assert.deepEqual(parsed.concurrency, {
    "cancel-in-progress": true,
    group: "zizmor-${{ github.workflow }}-${{ github.event.pull_request.number || github.sha }}",
  });

  const job = parsed.jobs?.zizmor;
  assert.ok(job, "missing zizmor job");
  assert.equal(job.name, "Run zizmor");
  assert.equal(job.needs, "plan");
  assert.equal(
    job.if,
    "${{ !cancelled() && (needs.plan.result != 'success' || needs.plan.outputs.audit != 'false') }}",
  );
  assert.equal(job["runs-on"], "blacksmith-32vcpu-ubuntu-2404");
  assert.equal(job["timeout-minutes"], 10);
  assert.notEqual(job["continue-on-error"], true);
  assert.deepEqual(job.permissions, {
    actions: "read",
    contents: "read",
    "security-events": "write",
  });

  const checkout = job.steps?.find((step) => step.uses?.startsWith("actions/checkout@"));
  assert.ok(checkout, "missing checkout step");
  assert.match(checkout.uses ?? "", /^actions\/checkout@[0-9a-f]{40}$/);
  assert.equal(checkout.with?.["persist-credentials"], false);

  const scan = job.steps?.find((step) => step.uses?.startsWith("zizmorcore/zizmor-action@"));
  assert.ok(scan, "missing zizmor action step");
  assert.equal(scan.uses, "zizmorcore/zizmor-action@70fb788f84895a7701f5643d103d587e460b5c99");
  assert.deepEqual(scan.with, {
    "advanced-security":
      "${{ github.event_name != 'pull_request' || github.event.pull_request.head.repo.full_name == github.repository }}",
    inputs: ".github",
    "min-confidence": "high",
    "min-severity": "high",
    "online-audits": true,
    version: "1.30.0",
  });
  for (const step of job.steps ?? []) {
    assert.notEqual(step["continue-on-error"], true, `${step.name ?? step.uses} must fail closed`);
  }
});

test("zizmor plans with read-only API calls and no candidate checkout", () => {
  const workflow = parse(readRepoFile(".github", "workflows", "zizmor.yml"));
  const plan = workflow.jobs.plan;
  assert.equal(plan["runs-on"], "ubuntu-24.04");
  assert.equal(plan["timeout-minutes"], 5);
  assert.deepEqual(plan.permissions, { "pull-requests": "read" });
  assert.deepEqual(plan.outputs, { audit: "${{ steps.plan.outputs.audit }}" });
  assert.equal(plan.steps.length, 1);
  assert.equal(plan.steps[0].id, "plan");
  assert.equal(
    plan.steps[0].uses,
    "actions/github-script@ed597411d8f924073f98dfc5c65a23a2325f34cd",
  );
  assert.doesNotMatch(plan.steps[0].with.script, /require\(|import\(|exec\(|spawn\(/);
});

test("a failed or incomplete audit plan cannot skip the scanner", () => {
  const workflow = parse(readRepoFile(".github", "workflows", "zizmor.yml"));
  const condition = workflow.jobs.zizmor.if.slice(3, -2).trim();
  const evaluate = (needs: object, cancelled: () => boolean) =>
    new Script(condition).runInNewContext({ needs, cancelled });
  for (const [result, audit, expected] of [
    ["success", "false", false],
    ["success", "true", true],
    ["success", undefined, true],
    ["success", "", true],
    ["failure", "false", true],
    ["skipped", "false", true],
    ["cancelled", "false", true],
  ] as const) {
    assert.equal(
      evaluate({ plan: { result, outputs: { audit } } }, () => false),
      expected,
    );
  }
  assert.equal(
    evaluate({ plan: { result: "success", outputs: { audit: "true" } } }, () => true),
    false,
  );
});

type SelectionCase = {
  eventName?: string;
  association?: string;
  fork?: boolean;
  files?: Array<{ filename: string; previous_filename?: string }>;
  changedFiles?: number;
  afterChangedFiles?: number;
  beforeHead?: string;
  afterHead?: string;
  apiFailure?: "get" | "paginate";
};

async function runSelector(options: SelectionCase = {}) {
  const workflow = parse(readRepoFile(".github", "workflows", "zizmor.yml"));
  const script = workflow.jobs.plan.steps[0].with.script as string;
  const AsyncFunction = Object.getPrototypeOf(async function () {}).constructor;
  const select = new AsyncFunction("context", "github", "core", script);
  const head = "a".repeat(40);
  const files = options.files ?? [{ filename: "davinci/vize_l1/src/lib.rs" }];
  const changedFiles = options.changedFiles ?? files.length;
  const context = {
    eventName: options.eventName ?? "pull_request",
    repo: { owner: "ubugeeei-prod", repo: "vize" },
    payload: {
      pull_request: {
        number: 42,
        author_association: options.association ?? "OWNER",
        head: {
          sha: head,
          repo: { full_name: options.fork ? "external/vize" : "ubugeeei-prod/vize" },
        },
      },
    },
  };
  const warnings: string[] = [];
  const outputs: Record<string, string> = {};
  let requests = 0;
  let listed = false;
  await select(
    context,
    {
      rest: {
        pulls: {
          get: async () => {
            requests++;
            if (options.apiFailure === "get") throw new Error("GitHub API unavailable");
            return {
              data: {
                head: {
                  sha: requests === 1 ? (options.beforeHead ?? head) : (options.afterHead ?? head),
                },
                changed_files:
                  requests === 1 ? changedFiles : (options.afterChangedFiles ?? changedFiles),
              },
            };
          },
          listFiles: () => {},
        },
      },
      paginate: async (_method: unknown, args: Record<string, unknown>) => {
        assert.deepEqual(args, {
          owner: "ubugeeei-prod",
          repo: "vize",
          pull_number: 42,
          per_page: 100,
        });
        listed = true;
        if (options.apiFailure === "paginate") throw new Error("File API unavailable");
        return files;
      },
    },
    {
      setOutput: (name: string, value: string) => {
        outputs[name] = value;
      },
      warning: (message: string) => warnings.push(message),
      summary: { addRaw: () => ({ write: async () => {} }) },
    },
  );
  return { audit: outputs.audit, warnings, requests, listed };
}

test("only complete, unchanged maintainer source PRs skip zizmor", async () => {
  for (const association of ["OWNER", "MEMBER", "COLLABORATOR"]) {
    assert.deepEqual(await runSelector({ association }), {
      audit: "false",
      warnings: [],
      requests: 2,
      listed: true,
    });
  }
  for (const association of [
    "NONE",
    "FIRST_TIMER",
    "FIRST_TIME_CONTRIBUTOR",
    "CONTRIBUTOR",
    "MANNEQUIN",
    "",
  ]) {
    assert.deepEqual(await runSelector({ association }), {
      audit: "true",
      warnings: [],
      requests: 0,
      listed: false,
    });
  }
  assert.deepEqual(await runSelector({ fork: true }), {
    audit: "true",
    warnings: [],
    requests: 0,
    listed: false,
  });
});

test("workflow edits and renamed workflows scan every stacked PR", async () => {
  for (const files of [
    [{ filename: ".github/workflows/check.yml" }],
    [{ filename: ".github/actions/custom/action.yml" }],
    [{ filename: "moved-check.yml", previous_filename: ".github/workflows/check.yml" }],
  ]) {
    assert.equal((await runSelector({ files })).audit, "true");
  }
  // The workflow inspects the complete API pagination, not only its first page.
  const files = Array.from({ length: 101 }, (_, index) => ({
    filename: `crates/file-${index}.rs`,
  }));
  files.push({ filename: ".github/workflows/check.yml" });
  assert.equal((await runSelector({ files })).audit, "true");
});

test("uncertain PR file lists retain the audit", async () => {
  for (const options of [
    { changedFiles: 0 },
    { changedFiles: 2 },
    { changedFiles: 3000 },
    { changedFiles: 3001 },
    { changedFiles: Number.NaN },
    { beforeHead: "b".repeat(40) },
    { afterHead: "b".repeat(40) },
    { afterChangedFiles: 2 },
    { apiFailure: "get" },
    { apiFailure: "paginate" },
  ] as SelectionCase[]) {
    assert.equal((await runSelector(options)).audit, "true", JSON.stringify(options));
  }
  assert.deepEqual((await runSelector({ apiFailure: "get" })).warnings, [
    "Unable to prove the audit is unnecessary: GitHub API unavailable",
  ]);
});

test("releases, scheduled audits, dispatches and workflow pushes always scan", async () => {
  for (const eventName of ["release", "schedule", "workflow_dispatch", "push"]) {
    assert.deepEqual(await runSelector({ eventName }), {
      audit: "true",
      warnings: [],
      requests: 0,
      listed: false,
    });
  }
});
