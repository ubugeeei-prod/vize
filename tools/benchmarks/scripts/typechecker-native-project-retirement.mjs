/** Source-bound physical launch/full-answer qualification for #3952. */
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { existsSync, readFileSync, realpathSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

export const TEST =
  "lsp_client::workspace_project::process_tests::native_project_replacement_reaps_previous_api_before_launch_and_keeps_whole_answer";

export function authenticateCopiedObserver({
  recipePath,
  sourceSha,
  sourceRoot = process.cwd(),
  workflowSha = process.env.GITHUB_WORKFLOW_SHA,
}) {
  assert.match(sourceSha, /^[0-9a-f]{40}$/u);
  assert.match(workflowSha, /^[0-9a-f]{40}$/u);
  const recipe = JSON.parse(readFileSync(recipePath, "utf8"));
  assert.equal(recipe.source, sourceSha);
  assert.equal(recipe.workflow, workflowSha);
  const git = (...args) => {
    const result = spawnSync("git", args, {
      cwd: sourceRoot,
      encoding: "utf8",
      maxBuffer: 1024 * 1024,
    });
    assert.equal(result.error, undefined, result.error?.message);
    assert.equal(result.status, 0, result.stderr);
    return result.stdout;
  };
  assert.equal(git("rev-parse", "HEAD").trim(), sourceSha);
  assert.equal(git("rev-parse", `${sourceSha}^{tree}`).trim(), recipe.sourceTree);
  const bytes = spawnSync(
    "git",
    ["show", `${workflowSha}:tools/benchmarks/scripts/typechecker-native-project-retirement.mjs`],
    { cwd: sourceRoot, maxBuffer: 1024 * 1024 },
  );
  assert.equal(bytes.error, undefined, bytes.error?.message);
  assert.equal(bytes.status, 0, bytes.stderr?.toString());
  assert.deepEqual(
    readFileSync(fileURLToPath(import.meta.url)),
    bytes.stdout,
    "copied observer bytes must equal the immutable workflow revision",
  );
}

export function qualifySourceBoundNativeProjectRetirement(options) {
  authenticateCopiedObserver(options);
  return qualifyNativeProjectRetirement(options);
}

export function qualifyNativeProjectRetirement({
  capture,
  recipePath,
  sourceSha,
  nativeBinary,
  sourceRoot = process.cwd(),
  run = spawnSync,
}) {
  const mode = JSON.parse(readFileSync(recipePath, "utf8")).mode;
  assert(["current-inline", "immutable-source-step"].includes(mode));
  const logPath = join(capture, "native-project-retirement-3952.log");
  let receipt;
  if (mode === "current-inline") {
    const result = run(
      "cargo",
      [
        "test",
        "--locked",
        "--profile",
        "ci-opt",
        "-p",
        "vize_canon",
        "--lib",
        TEST,
        "--config",
        'profile.ci-opt.inherits="release"',
        "--config",
        'profile.ci-opt.lto="thin"',
        "--config",
        "profile.ci-opt.codegen-units=16",
        "--",
        "--exact",
        "--nocapture",
      ],
      { cwd: sourceRoot, encoding: "utf8", maxBuffer: 16 * 1024 * 1024 },
    );
    const log = (result.stdout ?? "") + (result.stderr ?? "");
    writeFileSync(logPath, log);
    process.stdout.write(result.stdout ?? "");
    process.stderr.write(result.stderr ?? "");
    assert.equal(result.error, undefined, result.error?.message);
    assert.equal(result.signal, null, "the required native law must finish normally");
    assert.equal(result.status, 0, "the required native law must pass");
    const lines = log.split("\n");
    assert.equal(lines.filter((line) => line === "running 1 test").length, 1);
    assert.equal(lines.filter((line) => line === `test ${TEST} ... ok`).length, 1);
    const frames = lines.filter((line) => line.startsWith("NATIVE_PROJECT_RETIREMENT_3952 "));
    assert.equal(frames.length, 1, "one complete physical native process receipt is required");
    receipt = JSON.parse(frames[0].slice("NATIVE_PROJECT_RETIREMENT_3952 ".length));
    assert.equal(receipt.schema, "vize.native-project-retirement-3952");
    assert.equal(realpathSync(receipt.native), nativeBinary);
    assert.equal(receipt.physicalApiOwners.length, 5);
    const fixture = JSON.parse(
      readFileSync(
        join(
          sourceRoot,
          "tests/_fixtures/differential/lsp/native-project-retirement-3952/input.json",
        ),
        "utf8",
      ),
    );
    assert.deepEqual(receipt.expectedDiagnosticReport, fixture.expected);
    assert.equal(receipt.diagnosticReports.length, 4);
    receipt.diagnosticReports.forEach((report) => assert.deepEqual(report, fixture.expected));
    const api = receipt.launches.filter((launch) => launch.role === "--api");
    assert.equal(api.length, 5);
    api.forEach((launch, index) => {
      assert.equal(launch.pid, receipt.physicalApiOwners[index].pid);
      assert.equal(launch.birth, String(receipt.physicalApiOwners[index].birth));
      assert.equal(realpathSync(receipt.physicalApiOwners[index].executable), nativeBinary);
      assert(launch.previousNativeOwners.every((owner) => owner.role !== "--api"));
      if (index > 0)
        assert(
          launch.previousNativeOwners.some(
            (owner) =>
              owner.role === "--lsp" &&
              owner.pid === receipt.editor.pid &&
              owner.birth === String(receipt.editor.birth),
          ),
        );
    });
  } else {
    assert(!existsSync(logPath), "historical source receives no new native law credit");
    receipt = { status: "not-qualified", reason: "historical-immutable-source", sourceSha };
  }
  // Preserve both original native-binary gates from the PR-only outside-import step.
  for (const name of ["editor-original-script", "editor-monorepo-alias"]) {
    const runtime = JSON.parse(readFileSync(join(capture, name, "runtime.json"), "utf8"));
    assert.equal(realpathSync(runtime.nativeBinary), nativeBinary);
  }
  return receipt;
}
