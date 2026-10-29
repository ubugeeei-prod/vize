import assert from "node:assert/strict";
import { test } from "node:test";
import { runInNewContext } from "node:vm";
import { parse } from "yaml";

import { readRepoFile } from "../support/github-workflows.ts";

type Step = { uses?: string; with?: Record<string, unknown> };
type Job = {
  if?: string;
  needs?: string | string[];
  "runs-on"?: string;
  environment?: string;
  permissions?: Record<string, string>;
  steps?: Step[];
};
type CratePublicationFacts = {
  cancelled?: boolean;
  quota?: string;
  resume?: string;
  tag?: string;
  version?: string;
};

const release = parse(readRepoFile(".github", "workflows", "release.yml")) as {
  jobs: Record<string, Job>;
};

function evaluateCratePublication(condition: string, facts: CratePublicationFacts = {}): boolean {
  return Boolean(
    runInNewContext(
      condition
        .replace(/^\$\{\{\s*|\s*\}\}$/gu, "")
        .replaceAll("cancelled()", "cancelled")
        .replaceAll("needs.publication-quota.result", "quota")
        .replaceAll("needs.version.result", "version")
        .replaceAll("inputs.npm_resume_run", "resume")
        .replaceAll("needs.tag.result", "tag"),
      {
        cancelled: facts.cancelled ?? false,
        quota: facts.quota ?? "success",
        resume: facts.resume ?? "",
        tag: facts.tag ?? "success",
        version: facts.version ?? "success",
      },
    ),
  );
}

test("crate publication keeps its trusted identity on a hosted runner and resumes safely", () => {
  const crates = release.jobs["publish-crates"]!;
  assert.equal(crates["runs-on"], "ubuntu-24.04");
  assert.equal(crates.environment, "crates-io");
  assert.deepEqual(crates.permissions, {
    contents: "read",
    actions: "read",
    "id-token": "write",
  });
  assert.deepEqual(crates.needs, ["version", "tag", "publication-quota"]);
  assert.ok(crates.if);
  assert.equal(evaluateCratePublication(crates.if), true, "normal tagged release");
  assert.equal(
    evaluateCratePublication(crates.if, { tag: "skipped" }),
    false,
    "normal release requires its tag",
  );
  assert.equal(
    evaluateCratePublication(crates.if, { resume: "40700986990", tag: "skipped" }),
    true,
    "validated resume may use the existing tag",
  );
  for (const tag of ["failure", "cancelled"]) {
    assert.equal(
      evaluateCratePublication(crates.if, { resume: "40700986990", tag }),
      false,
      "resume cannot bypass a failed tag job",
    );
  }
  for (const failed of [{ quota: "failure" }, { version: "failure" }]) {
    assert.equal(evaluateCratePublication(crates.if, failed), false, "failed prerequisite");
    assert.equal(
      evaluateCratePublication(crates.if, {
        ...failed,
        resume: "40700986990",
        tag: "skipped",
      }),
      false,
      "resume preserves prerequisite failures",
    );
  }
  for (const resume of ["", "40700986990"]) {
    assert.equal(
      evaluateCratePublication(crates.if, { cancelled: true, resume }),
      false,
      "cancellation prevents normal and resumed publication",
    );
  }

  const checkout = crates.steps?.find((step) => step.uses?.startsWith("actions/checkout@"));
  assert.equal(checkout?.with?.ref, "${{ needs.version.outputs.tag }}");
  assert.ok(
    crates.steps?.some((step) => step.uses?.startsWith("rust-lang/crates-io-auth-action@")),
  );
});
