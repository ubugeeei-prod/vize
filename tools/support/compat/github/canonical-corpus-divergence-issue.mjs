import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const ansiSequence = new RegExp(`${String.fromCharCode(27)}\\[[0-?]*[ -/]*[@-~]`, "g");
const corpusPrefix = "tests/_fixtures/_git/";
const issueLabels = ["priority:p0", "area:ci", "area:testing", "area:davinci"];

function compareCodepoints(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}

function stripAnsi(text) {
  return text.replace(ansiSequence, "");
}

function exactCorpusCount(log) {
  const counters = [
    ...stripAnsi(log).matchAll(/davinci DOM corpus sweep: [^\r\n]* divergences=(\d+)/g),
  ];
  assert.equal(counters.length, 1, "Expected one canonical DOM corpus counter");
  const count = Number(counters[0][1]);
  assert(Number.isSafeInteger(count), "Invalid canonical DOM divergence count");
  return count;
}

export function parseCorpusDivergences(log) {
  const divergenceCount = exactCorpusCount(log);
  const samples = [];
  for (const rawLine of stripAnsi(log).split(/\r?\n/)) {
    const start = rawLine.indexOf(corpusPrefix);
    if (start === -1) continue;
    const line = rawLine.slice(start);
    const match =
      /^(tests\/_fixtures\/_git\/[^/]+\/.+?\.vue): old_len=(\d+) new_len=(\d+) first_diff=(\d+) old_window=(.*) new_window=(.*)$/.exec(
        line,
      );
    if (!match) continue;
    samples.push({
      sourcePath: match[1],
      oldLength: Number(match[2]),
      newLength: Number(match[3]),
      firstDifference: Number(match[4]),
      oldWindow: match[5],
      newWindow: match[6],
    });
  }
  assert.equal(
    samples.length,
    Math.min(divergenceCount, 20),
    "Canonical DOM divergence samples are incomplete",
  );
  return { divergenceCount, samples };
}

function fixturePaths(ledger) {
  assert.equal(ledger?.schema, "vize.fixtureCompatibilityLedger", "Foreign fixture ledger");
  assert.equal(ledger?.version, 1, "Unknown fixture ledger version");
  assert(Array.isArray(ledger.fixtures), "Fixture ledger has no fixture inventory");
  const paths = ledger.fixtures.map((fixture) => fixture?.fixturePath);
  assert(
    paths.every((path) => typeof path === "string"),
    "Fixture ledger path is invalid",
  );
  assert.equal(new Set(paths).size, paths.length, "Fixture ledger paths are not unique");
  return paths;
}

function ownerOf(sourcePath, owners) {
  return owners.find((fixturePath) => sourcePath.startsWith(`${fixturePath}/`));
}

function fingerprint(divergenceCount, samples) {
  return createHash("sha256")
    .update(
      JSON.stringify({
        divergenceCount,
        samples: samples.map(({ fixturePath, sourcePath, ...comparison }) => ({
          fixturePath,
          sourcePath,
          ...comparison,
        })),
      }),
    )
    .digest("hex")
    .slice(0, 12);
}

function code(value) {
  return `\`${String(value).replaceAll("`", "\\`")}\``;
}

function validateReductionReceipt(receipt, context, fingerprint, divergenceCount) {
  if (receipt === undefined) return null;
  assert.equal(
    receipt?.schema,
    "vize.realProjectDivergenceReductionReceipt",
    "Foreign divergence reduction receipt",
  );
  assert.equal(receipt?.version, 1, "Unknown divergence reduction receipt version");
  assert.equal(receipt?.sourceCommit, context.sourceCommit, "Reduction receipt commit differs");
  assert.equal(receipt?.fingerprint, fingerprint, "Reduction receipt fingerprint differs");
  assert.equal(receipt?.divergenceCount, divergenceCount, "Reduction receipt count differs");
  assert(
    ["reduced", "not-reproduced", "failed"].includes(receipt?.status),
    "Invalid divergence reduction status",
  );
  return receipt;
}

function validatePromotionReceipt(receipt, context, fingerprint) {
  if (receipt === undefined) return null;
  assert.equal(
    receipt?.schema,
    "vize.realProjectDivergenceFixturePromotionReceipt",
    "Foreign fixture promotion receipt",
  );
  assert.equal(receipt?.version, 1, "Unknown fixture promotion receipt version");
  assert.equal(receipt?.sourceCommit, context.sourceCommit, "Promotion receipt commit differs");
  assert(
    ["skipped", "existing-fixture", "existing-pr", "created"].includes(receipt?.status),
    "Invalid fixture promotion status",
  );
  if (receipt.status !== "skipped") {
    assert.equal(receipt.fingerprint, fingerprint, "Promotion receipt fingerprint differs");
  }
  return receipt;
}

export function createDivergenceIssue(
  log,
  ledger,
  selectedGitlinks,
  context,
  reductionReceipt,
  promotionReceipt,
) {
  const { divergenceCount, samples: rawSamples } = parseCorpusDivergences(log);
  const owners = fixturePaths(ledger);
  const selected = new Set(
    selectedGitlinks
      .split(/\r?\n/)
      .map((line) => line.trim())
      .filter(Boolean),
  );
  assert.equal(selected.size, owners.length, "Selected canonical fixture inventory is incomplete");
  assert.deepEqual(
    [...selected].sort(compareCodepoints),
    [...owners].sort(compareCodepoints),
    "Selected fixtures differ from ledger",
  );
  if (divergenceCount === 0) return null;
  const samples = rawSamples.map((sample) => {
    const fixturePath = ownerOf(sample.sourcePath, owners);
    assert(fixturePath, `Divergence source is outside the fixture ledger: ${sample.sourcePath}`);
    assert(selected.has(fixturePath), `Divergence source fixture was not selected: ${fixturePath}`);
    return { fixturePath, ...sample };
  });
  const digest = fingerprint(divergenceCount, samples);
  const reduction = validateReductionReceipt(reductionReceipt, context, digest, divergenceCount);
  const promotion = validatePromotionReceipt(promotionReceipt, context, digest);
  const title = `fix(davinci): triage canonical DOM divergence ${digest}`;
  const fixtures = [...new Set(samples.map((sample) => sample.fixturePath))].sort(
    compareCodepoints,
  );
  const comparisons = samples.map(
    (sample) =>
      `- ${code(sample.sourcePath)}: old=${sample.oldLength} bytes, Davinci=${sample.newLength} bytes, first difference=${sample.firstDifference}; old window=${code(sample.oldWindow)}; Davinci window=${code(sample.newWindow)}`,
  );
  const artifact = `canonical-corpus-dom-${context.runId}-${context.runAttempt}-${context.sourceCommit}`;
  const reductionArtifact = `canonical-corpus-dom-reduction-${context.runId}-${context.runAttempt}-${context.sourceCommit}`;
  const reductionLines = reduction
    ? [
        `- Automatic reduction: ${code(reduction.status)}`,
        `- Reduced witness artifact: ${code(reductionArtifact)}`,
        ...(reduction.status === "reduced"
          ? [
              `- Reduced witness: ${code(reduction.reducedFile)} (${reduction.sourceBytes} -> ${reduction.reducedBytes} bytes, sha256 ${code(reduction.reducedSha256)})`,
            ]
          : []),
      ]
    : [];
  const nextAction = promotion?.pullRequestUrl
    ? `Next action: fix the product difference on generated draft ${promotion.pullRequestUrl}; its promoted-fixture parity test is expected to stay red until the fix lands.`
    : reduction?.status === "reduced"
      ? "Next action: inspect the reduced witness artifact and commit only the minimized in-repo fixture before fixing the product behavior."
      : "Next action: reproduce the exact comparison from the workflow artifact, minimize it with `vize reduce`, and commit only the minimized in-repo fixture before fixing the product behavior.";
  const body = [
    `The canonical real-project corpus found ${divergenceCount} legacy-vs-Davinci DOM output divergence(s).`,
    "",
    `<!-- vize-real-project-divergence:${digest} -->`,
    "",
    `- Source commit: ${code(context.sourceCommit)}`,
    `- Workflow run: ${context.runUrl}`,
    `- Evidence artifact: ${code(artifact)}`,
    `- Pinned fixture owner(s): ${fixtures.map(code).join(", ")}`,
    ...reductionLines,
    ...(promotion
      ? [
          `- Fixture promotion: ${code(promotion.status)}`,
          ...(promotion.pullRequestUrl ? [`- Generated draft: ${promotion.pullRequestUrl}`] : []),
        ]
      : []),
    "",
    `Captured comparison sample(s) (${samples.length}/${divergenceCount}):`,
    "",
    ...comparisons,
    "",
    nextAction,
    "",
    "This report contains paths and mismatch windows only; it does not copy a third-party source file.",
  ].join("\n");
  return {
    schema: "vize.realProjectDivergenceIssuePlan",
    version: 1,
    divergenceCount,
    fingerprint: digest,
    title,
    body,
    labels: issueLabels,
    fixtures,
    samples,
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

export function githubClient(repository) {
  return {
    findOpenIssue(title) {
      const result = ghJson([
        "api",
        "--method",
        "GET",
        "search/issues",
        "-f",
        `q=repo:${repository} is:issue is:open in:title "${title}"`,
        "-f",
        "per_page=100",
      ]);
      return result.items.find((issue) => issue.title === title) ?? null;
    },
    createIssue(plan) {
      return ghJson(
        ["api", "--method", "POST", `repos/${repository}/issues`, "--input", "-"],
        JSON.stringify({ title: plan.title, body: plan.body, labels: plan.labels }),
      );
    },
  };
}

export function publishDivergenceIssue(plan, context, github) {
  if (plan === null) {
    return {
      schema: "vize.realProjectDivergenceIssueReceipt",
      version: 1,
      status: "clean",
      divergenceCount: 0,
      sourceCommit: context.sourceCommit,
      runUrl: context.runUrl,
    };
  }
  const existing = github.findOpenIssue(plan.title);
  const issue = existing ?? github.createIssue(plan);
  assert(Number.isSafeInteger(issue.number), "GitHub returned no issue number");
  assert(typeof issue.html_url === "string", "GitHub returned no issue URL");
  return {
    schema: "vize.realProjectDivergenceIssueReceipt",
    version: 1,
    status: existing ? "existing" : "created",
    divergenceCount: plan.divergenceCount,
    fingerprint: plan.fingerprint,
    issueNumber: issue.number,
    issueUrl: issue.html_url,
    sourceCommit: context.sourceCommit,
    runUrl: context.runUrl,
  };
}

function workflowContext(env) {
  assert(
    ["schedule", "workflow_dispatch"].includes(env.GITHUB_EVENT_NAME),
    "Divergence issue publication is restricted to schedule and workflow_dispatch",
  );
  assert(/^[a-f0-9]{40}$/.test(env.GITHUB_SHA ?? ""), "Expected an exact source commit");
  assert(/^\d+$/.test(env.GITHUB_RUN_ID ?? ""), "Expected a workflow run ID");
  assert(/^\d+$/.test(env.GITHUB_RUN_ATTEMPT ?? ""), "Expected a workflow run attempt");
  assert(
    /^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/.test(env.GITHUB_REPOSITORY ?? ""),
    "Invalid repository",
  );
  assert(env.GH_TOKEN, "Issue publication requires GH_TOKEN");
  const server = env.GITHUB_SERVER_URL ?? "https://github.com";
  return {
    repository: env.GITHUB_REPOSITORY,
    sourceCommit: env.GITHUB_SHA,
    runId: env.GITHUB_RUN_ID,
    runAttempt: env.GITHUB_RUN_ATTEMPT,
    runUrl: `${server}/${env.GITHUB_REPOSITORY}/actions/runs/${env.GITHUB_RUN_ID}`,
  };
}

export function publishFromArtifact(
  artifactDirectory,
  reductionDirectory,
  promotionDirectory,
  env = process.env,
  github,
) {
  const context = workflowContext(env);
  const plan = createDivergenceIssue(
    readFileSync(join(artifactDirectory, "dom-corpus.log"), "utf8"),
    JSON.parse(readFileSync("tests/_fixtures/fixture-compatibility-ledger.json", "utf8")),
    readFileSync(join(artifactDirectory, "selected-gitlinks.txt"), "utf8"),
    context,
    reductionDirectory
      ? JSON.parse(readFileSync(join(reductionDirectory, "reduction.json"), "utf8"))
      : undefined,
    promotionDirectory
      ? JSON.parse(readFileSync(join(promotionDirectory, "promotion.json"), "utf8"))
      : undefined,
  );
  const receipt = publishDivergenceIssue(plan, context, github ?? githubClient(context.repository));
  writeFileSync(join(artifactDirectory, "divergence-issue.json"), `${JSON.stringify(receipt)}\n`);
  console.log(JSON.stringify(receipt));
  return receipt;
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const [command, artifactDirectory, ...extra] = process.argv.slice(2);
  assert(
    command === "publish" && artifactDirectory && extra.length <= 2,
    "Usage: canonical-corpus-divergence-issue.mjs publish ARTIFACT_DIRECTORY [REDUCTION_DIRECTORY] [PROMOTION_DIRECTORY]",
  );
  publishFromArtifact(artifactDirectory, extra[0], extra[1]);
}
