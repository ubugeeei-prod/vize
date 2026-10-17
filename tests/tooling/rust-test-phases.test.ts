import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import { copyFileSync, mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { parse } from "yaml";
import { readRepoFile, root } from "./support/github-workflows.ts";

type Step = { name?: string; run?: string; env?: Record<string, string> };
type Job = { steps?: Step[] };
type PhaseRecord = { phase: string; elapsed_seconds: number; exit_code: number };
const rust = parse(readRepoFile(".github", "workflows", "pr-rust-checks.yml")) as {
  jobs: Record<string, Job>;
};
const measured = rust.jobs["merge-rust-source"];
const phase = (name: string) => {
  const step = measured.steps?.find((candidate) => candidate.name === name);
  assert.ok(step?.run, `${name} must be an executable workflow step`);
  return step;
};
const build = phase("Build Rust workspace tests");
const buildArgs = [
  "nextest",
  "archive",
  "--package",
  "vize_relief",
  "--cargo-profile",
  "ci",
  "--timings",
  "--archive-file",
  "target/rust-test-archive/tests.tar.zst",
];

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
  mkdirSync(join(cwd, "tools/support/compat/github"), { recursive: true });
  for (const helper of ["rust-test-archive.mjs", "run-affected-rust.ts"]) {
    const relative = join("tools/support/compat/github", helper);
    copyFileSync(join(root, relative), join(cwd, relative));
  }
  writeFileSync(
    join(temporary, "rust-plan.json"),
    JSON.stringify({
      schemaVersion: 1,
      scope: "workspace",
      packages: ["vize_relief"],
      cargoArgs: ["--package", "vize_relief"],
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
  'nextest archive --package vize_relief --cargo-profile ci --timings --archive-file target/rust-test-archive/tests.tar.zst')
    test ! -f target/cargo-timings/cargo-timing.html || exit 90
    printf 'fixture archive' > target/rust-test-archive/tests.tar.zst
    if [[ "$FAKE_CARGO_EXIT" == 0 && "$FAKE_CARGO_HTML" == present ]]; then
      mkdir -p target/cargo-timings
      printf '%s' "$FAKE_CARGO_HTML_BYTES" > target/cargo-timings/cargo-timing.html
    fi;;
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
  const calls = (): string[][] =>
    readFileSync(argv, "utf8")
      .trimEnd()
      .split("\n")
      .map((line) => line.split("\0").slice(0, -1));
  const summarize = (buildOutcome: string) =>
    spawnSync(process.execPath, [join(root, "tools/support/compat/github/rust-test-timings.mjs")], {
      cwd,
      encoding: "utf8",
      env: {
        ...env,
        ...measured.steps?.find((step) => step.name === "Summarize Rust test timing evidence")?.env,
        BUILD_OUTCOME: buildOutcome,
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

test("the archive producer runs the selected Cargo packages and emits only its own timing evidence", () => {
  const f = fixture();
  try {
    mkdirSync(join(f.cwd, "target/cargo-timings"), { recursive: true });
    writeFileSync(f.html, "stale timing HTML\n");
    const compiled = f.execute(build);
    assert.equal(compiled.status, 0, compiled.stderr);
    assert.deepEqual(f.calls(), [buildArgs, ["nextest", "--version"]]);
    assert.equal(readFileSync(f.environments, "utf8"), "1\n");
    const buildRecord = f.receipt("build");
    assertReceipt(buildRecord, "build", 0);
    const summary = f.summarize("success");
    assert.equal(summary.status, 0, summary.stderr);
    assert.deepEqual(f.calls(), [buildArgs, ["nextest", "--version"], ["-V"]]);
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
    assert.match(identity.build, /run-affected-rust\.ts.*cargo nextest archive @packages@/);
    assert.equal(identity.run, "parallel Clippy and doctest job");
    assert.equal(identity.profile, "ci");
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

test("the evidence collector rejects a successful build without fresh timing HTML", () => {
  const f = fixture();
  try {
    mkdirSync(join(f.cwd, "target/cargo-timings"), { recursive: true });
    writeFileSync(f.html, "stale timing HTML\n");
    assert.equal(f.execute(build, 0, "missing").status, 0);
    assertReceipt(f.receipt("build"), "build", 0);
    const result = f.summarize("success");
    assert.equal(result.status, 1);
    assert.match(result.stderr, /Missing fresh Cargo timing HTML/);
  } finally {
    f.cleanup();
  }
});
