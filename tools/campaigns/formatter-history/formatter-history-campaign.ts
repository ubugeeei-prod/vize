import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";

import { createCampaignPlan } from "./campaign-plan.ts";
import { captureRustLaws } from "./campaign-rust-laws.ts";
import { captureImportSorting } from "./formatter-import-sorting-capture.ts";
import { validateCampaignAuthority } from "./campaign-authority.ts";
import { captureEngineeringControls } from "./campaign-controls.ts";

const args = process.argv.slice(2);
assert(
  (args[0] === "--plan" && [2, 3].includes(args.length)) ||
    (args[0] === "--run" && args.length === 5),
  "use --plan <repo-root> [output.json] or --run <repo-root> <reviewed-plan.json> <head-sha> <plan-sha256>",
);
const root = fs.realpathSync(args[1]);
const context = await createCampaignPlan(root);
const {
  plan,
  source,
  packs,
  requiredLaws,
  cliBuild,
  rustBuild,
  git,
  sha256,
  validateFormatterHistoryExecution,
  loadFormatterApiManifest,
  validateFormatterApiReport,
  writeBuildReceipt,
  validateBuildReceipt,
  expectedBuildIdentity,
} = context;
if (args[0] === "--plan") {
  const output = path.resolve(args[2] ?? path.join(import.meta.dirname, "campaign-plan.json"));
  assert(!fs.existsSync(output), "never overwrite a previous plan");
  fs.writeFileSync(output, `${JSON.stringify(plan, null, 2)}\n`);
  console.log(
    JSON.stringify({
      output,
      sha256: sha256(fs.readFileSync(output)),
      revision: source.revision,
      apiCases: plan.registeredApi,
      cliCases: plan.cliCases,
      lawReferences: requiredLaws.length,
      controls: plan.engineeringControls.length,
      state: "not-executed",
      nativeHandled: 0,
    }),
  );
  process.exit(0);
}
assert.equal(process.env.GITHUB_ACTIONS, "true", "Cargo campaign executes on Actions only");
assert.equal(process.env.CI, "true");
assert.equal(process.env.GITHUB_SHA, source.revision);
assert.equal(process.env.GITHUB_REPOSITORY, "ubugeeei-prod/vize");
const authority = validateCampaignAuthority(context, path.resolve(args[2]), args[3], args[4], root);
const evidence = path.join(root, "target/differential/formatter-campaign");
assert(!fs.existsSync(evidence), "never overwrite a prior campaign");
fs.mkdirSync(evidence, { recursive: true });
const write = (file: string, data: unknown) =>
  fs.writeFileSync(path.join(evidence, file), `${JSON.stringify(data, null, 2)}\n`);
write("plan.json", plan);
write("authority.json", authority);
const processes: any[] = [];
const failures: string[] = [];
const run = (
  id: string,
  program: string,
  argv: string[],
  env: Record<string, string | undefined> = {},
  timeout = 900_000,
) => {
  const startedAtUtc = new Date().toISOString();
  const r = spawnSync(program, argv, {
    cwd: root,
    env: { ...process.env, ...env, CARGO_INCREMENTAL: "0" },
    timeout,
    maxBuffer: 64 * 1024 * 1024,
  });
  const stdout = r.stdout ?? Buffer.alloc(0),
    stderr = r.stderr ?? Buffer.alloc(0);
  fs.writeFileSync(path.join(evidence, `${id}.stdout`), stdout);
  fs.writeFileSync(path.join(evidence, `${id}.stderr`), stderr);
  const observation = {
    id,
    program,
    argv,
    cwd: root,
    startedAtUtc,
    finishedAtUtc: new Date().toISOString(),
    exitStatus: r.status,
    signal: r.signal,
    processError: r.error?.message ?? null,
    stdoutSha256: sha256(stdout),
    stderrSha256: sha256(stderr),
    stdoutBytes: stdout.length,
    stderrBytes: stderr.length,
  };
  processes.push(observation);
  write("processes.json", processes);
  return { ...observation, stdout, stderr };
};
const successful = (r: ReturnType<typeof run>) =>
  r.exitStatus === 0 && r.signal === null && r.processError === null;
const messages = (stdout: Buffer) =>
  stdout
    .toString()
    .split("\n")
    .filter((line) => line.trim())
    .map((line) => JSON.parse(line));
const packagePath = (id: string) => {
  assert(id.startsWith("path+file:"));
  const url = new URL(id.slice(5));
  url.hash = "";
  return fs.realpathSync(url);
};
for (const [id, program] of [
  ["actual-rustc-version", "rustc"],
  ["actual-cargo-version", "cargo"],
]) {
  const version = run(id!, program!, ["--version", "--verbose"], {}, 30_000);
  if (!successful(version)) failures.push(`Actual hosted toolchain capture failed: ${id}`);
}
const cli = run("cli-build", "cargo", cliBuild);
let cliReady = false;
try {
  assert(successful(cli), "CLI build failed; cached binary cannot count");
  const data = messages(cli.stdout);
  assert.deepEqual(
    data.filter((x) => x.reason === "build-finished").map((x) => x.success),
    [true],
  );
  const selected = data.filter(
    (x) =>
      x.reason === "compiler-artifact" &&
      x.target.name === "vize" &&
      x.target.kind.includes("bin") &&
      x.executable,
  );
  assert.equal(selected.length, 1);
  const artifact = selected[0];
  assert.equal(packagePath(artifact.package_id), fs.realpathSync(path.join(root, "crates/vize")));
  assert.equal(
    fs.realpathSync(artifact.executable),
    fs.realpathSync(path.join(root, "target/ci/vize")),
  );
  assert.equal(artifact.profile.test, false);
  const receiptPath = writeBuildReceipt(root),
    receipt = JSON.parse(fs.readFileSync(receiptPath, "utf8"));
  validateBuildReceipt(receipt, expectedBuildIdentity(root));
  fs.copyFileSync(receiptPath, path.join(evidence, "cli-build-receipt.json"));
  fs.copyFileSync(artifact.executable, path.join(evidence, "vize"));
  const version = run("cli-version", artifact.executable, ["--version"], {}, 30_000);
  assert(successful(version));
  assert.equal(version.stdout.toString(), `${receipt.cliVersion}\n`);
  assert.equal(version.stderr.length, 0);
  write("cli-artifact.json", {
    ...artifact,
    sha256: receipt.binarySha256,
    actualCommand: ["cargo", ...cliBuild],
    receiptRecipeBoundary:
      "locked and message-format flags retain the successful actual artifact; standard shared CLI receipt recipe is unchanged",
  });
  cliReady = true;
} catch (error) {
  failures.push(`CLI build: ${String(error)}`);
}
const repeats: any[] = [];
const repeatedReports: any[] = [];
for (const repeat of [1, 2]) {
  const prefix = `repeat-${repeat}`;
  const apiDir = path.join(evidence, prefix, "formatter-api");
  const api = run(
    `${prefix}-api-tests`,
    "vp",
    [
      "node",
      "--test",
      "--test-concurrency=1",
      "--test-reporter=tap",
      "tests/tooling/differential-formatter-api-execution.test.mjs",
    ],
    { VIZE_FORMATTER_API_EVIDENCE_DIR: apiDir },
  );
  const reports: any[] = [];
  try {
    assert(successful(api), "mandatory API execution test failed");
    const receipt = JSON.parse(fs.readFileSync(path.join(apiDir, "build-receipt.json"), "utf8"));
    assert.equal(receipt.source.sourceRevision, source.revision);
    for (const [name, count, bytes, errors, internal] of packs) {
      const artifact = name === "script" ? "report.json" : `${name}-report.json`;
      const report = JSON.parse(fs.readFileSync(path.join(apiDir, artifact), "utf8"));
      const loaded = loadFormatterApiManifest(
        path.join(root, `tests/_fixtures/differential/formatter-history/${name}-manifest.json`),
        root,
      );
      validateFormatterApiReport(loaded, report, receipt);
      assert.deepEqual(report.summary, {
        plannedCases: count,
        legacyByteMatches: bytes,
        legacyInternalObservations: internal,
        legacyErrorMatches: errors,
        legacyFailures: 0,
        nativeUnsupported: count,
        nativeHandled: 0,
        nativeEquivalent: 0,
        pairedComparisons: 0,
      });
      reports.push(report);
    }
    validateFormatterHistoryExecution(root, reports);
  } catch (error) {
    failures.push(`${prefix} API: ${String(error)}`);
  }
  let cliHistory: any;
  let cliHistoryValidated = false;
  if (cliReady) {
    const oldReport = path.join(root, "target/differential/formatter-api/cli-history-report.json");
    fs.rmSync(oldReport, { force: true });
    const observed = run(`${prefix}-cli-tests`, "vp", [
      "node",
      "--test",
      "--test-concurrency=1",
      "--test-reporter=tap",
      "tests/tooling/differential-formatter-cli.test.mjs",
    ]);
    try {
      assert(successful(observed), "mandatory historical CLI execution test failed");
      cliHistory = JSON.parse(fs.readFileSync(oldReport, "utf8"));
      assert.equal(cliHistory.sourceRevision, source.revision);
      assert.deepEqual(cliHistory.summary, {
        plannedCases: 5,
        legacyMatches: 5,
        legacyFailures: 0,
        nativeUnsupported: 5,
        nativeHandled: 0,
        nativeEquivalent: 0,
        pairedComparisons: 0,
      });
      fs.copyFileSync(oldReport, path.join(evidence, prefix, "cli-history-report.json"));
      cliHistoryValidated = true;
    } catch (error) {
      failures.push(`${prefix} CLI: ${String(error)}`);
    }
  }
  repeatedReports.push({ reports, cliHistory: cliHistoryValidated ? cliHistory : undefined });
  repeats.push({
    repeat,
    apiValidatedRows: reports.reduce((n, r) => n + r.rows.length, 0),
    apiMatchedRows: reports.reduce(
      (n, r) => n + r.rows.filter((x: any) => x.legacy.state === "matched-reference").length,
      0,
    ),
    cliValidatedRows: cliHistoryValidated ? cliHistory.rows.length : 0,
    cliMatchedRows: cliHistoryValidated ? cliHistory.summary.legacyMatches : 0,
  });
}
let repeatedBytesVerified = false;
try {
  const [first, second] = repeatedReports;
  assert.equal(first.reports.length, 8);
  assert.equal(second.reports.length, 8);
  assert.equal(first.cliHistory?.rows.length, 5);
  assert.equal(second.cliHistory?.rows.length, 5);
  for (const [index, original] of first.reports.entries()) {
    const current = second.reports[index];
    assert.deepEqual(
      current.rows,
      original.rows,
      "repeated full API bytes/options/errors/fixed points changed",
    );
    assert.deepEqual(current.buildReceipt.source, original.buildReceipt.source);
    assert.equal(current.buildReceipt.artifact.sha256, original.buildReceipt.artifact.sha256);
  }
  assert.deepEqual(
    second.cliHistory.rows,
    first.cliHistory.rows,
    "repeated full CLI inputs/output/streams/verdicts changed",
  );
  assert.deepEqual(second.cliHistory.buildReceipt, first.cliHistory.buildReceipt);
  repeatedBytesVerified = true;
} catch (error) {
  failures.push(`Repeated observations: ${String(error)}`);
}
let importSortingControls = 0;
if (cliReady) {
  try {
    importSortingControls = (await captureImportSorting(root, evidence)).summary
      .actualObservedControls;
  } catch (error) {
    failures.push(`#7258 CLI controls: ${String(error)}`);
  }
}
const laws = captureRustLaws({
  root,
  rustBuild,
  requiredLaws,
  run,
  successful,
  messages,
  packagePath,
  sha256,
  write,
  failures,
  evidence,
  selectedIntegrationTargets: plan.selectedIntegrationTargets,
});
const controls = captureEngineeringControls({
  root,
  evidence,
  plan,
  audit: context.audit,
  run,
  successful,
  messages,
  packagePath,
  sha256,
  write,
  failures,
  laws,
  repeatedBytesVerified,
});
assert.equal(git("status", "--porcelain"), "", "campaign must not mutate source");
assert.equal(git("rev-parse", "HEAD"), source.revision);
const files = (dir: string): string[] =>
  fs
    .readdirSync(dir, { withFileTypes: true })
    .flatMap((entry) =>
      entry.isDirectory() ? files(path.join(dir, entry.name)) : [path.join(dir, entry.name)],
    );
const inventory = files(evidence).map((file) => ({
  path: path.relative(evidence, file),
  bytes: fs.statSync(file).size,
  sha256: sha256(fs.readFileSync(file)),
}));
write("evidence-files.json", inventory);
write("campaign-receipt.json", {
  schema: "vize.formatter-history.campaign",
  version: 1,
  source,
  actions: {
    repository: process.env.GITHUB_REPOSITORY,
    runId: process.env.GITHUB_RUN_ID,
    attempt: process.env.GITHUB_RUN_ATTEMPT,
    job: process.env.GITHUB_JOB,
    workflow: process.env.GITHUB_WORKFLOW,
    workflowRef: process.env.GITHUB_WORKFLOW_REF,
    workflowSha: process.env.GITHUB_WORKFLOW_SHA,
    ref: process.env.GITHUB_REF,
  },
  processes,
  repeats,
  repeatedBytesVerified,
  actualPassedRustLawReferences: laws.length,
  expectedRustLawReferences: requiredLaws.length,
  importSortingObservedCliControls: importSortingControls,
  state: failures.length ? "failed" : "observed",
  failures,
  engineeringControls: {
    declared: 24,
    observedHostedArms: controls.filter((row: any) => row.state === "observed-hosted-control-arms")
      .length,
    accepted: 0,
    measuredPerformanceAccepted: 0,
    state: "pending independent full receipt review and applicable metrics",
  },
  importSorting7258: "unfinished; partial CLI controls do not register public API feature history",
  nativeHandled: 0,
  nativeEquivalent: 0,
  pairedComparisons: 0,
  wholeHistoryGate: "unfinished; diagnostic execution is not protected merge/queue acceptance",
});
assert.deepEqual(failures, [], "actual campaign failures retain raw evidence");
