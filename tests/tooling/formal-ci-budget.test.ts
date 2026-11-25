import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { parse as parseYaml } from "yaml";

// docs/davinci/decisions/2026-11-25-proof-carrying-toolchain.md: formal
// verification never becomes the CI bottleneck. Every generated job that runs
// Kani is capped at 10 minutes and every job that runs Creusot at 30 minutes,
// which covers a cold toolchain build.

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const kaniLimit = 10;
const creusotLimit = 30;

type Step = { uses?: string; run?: string };
type Job = { "timeout-minutes"?: number; steps?: Step[] };
type Workflow = { jobs?: Record<string, Job> };

function workflowFiles(): string[] {
  return execFileSync("git", ["ls-files", "-z", "--", ".github/workflows/*.yml"], {
    cwd: repoRoot,
    encoding: "utf8",
  })
    .split("\0")
    .filter(Boolean);
}

function commandLines(step: Step): string[][] {
  return (step.run ?? "").split("\n").map((line) => line.trim().split(/\s+/u));
}

function runsKani(step: Step): boolean {
  if (step.uses?.startsWith("model-checking/kani-github-action@")) return true;
  return commandLines(step).some(
    (words) => words[0] === "kani" || (words[0] === "cargo" && words[1] === "kani"),
  );
}

function runsCreusot(step: Step): boolean {
  return commandLines(step).some((words) => {
    const cargo = words.indexOf("cargo");
    return cargo >= 0 && words[cargo + 1] === "creusot";
  });
}

/** Every generated job that runs a formal verifier, with the cap it must respect. */
function formalJobs(): { where: string; limit: number; timeout: number | undefined }[] {
  const jobs = [];
  for (const file of workflowFiles()) {
    const workflow = parseYaml(fs.readFileSync(path.join(repoRoot, file), "utf8")) as Workflow;
    for (const [name, job] of Object.entries(workflow.jobs ?? {})) {
      const steps = job.steps ?? [];
      const creusot = steps.some(runsCreusot);
      const kani = steps.some(runsKani);
      if (!creusot && !kani) continue;
      jobs.push({
        where: `${file} ${name}`,
        limit: creusot ? creusotLimit : kaniLimit,
        timeout: job["timeout-minutes"],
      });
    }
  }
  return jobs;
}

test("every Kani and Creusot job stays inside the formal CI budget", () => {
  const jobs = formalJobs();
  assert.ok(jobs.length > 0, "the generated workflows run Kani or Creusot somewhere");
  const overBudget = jobs
    .filter((job) => job.timeout === undefined || job.timeout > job.limit)
    .map((job) => `${job.where}: timeout-minutes ${job.timeout ?? "unset"} > ${job.limit}`);
  assert.deepEqual(overBudget, []);
});
