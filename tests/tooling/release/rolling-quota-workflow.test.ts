import assert from "node:assert/strict";
import { test } from "node:test";
import { compileFunction } from "node:vm";
import { parse } from "yaml";
import { readRepoFile } from "../support/github-workflows.ts";

interface Step {
  name?: string;
  run?: string;
  uses?: string;
  env?: Record<string, string>;
}
interface Job {
  needs?: string | string[];
  if?: string;
  steps: Step[];
  permissions?: Record<string, string>;
  env?: Record<string, string>;
}
const release = parse(readRepoFile(".github", "workflows", "release.yml")) as {
  jobs: Record<string, Job>;
};
const verification = parse(readRepoFile(".github", "workflows", "release-verify.yml")) as {
  jobs: Record<string, Job>;
};
const publishers = [
  "publish-npm",
  "publish-crates",
  "publish-vscode",
  "publish-open-vsx",
  "publish-vscode-platforms",
  "publish-open-vsx-platforms",
  "github-release",
];

function permitted(job: Job, gate: string, resume = ""): boolean {
  if (!job.if?.includes("always()") && gate !== "success") return false;
  const expression = (job.if ?? "true")
    .replace(/^\$\{\{\s*|\s*\}\}$/g, "")
    .replaceAll("always()", "true")
    .replace(/needs\.([a-z-]+)\./g, (_, key: string) => `needs["${key}"].`);
  const needs: Record<string, { result: string; outputs: Record<string, string> }> = {};
  for (const dependency of [job.needs ?? []].flat())
    needs[dependency] = {
      result: dependency === "publication-quota" ? gate : "success",
      outputs: { ref: "refs/release-candidates/source" },
    };
  return Boolean(
    compileFunction("return (" + expression + ");", ["needs", "inputs"])(needs, {
      npm_resume_run: resume,
    }),
  );
}
test("every initial registry writer requires the fresh post-build admission barrier", () => {
  const barrier = release.jobs["publication-quota"];
  assert.ok(barrier);
  assert.deepEqual(barrier.needs, ["version", "tag"]);
  assert.equal(barrier.permissions?.["id-token"], undefined);
  for (const id of publishers) {
    const job = release.jobs[id];
    assert.ok(job, id);
    assert.ok(job.needs?.includes("publication-quota"), id);
    assert.equal(permitted(job, "success"), true, id);
    for (const state of ["failure", "cancelled", "skipped"]) {
      assert.equal(permitted(job, state), false, id + "/" + state);
      assert.equal(permitted(job, state, "42"), false, id + "/resume/" + state);
    }
  }
});
test("failed-job crate recovery refreshes the same producer-bound quota immediately before publishing", () => {
  const steps = release.jobs["publish-crates"]!.steps;
  const guard = steps.findIndex((s) => s.run?.includes("release/main.ts quota"));
  const publish = steps.findIndex((s) => s.run?.includes("publish_crates/main.ts"));
  assert.ok(guard >= 0 && guard < publish);
  const step = steps[guard]!;
  assert.equal(step.env?.RELEASE_SHA, "${{ needs.version.outputs.sha }}");
  assert.equal(step.env?.RELEASE_PRODUCER_RUN, "${{ inputs.npm_resume_run || github.run_id }}");
  assert.ok(
    release.jobs["publication-quota"]!.steps.some((s) => s.run?.includes("release/main.ts quota")),
  );
});
test("live dry-run is read-only and cannot enter npm verification or JSR publication", () => {
  const dry = verification.jobs["quota-dry-run"]!;
  assert.deepEqual(dry.permissions, { contents: "read" });
  assert.match(dry.if ?? "", /quota_dry_run == true/);
  assert.ok(dry.steps.some((s) => s.run?.includes("quota-dry-run")));
  assert.equal(
    dry.steps.some((s) => /(?:publish| stamp | cut )/.test(s.run ?? "")),
    false,
  );
  assert.match(verification.jobs["verify-npm"]!.if ?? "", /quota_dry_run != true/);
  for (const id of ["verify-config", "verify-musea-vrt", "publish-jsr"]) {
    assert.deepEqual([verification.jobs[id]!.needs ?? []].flat(), ["verify-npm"]);
    assert.equal(verification.jobs[id]!.if?.includes("always()") ?? false, false);
  }
});
test("cadence and manual preparation both use the common guard before stamping", () => {
  const main = readRepoFile("tools", "support", "release", "release", "main.ts");
  const cut = main.slice(
    main.indexOf("async function cut("),
    main.indexOf("function finishCadence("),
  );
  assert.ok(cut.indexOf("releaseQuota(version)") >= 0);
  assert.ok(cut.indexOf("releaseQuota(version)") < cut.indexOf("stampVersion("));
  const cadence = main.slice(
    main.indexOf("async function cadence("),
    main.indexOf("export async function release("),
  );
  assert.match(cadence, /releaseQuota\(version\)/);
});

test("runner output paths belong to step context rather than invalid job env context", () => {
  for (const job of [release.jobs["publication-quota"]!, verification.jobs["quota-dry-run"]!]) {
    assert.equal(
      Object.values(job.env ?? {}).some((value) => value.includes("runner.")),
      false,
    );
    const step = job.steps.find((item) => item.run?.includes("release/main.ts quota"));
    assert.ok(step);
    assert.match(step.env?.RELEASE_QUOTA_EVIDENCE ?? "", /^\$\{\{ runner\.temp \}\}\//);
  }
});
