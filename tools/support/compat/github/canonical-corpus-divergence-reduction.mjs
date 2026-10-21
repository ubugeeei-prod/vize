import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { appendFileSync, mkdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { basename, join, normalize, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";

import { createDivergenceIssue } from "./canonical-corpus-divergence-issue.mjs";

function contextFromEnv(env) {
  assert(/^[a-f0-9]{40}$/.test(env.GITHUB_SHA ?? ""), "Expected an exact source commit");
  assert(/^\d+$/.test(env.GITHUB_RUN_ID ?? ""), "Expected a workflow run ID");
  assert(/^\d+$/.test(env.GITHUB_RUN_ATTEMPT ?? ""), "Expected a workflow run attempt");
  assert(env.GITHUB_OUTPUT, "Expected a GitHub output file");
  const repository = env.GITHUB_REPOSITORY ?? "ubugeeei-prod/vize";
  const server = env.GITHUB_SERVER_URL ?? "https://github.com";
  return {
    sourceCommit: env.GITHUB_SHA,
    runId: env.GITHUB_RUN_ID,
    runAttempt: env.GITHUB_RUN_ATTEMPT,
    runUrl: `${server}/${repository}/actions/runs/${env.GITHUB_RUN_ID}`,
  };
}

function safeRepositoryPath(path) {
  assert(!/[\0\r\n]/.test(path), "Reduction source path contains a control character");
  assert(
    !path.startsWith("/") && normalize(path) === path,
    "Reduction source path is not normalized",
  );
  assert(!path.split(sep).includes(".."), "Reduction source path escapes the repository");
  return path;
}

function writeJson(path, value) {
  writeFileSync(path, `${JSON.stringify(value)}\n`);
}

export function createReductionPlan(log, ledger, selectedGitlinks, context) {
  const issue = createDivergenceIssue(log, ledger, selectedGitlinks, context);
  if (issue === null) {
    return {
      schema: "vize.realProjectDivergenceReductionPlan",
      version: 1,
      status: "clean",
      divergenceCount: 0,
      sourceCommit: context.sourceCommit,
      runUrl: context.runUrl,
    };
  }
  const sample = issue.samples[0];
  return {
    schema: "vize.realProjectDivergenceReductionPlan",
    version: 1,
    status: "required",
    divergenceCount: issue.divergenceCount,
    fingerprint: issue.fingerprint,
    fixturePath: safeRepositoryPath(sample.fixturePath),
    sourcePath: safeRepositoryPath(sample.sourcePath),
    sourceCommit: context.sourceCommit,
    runUrl: context.runUrl,
  };
}

export function planFromArtifact(artifactDirectory, outputDirectory, env = process.env) {
  const context = contextFromEnv(env);
  const plan = createReductionPlan(
    readFileSync(join(artifactDirectory, "dom-corpus.log"), "utf8"),
    JSON.parse(readFileSync("tests/_fixtures/fixture-compatibility-ledger.json", "utf8")),
    readFileSync(join(artifactDirectory, "selected-gitlinks.txt"), "utf8"),
    context,
  );
  mkdirSync(outputDirectory, { recursive: true });
  writeJson(join(outputDirectory, "reduction-plan.json"), plan);
  appendFileSync(env.GITHUB_OUTPUT, `required=${plan.status === "required"}\n`);
  if (plan.status === "required") {
    appendFileSync(
      env.GITHUB_OUTPUT,
      `fixture_path=${plan.fixturePath}\nsource_path=${plan.sourcePath}\n`,
    );
  } else {
    writeJson(join(outputDirectory, "reduction.json"), {
      schema: "vize.realProjectDivergenceReductionReceipt",
      version: 1,
      status: "clean",
      divergenceCount: 0,
      sourceCommit: plan.sourceCommit,
      runUrl: plan.runUrl,
    });
  }
  return plan;
}

export function runReduction(planPath, outputDirectory, vizeBin) {
  const plan = JSON.parse(readFileSync(planPath, "utf8"));
  assert.equal(plan?.schema, "vize.realProjectDivergenceReductionPlan", "Foreign reduction plan");
  assert.equal(plan?.version, 1, "Unknown reduction plan version");
  assert.equal(plan?.status, "required", "Reduction plan does not request a reduction");
  const sourcePath = safeRepositoryPath(plan.sourcePath);
  const source = readFileSync(sourcePath);
  const reducedPath = join(outputDirectory, "canonical-dom-divergence.reduced.vue");
  const result = spawnSync(
    vizeBin,
    ["reduce", sourcePath, "--dom-divergence", "--out", reducedPath, "--max-runs", "2000"],
    { encoding: "utf8" },
  );
  let status = "failed";
  let reducedBytes = null;
  let reducedSha256 = null;
  if (result.status === 0) {
    reducedBytes = statSync(reducedPath).size;
    assert(reducedBytes > 0, "Reducer wrote an empty witness");
    assert(reducedBytes < source.length, "Reducer did not shrink the divergence witness");
    reducedSha256 = createHash("sha256").update(readFileSync(reducedPath)).digest("hex");
    status = "reduced";
  } else if (result.status === 1) {
    status = "not-reproduced";
  }
  const receipt = {
    schema: "vize.realProjectDivergenceReductionReceipt",
    version: 1,
    status,
    divergenceCount: plan.divergenceCount,
    fingerprint: plan.fingerprint,
    sourcePath,
    sourceBytes: source.length,
    reducedFile: status === "reduced" ? basename(reducedPath) : null,
    reducedBytes,
    reducedSha256,
    reducerExitCode: result.status,
    reducerSummary: String(result.stderr ?? "")
      .trim()
      .slice(0, 2000),
    sourceCommit: plan.sourceCommit,
    runUrl: plan.runUrl,
  };
  writeJson(join(outputDirectory, "reduction.json"), receipt);
  console.log(JSON.stringify(receipt));
  return receipt;
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const [command, ...args] = process.argv.slice(2);
  if (command === "plan" && args.length === 2) {
    planFromArtifact(args[0], args[1]);
  } else if (command === "run" && args.length === 3) {
    runReduction(args[0], args[1], args[2]);
  } else {
    assert.fail(
      "Usage: canonical-corpus-divergence-reduction.mjs plan ARTIFACT_DIRECTORY OUTPUT_DIRECTORY\n" +
        "   or: canonical-corpus-divergence-reduction.mjs run PLAN OUTPUT_DIRECTORY VIZE_BIN",
    );
  }
}
