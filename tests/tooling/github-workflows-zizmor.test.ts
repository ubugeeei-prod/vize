import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";

import { readRepoFile } from "./support/github-workflows.ts";

test("zizmor workflow audits GitHub Actions with pinned security scanning", () => {
  const workflow = readRepoFile(".github", "workflows", "zizmor.yml");
  const parsed = parse(workflow) as {
    name?: string;
    on?: {
      push?: { branches?: string[]; paths?: string[] };
      pull_request?: { branches?: string[]; paths?: string[] };
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
  assert.deepEqual(parsed.on?.pull_request?.branches, ["main", "davinci"]);
  assert.deepEqual(parsed.on?.pull_request?.paths, [".github/**"]);
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
    "advanced-security": true,
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
