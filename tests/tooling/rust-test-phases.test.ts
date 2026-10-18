import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import { copyFileSync, mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { parse } from "yaml";
import { readRepoFile, root } from "./support/github-workflows.ts";

type Step = { name?: string; run?: string; env?: Record<string, string> };
type Job = { steps?: Step[]; uses?: string };
type PhaseRecord = { phase: string; elapsed_seconds: number; exit_code: number };
const source = parse(readRepoFile(".github", "workflows", "pr-source-checks.yml")) as {
  jobs: Record<string, Job>;
};
const caller = source.jobs["pr-rust-products"];
// Follow both required callers into the producer and its parallel doctest job.
assert.equal(caller.uses, "./.github/workflows/pr-rust-products.yml");
const jobs = (parse(readRepoFile(caller.uses!)) as { jobs: Record<string, Job> }).jobs;
const sourceCaller = source.jobs["pr-rust-source"];
assert.equal(sourceCaller.uses, "./.github/workflows/pr-rust-checks.yml");
const sourceJobs = (parse(readRepoFile(sourceCaller.uses!)) as { jobs: Record<string, Job> }).jobs;
const measured = jobs["merge-rust-source"];
const phase = (job: Job, name: string) => {
  const step = job.steps?.find((candidate) => candidate.name === name);
  assert.ok(step?.run, `${name} must be an executable workflow step`);
  return step;
};
const build = phase(measured, "Build Rust workspace tests");
const run = phase(sourceJobs["pr-rust-build"], "Run affected Rust doctests");
const packages = ["--package", "vize_glyph", "--package", "vize_maestro"];
const buildArgs = [
  "nextest",
  "archive",
  ...packages,
  "--cargo-profile",
  "ci",
  "--timings",
  "--archive-file",
  "target/rust-test-archive/tests.tar.zst",
];
const runArgs = ["test", ...packages, "--profile", "ci", "--doc"];

function fixture() {
  const cwd = mkdtempSync(join(tmpdir(), "vize-rust-phases-"));
  const bin = join(cwd, "bin");
  const temporary = join(cwd, "runner-temp");
  mkdirSync(bin);
  mkdirSync(temporary);
  const git = (...args: string[]) => execFileSync("git", args, { cwd, encoding: "utf8" }).trim();
  git("init", "-q");
  git("config", "user.name", "CI Test");
  git("config", "user.email", "ci@example.invalid");
  const helper = "tools/support/compat/github/rust-test-archive.mjs";
  mkdirSync(join(cwd, "tools/support/compat/github"), { recursive: true });
  copyFileSync(join(root, helper), join(cwd, helper));
  copyFileSync(
    join(root, "tools/support/compat/github/run-affected-rust.ts"),
    join(cwd, "tools/support/compat/github/run-affected-rust.ts"),
  );
  writeFileSync(
    join(temporary, "rust-plan.json"),
    JSON.stringify({
      schemaVersion: 1,
      scope: "affected",
      packages: ["vize_glyph", "vize_maestro"],
      cargoArgs: packages,
    }),
  );
  writeFileSync(join(cwd, "source"), "fixture\n");
  writeFileSync(join(cwd, "rust-toolchain.toml"), '[toolchain]\nchannel = "1.99.1"\n');
  git("add", "source", "rust-toolchain.toml");
  git("commit", "-qm", "fixture");
  const sha = git("rev-parse", "HEAD");
  const argv = join(cwd, "cargo-argv");
  const environments = join(cwd, "cargo-env");
  const html = join(cwd, "target/cargo-timings/cargo-timing.html");
  const generatedHtml = "<html><title>fake Cargo timing observation</title></html>\n";
  writeFileSync(
    join(bin, "cargo"),
    `#!/bin/bash
printf '%s\\0' "$@" >> "$FAKE_CARGO_ARGV"
printf '\\n' >> "$FAKE_CARGO_ARGV"
case "$*" in
  -V) printf 'cargo 1.99.1 (phase-test fixture)\\n'; exit 0;;
  'nextest --version') printf 'cargo-nextest 0.9.146\\n'; exit 0;;
  'nextest archive --package vize_glyph --package vize_maestro --cargo-profile ci --timings --archive-file target/rust-test-archive/tests.tar.zst')
    test ! -f target/cargo-timings/cargo-timing.html || exit 90
    printf 'fixture archive' > target/rust-test-archive/tests.tar.zst
    if [[ "$FAKE_CARGO_EXIT" == 0 && "$FAKE_CARGO_HTML" == present ]]; then
      mkdir -p target/cargo-timings
      printf '%s' "$FAKE_CARGO_HTML_BYTES" > target/cargo-timings/cargo-timing.html
    fi;;
  'test --package vize_glyph --package vize_maestro --profile ci --doc') ;;
  *) exit 91;;
esac
printf '%s\\n' "$VIZE_TEST_REQUIRE_TSGO" >> "$FAKE_CARGO_ENV"
test "$VIZE_TEST_REQUIRE_TSGO" = 1 || exit 92
exit "$FAKE_CARGO_EXIT"
`,
    { mode: 0o755 },
  );
  writeFileSync(join(bin, "rustc"), "#!/bin/sh\nprintf 'rustc 1.99.1 (phase-test fixture)\\n'\n", {
    mode: 0o755,
  });
  const env: Record<string, string | undefined> = {
    ...process.env,
    PATH: `${bin}:${process.env.PATH ?? ""}`,
    RUNNER_TEMP: temporary,
    FAKE_CARGO_ARGV: argv,
    FAKE_CARGO_ENV: environments,
    FAKE_CARGO_HTML_BYTES: generatedHtml,
    GITHUB_STEP_SUMMARY: join(cwd, "summary.md"),
    GITHUB_ENV: join(cwd, "github-env"),
    GITHUB_EVENT_NAME: "merge_group",
    GITHUB_RUN_ID: "phase-test-run",
    GITHUB_RUN_ATTEMPT: "2",
    SOURCE_SHA: sha,
    CACHE_NAMESPACE: "phase-test-cache",
    VIZE_NUXT_CONFIG_ITERATIONS: "100",
  };
  delete env.VIZE_TEST_DISABLE_TSGO;
  delete env.VIZE_TEST_REQUIRE_TSGO;
  const execute = (step: Step, exit = 0, timingHtml = "present") =>
    spawnSync("/bin/bash", ["--noprofile", "--norc", "-eo", "pipefail", "-c", step.run!], {
      cwd,
      encoding: "utf8",
      env: {
        ...env,
        ...step.env,
        FAKE_CARGO_EXIT: String(exit),
        FAKE_CARGO_HTML: timingHtml,
      },
    });
  const receipt = (name: string): PhaseRecord =>
    JSON.parse(readFileSync(join(temporary, "rust-test-timings", `${name}.json`), "utf8"));
  const doctests = (exit = 0) => {
    const selected = execute(
      phase(sourceJobs["pr-rust-build"], "Select Clippy and doctest runtime envelope"),
    );
    assert.equal(selected.status, 0, selected.stderr);
    const runtime = Object.fromEntries(
      readFileSync(env.GITHUB_ENV!, "utf8")
        .trim()
        .split("\n")
        .map((line) => line.split("=")),
    );
    return execute({ ...run, env: runtime }, exit);
  };
  const calls = (): string[][] =>
    readFileSync(argv, "utf8")
      .trimEnd()
      .split("\n")
      .map((line) => line.split("\0").slice(0, -1));
  const summaryStep = measured.steps?.find(
    (step) => step.name === "Summarize Rust test timing evidence",
  );
  const summarize = (buildOutcome: string, runOutcome = "skipped") =>
    spawnSync(process.execPath, [join(root, "tools/support/compat/github/rust-test-timings.mjs")], {
      cwd,
      encoding: "utf8",
      env: {
        ...env,
        ...summaryStep?.env,
        BUILD_OUTCOME: buildOutcome,
        RUN_OUTCOME: runOutcome,
        SOURCE_SHA: sha,
        CACHE_NAMESPACE: "phase-test-cache",
      },
    });
  return {
    cwd,
    temporary,
    sha,
    html,
    generatedHtml,
    environments,
    execute,
    doctests,
    receipt,
    calls,
    summarize,
    cleanup: () => rmSync(cwd, { recursive: true, force: true }),
  };
}

function assertReceipt(record: PhaseRecord, expectedPhase: string, exitCode: number) {
  assert.deepEqual(Object.keys(record).sort(), ["elapsed_seconds", "exit_code", "phase"]);
  assert.equal(record.phase, expectedPhase);
  assert.equal(record.exit_code, exitCode);
  assert.ok(Number.isInteger(record.elapsed_seconds) && record.elapsed_seconds >= 0);
}

test("the actual affected archive records timing while doctests execute independently", () => {
  const f = fixture();
  try {
    mkdirSync(join(f.cwd, "target/cargo-timings"), { recursive: true });
    writeFileSync(f.html, "stale timing HTML\n");
    const compiled = f.execute(build);
    assert.equal(compiled.status, 0, compiled.stderr);
    const executed = f.doctests();
    assert.equal(executed.status, 0, executed.stderr);
    assert.deepEqual(f.calls(), [buildArgs, ["nextest", "--version"], runArgs]);
    assert.equal(readFileSync(f.environments, "utf8"), "1\n1\n");
    const buildRecord = f.receipt("build");
    assertReceipt(buildRecord, "build", 0);
    const summary = f.summarize("success");
    assert.equal(summary.status, 0, summary.stderr);
    assert.deepEqual(f.calls(), [buildArgs, ["nextest", "--version"], runArgs, ["-V"]]);
    const evidence = join(f.temporary, "rust-test-timings");
    assert.deepEqual(JSON.parse(readFileSync(join(evidence, "summary.json"), "utf8")), [
      { ...buildRecord, outcome: "success" },
      { phase: "run", outcome: "skipped", measurement: "missing" },
    ]);
    const identity = JSON.parse(readFileSync(join(evidence, "identity.json"), "utf8"));
    assert.equal(identity.event_head_sha, f.sha);
    assert.equal(identity.event, "merge_group");
    assert.equal(identity.run_id, "phase-test-run");
    assert.equal(identity.attempt, "2");
    assert.equal(identity.cache_namespace, "phase-test-cache");
    assert.equal(
      identity.build,
      measured.steps?.find((step) => step.name === "Summarize Rust test timing evidence")?.env
        ?.BUILD_COMMAND,
    );
    assert.equal(identity.run, "parallel Clippy and doctest job");
    assert.equal(readFileSync(join(evidence, "checked-out-sha.txt"), "utf8").trim(), f.sha);
    assert.equal(readFileSync(join(evidence, "cargo-timing.html"), "utf8"), f.generatedHtml);
  } finally {
    f.cleanup();
  }
});

test("a nonzero build records and propagates the actual Cargo failure status", () => {
  const f = fixture();
  try {
    const result = f.execute(build, 41);
    assert.equal(result.status, 41, result.stderr);
    assert.deepEqual(f.calls(), [buildArgs]);
    assertReceipt(f.receipt("build"), "build", 41);
  } finally {
    f.cleanup();
  }
});

test("the parallel doctest step propagates its actual Cargo failure", () => {
  const f = fixture();
  try {
    assert.equal(f.execute(build).status, 0);
    const result = f.doctests(43);
    assert.equal(result.status, 43, result.stderr);
    assert.deepEqual(f.calls(), [buildArgs, ["nextest", "--version"], runArgs]);
    assertReceipt(f.receipt("build"), "build", 0);
  } finally {
    f.cleanup();
  }
});

test("the evidence collector rejects a successful build without fresh timing HTML", () => {
  const f = fixture();
  try {
    mkdirSync(join(f.cwd, "target/cargo-timings"), { recursive: true });
    writeFileSync(f.html, "stale timing HTML\n");
    assert.equal(f.execute(build, 0, "missing").status, 0);
    assertReceipt(f.receipt("build"), "build", 0);
    const result = f.summarize("success", "skipped");
    assert.equal(result.status, 1);
    assert.match(result.stderr, /Missing fresh Cargo timing HTML/);
  } finally {
    f.cleanup();
  }
});

test("required aggregation includes the independently executed doctests", () => {
  const productReport = jobs["rust-product-report"] as Job & { needs: string[] };
  const sourceReport = sourceJobs["rust-source-report"] as Job & { needs: string[] };
  assert.ok(productReport.needs.includes("merge-rust-source"));
  assert.ok(productReport.needs.includes("pr-rust-shard"));
  assert.ok(sourceReport.needs.includes("pr-rust-build"));
  assert.ok(sourceReport.needs.includes("merge-rust-differential"));
  const outer = source.jobs["source-report"] as Job & { needs: string[] };
  for (const required of ["pr-rust-source", "pr-rust-products"])
    assert.ok(outer.needs.includes(required));
  for (const [report, mode] of [
    [sourceReport, "source"],
    [productReport, "products"],
  ] as const) {
    const aggregate = report.steps?.find((step) => step.name === "Require the complete Rust tier");
    assert.equal(aggregate?.env?.NEEDS_JSON, "${{ toJSON(needs) }}");
    assert.equal(aggregate?.run, `node tools/support/compat/github/require-rust-tier.ts --${mode}`);
  }
  const summary = measured.steps?.find(
    (step) => step.name === "Summarize Rust test timing evidence",
  );
  assert.equal(summary?.env?.RUN_OUTCOME, "skipped");
  assert.equal(summary?.env?.BUILD_OUTCOME, "${{ steps.rust-test-build.outcome }}");
});
