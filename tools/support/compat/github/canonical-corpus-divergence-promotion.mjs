import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const fixtureRoot = "tests/_fixtures/differential/compiler/real-project-dom-reductions";

function code(value) {
  return `\`${String(value).replaceAll("`", "\\`")}\``;
}

function writeJson(path, value) {
  writeFileSync(path, `${JSON.stringify(value)}\n`);
}

function promotionContext(env) {
  assert(
    ["schedule", "workflow_dispatch"].includes(env.GITHUB_EVENT_NAME),
    "Fixture promotion is restricted to schedule and workflow_dispatch",
  );
  assert(/^[a-f0-9]{40}$/.test(env.GITHUB_SHA ?? ""), "Expected an exact source commit");
  assert(
    /^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/.test(env.GITHUB_REPOSITORY ?? ""),
    "Invalid repository",
  );
  assert(env.GH_TOKEN, "Fixture promotion requires GH_TOKEN");
  return {
    repository: env.GITHUB_REPOSITORY,
    sourceCommit: env.GITHUB_SHA,
  };
}

export function createPromotionPlan(reductionDirectory) {
  const receipt = JSON.parse(readFileSync(join(reductionDirectory, "reduction.json"), "utf8"));
  assert.equal(
    receipt?.schema,
    "vize.realProjectDivergenceReductionReceipt",
    "Foreign reduction receipt",
  );
  assert.equal(receipt?.version, 1, "Unknown reduction receipt version");
  if (receipt.status !== "reduced") return null;
  assert(/^[a-f0-9]{12}$/.test(receipt.fingerprint ?? ""), "Invalid divergence fingerprint");
  assert(/^[a-f0-9]{40}$/.test(receipt.sourceCommit ?? ""), "Invalid reduction source commit");
  assert.equal(
    receipt.reducedFile,
    "canonical-dom-divergence.reduced.vue",
    "Unexpected reduced witness name",
  );
  assert(
    Number.isSafeInteger(receipt.sourceBytes) && receipt.sourceBytes > 0,
    "Invalid source size",
  );
  assert(
    Number.isSafeInteger(receipt.reducedBytes) &&
      receipt.reducedBytes > 0 &&
      receipt.reducedBytes < receipt.sourceBytes &&
      receipt.reducedBytes <= 256 * 1024,
    "Invalid reduced witness size",
  );
  const witness = readFileSync(join(reductionDirectory, receipt.reducedFile));
  assert.equal(witness.length, receipt.reducedBytes, "Reduced witness size differs from receipt");
  assert.equal(
    createHash("sha256").update(witness).digest("hex"),
    receipt.reducedSha256,
    "Reduced witness hash differs from receipt",
  );
  const directory = `${fixtureRoot}/${receipt.fingerprint}`;
  const provenance = {
    schema: "vize.realProjectDivergenceFixtureProvenance",
    version: 1,
    fingerprint: receipt.fingerprint,
    sourcePath: receipt.sourcePath,
    sourceCommit: receipt.sourceCommit,
    sourceRunUrl: receipt.runUrl,
    sourceBytes: receipt.sourceBytes,
    reducedBytes: receipt.reducedBytes,
    reducedSha256: receipt.reducedSha256,
  };
  return {
    schema: "vize.realProjectDivergenceFixturePromotionPlan",
    version: 1,
    fingerprint: receipt.fingerprint,
    branch: `automation/davinci-dom-${receipt.fingerprint}`,
    title: `test(davinci): preserve real-project DOM divergence ${receipt.fingerprint}`,
    commitMessage: `test(davinci): preserve DOM divergence ${receipt.fingerprint}`,
    body: [
      "The Real Project Matrix reduced a canonical legacy-vs-Davinci DOM difference into this fixture.",
      "",
      `- Fingerprint: ${code(receipt.fingerprint)}`,
      `- Source commit: ${code(receipt.sourceCommit)}`,
      `- Source workflow: ${receipt.runUrl}`,
      `- Reduction: ${receipt.sourceBytes} -> ${receipt.reducedBytes} bytes`,
      `- Witness sha256: ${code(receipt.reducedSha256)}`,
      "",
      "This draft is expected to keep the promoted-fixture parity test red until the product difference is fixed. It contains only the minimized witness, not the full third-party source file.",
    ].join("\n"),
    files: [
      { path: `${directory}/witness.vue.txt`, content: witness.toString("utf8") },
      { path: `${directory}/provenance.json`, content: `${JSON.stringify(provenance, null, 2)}\n` },
    ],
  };
}

function ghJson(args, input) {
  return JSON.parse(
    execFileSync("gh", args, {
      encoding: "utf8",
      input,
      env: { ...process.env, GH_TOKEN: process.env.GH_TOKEN },
      maxBuffer: 4 * 1024 * 1024,
    }),
  );
}

function ghMaybeJson(args) {
  try {
    return ghJson(args);
  } catch (error) {
    if (/HTTP 404/.test(String(error?.stderr ?? error?.message ?? ""))) return null;
    throw error;
  }
}

export function githubClient(repository) {
  const [owner] = repository.split("/");
  const api = (path, payload) =>
    ghJson(
      ["api", "--method", "POST", `repos/${repository}/${path}`, "--input", "-"],
      JSON.stringify(payload),
    );
  return {
    fixtureAtBase(path) {
      const result = ghMaybeJson([
        "api",
        "--method",
        "GET",
        `repos/${repository}/contents/${path}`,
        "-f",
        "ref=main",
      ]);
      return result?.content ? Buffer.from(result.content, "base64").toString("utf8") : null;
    },
    findOpenPull(branch) {
      const pulls = ghJson([
        "api",
        "--method",
        "GET",
        `repos/${repository}/pulls`,
        "-f",
        "state=open",
        "-f",
        `head=${owner}:${branch}`,
      ]);
      return pulls[0] ?? null;
    },
    createPull(plan) {
      const base = ghJson(["api", "--method", "GET", `repos/${repository}/git/ref/heads/main`])
        .object.sha;
      const tree = api("git/trees", {
        base_tree: base,
        tree: plan.files.map((file) => ({
          path: file.path,
          mode: "100644",
          type: "blob",
          content: file.content,
        })),
      });
      const commit = api("git/commits", {
        message: plan.commitMessage,
        tree: tree.sha,
        parents: [base],
      });
      api("git/refs", { ref: `refs/heads/${plan.branch}`, sha: commit.sha });
      return api("pulls", {
        title: plan.title,
        head: plan.branch,
        base: "main",
        body: plan.body,
        draft: true,
      });
    },
  };
}

export function publishPromotion(plan, context, github) {
  if (plan === null) {
    return {
      schema: "vize.realProjectDivergenceFixturePromotionReceipt",
      version: 1,
      status: "skipped",
      sourceCommit: context.sourceCommit,
    };
  }
  const existingFiles = plan.files.map((file) => {
    const existing = github.fixtureAtBase(file.path);
    if (existing !== null) {
      assert.equal(existing, file.content, `Existing promoted fixture differs: ${file.path}`);
    }
    return existing;
  });
  if (existingFiles.every((existing) => existing !== null)) {
    return {
      schema: "vize.realProjectDivergenceFixturePromotionReceipt",
      version: 1,
      status: "existing-fixture",
      fingerprint: plan.fingerprint,
      sourceCommit: context.sourceCommit,
    };
  }
  const existingPull = github.findOpenPull(plan.branch);
  const pull = existingPull ?? github.createPull(plan);
  assert(Number.isSafeInteger(pull.number), "GitHub returned no pull request number");
  assert(typeof pull.html_url === "string", "GitHub returned no pull request URL");
  return {
    schema: "vize.realProjectDivergenceFixturePromotionReceipt",
    version: 1,
    status: existingPull ? "existing-pr" : "created",
    fingerprint: plan.fingerprint,
    pullRequestNumber: pull.number,
    pullRequestUrl: pull.html_url,
    sourceCommit: context.sourceCommit,
  };
}

export function publishFromArtifact(
  reductionDirectory,
  outputDirectory,
  env = process.env,
  github,
) {
  const context = promotionContext(env);
  const plan = createPromotionPlan(reductionDirectory);
  const receipt = publishPromotion(plan, context, github ?? githubClient(context.repository));
  mkdirSync(outputDirectory, { recursive: true });
  writeJson(join(outputDirectory, "promotion.json"), receipt);
  console.log(JSON.stringify(receipt));
  return receipt;
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const [command, reductionDirectory, outputDirectory, ...extra] = process.argv.slice(2);
  assert(
    command === "publish" && reductionDirectory && outputDirectory && extra.length === 0,
    "Usage: canonical-corpus-divergence-promotion.mjs publish REDUCTION_DIRECTORY OUTPUT_DIRECTORY",
  );
  publishFromArtifact(reductionDirectory, outputDirectory);
}
