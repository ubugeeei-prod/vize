import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";
import { readRepoFile } from "./support/github-workflows.ts";

type Step = {
  id?: string;
  "timeout-minutes"?: number;
  name?: string;
  if?: string;
  run?: string;
  uses?: string;
  with?: Record<string, string | boolean>;
  env?: Record<string, string>;
};
type Job = {
  if?: string;
  "runs-on"?: string;
  needs?: string[] | string;
  steps?: Step[];
  uses?: string;
  with?: Record<string, string | boolean>;
  env?: Record<string, string>;
  strategy?: { matrix?: string; "fail-fast"?: boolean };
};
type Workflow = {
  on?: Record<string, unknown>;
  jobs: Record<string, Job>;
  concurrency?: { "cancel-in-progress"?: boolean };
};
const workflow = (name: string): Workflow => parse(readRepoFile(".github", "workflows", name));
const check = workflow("check.yml");
const source = workflow("pr-source-checks.yml");
const rust = workflow("pr-rust-checks.yml");
const js = workflow("pr-js-checks.yml");
const commands = (job: Job): string => (job.steps ?? []).map((step) => step.run ?? "").join("\n");
const needs = (job: Job): string[] =>
  typeof job.needs === "string" ? [job.needs] : (job.needs ?? []);

void test("required source reports include selected product gates and mandatory Rust acceptance", () => {
  const entry = check.jobs["pr-source-checks"];
  assert.equal(entry.uses, "./.github/workflows/pr-source-checks.yml");
  for (const event of ["pull_request", "merge_group"]) assert.ok(entry.if?.includes(`'${event}'`));
  assert.ok(Object.hasOwn(source.on ?? {}, "workflow_call"));
  assert.ok(needs(check.jobs["test-report"]).includes("pr-source-checks"));
  for (const name of [
    "pr-rust-source",
    "pr-js-packages",
    "pr-native-runtime",
    "pr-playground-test",
  ]) {
    assert.ok(
      needs(source.jobs[name]).includes("pr-source-plan"),
      `${name} consumes the source plan`,
    );
    assert.ok(
      needs(source.jobs["source-report"]).includes(name),
      `${name} is a required product gate`,
    );
  }
  for (const name of ["pr-source-plan", "wit-contracts", "canonical-corpus"])
    assert.ok(needs(source.jobs["source-report"]).includes(name), name);
  assert.equal(source.jobs["pr-rust-source"].uses, "./.github/workflows/pr-rust-checks.yml");
  assert.equal(source.jobs["pr-rust-source"].with?.["run-rust"], true);
  assert.equal(
    source.jobs["pr-rust-source"].with?.["rust-plan"],
    "${{ needs.pr-source-plan.outputs.rust-plan }}",
  );
  assert.equal(source.jobs["pr-js-packages"].uses, "./.github/workflows/pr-js-checks.yml");
  assert.equal(
    source.jobs["source-report"].if,
    "${{ always() && github.event_name == 'merge_group' && !contains(needs.*.result, 'cancelled') }}",
  );
  assert.match(commands(source.jobs["source-report"]), /canonical-corpus-selection\.mjs --check/);
  for (const [name, job] of [
    ...Object.entries(js.jobs),
    ["pr-native-runtime", source.jobs["pr-native-runtime"]],
    ["pr-playground-test", source.jobs["pr-playground-test"]],
  ] as Array<[string, Job]>) {
    assert.ok(job.steps?.length, `${name} keeps validation or an explicit unrelated skip`);
    assert.ok(
      job.steps?.every((step) => step.if),
      `${name} guards expensive work with its selection`,
    );
  }
  const jsCommands = Object.values(js.jobs).map(commands).join("\n");
  assert.match(jsCommands, /vp run --workspace-root "test:js:\$VIZE_JS_PACKAGE_GROUP"/);
  assert.match(jsCommands, /vp run --filter '\.\/npm\/ui' check/);
  assert.match(
    commands(source.jobs["pr-playground-test"]),
    /vp run --filter '\.\/playground' test:browser/,
  );
});

void test("Clippy and doctests start beside affected cohort execution and stay required", () => {
  const parallel = rust.jobs["pr-rust-build"];
  assert.deepEqual(needs(parallel), []);
  assert.equal(parallel.if, "${{ inputs.run-rust }}");
  const validation = commands(parallel);
  assert.match(
    validation,
    /cargo clippy @packages@ --profile ci -- -D warnings -D clippy::wildcard_imports/,
  );
  assert.match(validation, /cargo test @packages@ --profile ci --doc/);
  const steps = parallel.steps ?? [];
  const doctests = steps.findIndex((step) =>
    /cargo test @packages@ --profile ci --doc/.test(step.run ?? ""),
  );
  const examples = steps.filter((step) =>
    /cargo build @packages@ --profile ci --examples/.test(step.run ?? ""),
  );
  assert.equal(examples.length, 1);
  assert.equal(steps.indexOf(examples[0]), doctests + 1);
  assert.equal(examples[0].if, "${{ github.event_name == 'pull_request' }}");
  assert.match(examples[0].run ?? "", /rust-build-evidence\.ts before "\$evidence"/);
  assert.ok(
    examples[0].run?.includes(
      'node tools/support/compat/github/run-affected-rust.ts "$RUNNER_TEMP/rust-plan.json" cargo build @packages@ --profile ci --examples --timings --message-format=json-render-diagnostics > "$evidence/cargo.jsonl"',
    ),
  );
  const evidence = steps[steps.indexOf(examples[0]) + 1];
  assert.equal(evidence.if, "${{ always() && github.event_name == 'pull_request' }}");
  assert.equal(
    evidence.run,
    'node tools/support/compat/github/rust-build-evidence.ts collect "$RUNNER_TEMP/rust-example-build-evidence"',
  );
  const upload = steps[steps.indexOf(examples[0]) + 2];
  assert.equal(upload.if, evidence.if);
  assert.ok(upload.uses?.startsWith("actions/upload-artifact@"));
  assert.equal(
    upload.with?.name,
    "rust-build-evidence-examples-${{ github.run_id }}-${{ github.run_attempt }}-${{ github.sha }}",
  );
  assert.equal(upload.with?.path, "${{ runner.temp }}/rust-example-build-evidence/");
  assert.equal(upload.with?.["if-no-files-found"], "error");
  assert.doesNotMatch(validation, /cargo test[^\n]*--examples/);
  assert.doesNotMatch(validation, /nextest archive/);
  const producer = rust.jobs["merge-rust-source"];
  assert.match(producer.if ?? "", /github\.event_name == 'merge_group'/);
  assert.match(commands(producer), /nextest archive @packages@ --cargo-profile ci/);
  const producerSteps = producer.steps ?? [];
  const build = producerSteps.findIndex((step) => /nextest archive/.test(step.run ?? ""));
  const pkl = producerSteps.findIndex((step) =>
    /npm install --global @pkl-community\/pkl@/.test(step.run ?? ""),
  );
  assert.ok(pkl >= 0 && pkl < build, "Pkl is available before archive compilation");
  for (const name of [
    "merge-rust-source",
    "merge-rust-differential",
    "pr-rust-build",
    "pr-rust-shard",
  ])
    assert.ok(needs(rust.jobs["rust-source-report"]).includes(name), `${name} is required`);
  assert.match(commands(rust.jobs["rust-source-report"]), /require-rust-tier\.ts/);
});

void test("named Rust cohorts preserve filtered execution and receipt-bound queue archives", () => {
  const workers = rust.jobs["pr-rust-shard"];
  assert.equal(workers.strategy?.matrix, "${{ fromJSON(inputs.rust-plan).testMatrix }}");
  assert.equal(workers.strategy?.["fail-fast"], false);
  assert.equal(workers.env?.COHORT, "${{ matrix.cohort }}");
  assert.equal(workers.env?.NEXTEST_FILTER, "${{ matrix.filter }}");
  assert.deepEqual(needs(workers), ["merge-rust-source"]);
  const steps = workers.steps ?? [];
  const prRun = steps.find((step) => /run-rust-test-cohort\.ts/.test(step.run ?? ""));
  const archiveRun = steps.find((step) => /cargo nextest run --archive-file/.test(step.run ?? ""));
  assert.ok(prRun && archiveRun);
  assert.match(prRun.if ?? "", /pull_request/);
  assert.match(archiveRun.if ?? "", /merge_group/);
  for (const step of [prRun, archiveRun]) {
    assert.match(step.run ?? "", /-E "\$NEXTEST_FILTER"/);
    assert.match(step.run ?? "", /--profile "\$NEXTEST_PROFILE"/);
    assert.doesNotMatch(step.run ?? "", /--partition/);
  }
  assert.match(
    prRun.run ?? "",
    /exec node tools\/support\/compat\/github\/run-rust-test-cohort\.ts/,
  );
  assert.match(archiveRun.run ?? "", /else phase_exit=\$\?/);
  assert.match(archiveRun.run ?? "", /exit "\$phase_exit"/);
  const verify = steps.findIndex((step) => /rust-test-archive\.mjs verify/.test(step.run ?? ""));
  assert.ok(
    verify >= 0 && verify < steps.indexOf(archiveRun),
    "verify custody before executing archived binaries",
  );
  assert.match(archiveRun.run ?? "", /--extract-to "\$GITHUB_WORKSPACE"/);
  assert.match(archiveRun.run ?? "", /--workspace-remap "\$GITHUB_WORKSPACE"/);
  assert.match(steps[verify].if ?? "", /merge_group/);
  const artifacts = steps.filter((step) => step.uses?.startsWith("actions/download-artifact@"));
  assert.equal(artifacts.length, 2, "one bounded attempt and one clean retry");
  for (const artifact of artifacts) {
    assert.equal(artifact.with?.["artifact-ids"], "${{ steps.rust-archive.outputs.id }}");
    assert.equal(artifact.with?.["digest-mismatch"], "error");
    assert.equal(artifact["timeout-minutes"], 3);
  }
  assert.ok(
    steps.some(
      (step) =>
        step.id === "rust-archive" && /rust-archive-download\.ts select/.test(step.run ?? ""),
    ),
  );
});

void test("full repository ratchets and coverage remain scheduled or explicitly requested", () => {
  for (const command of [/check-skeleton-todos\.rs --check/, /write-coverage-summary\.rs/]) {
    const jobs = Object.values(check.jobs).filter((job) => command.test(commands(job)));
    assert.ok(jobs.length > 0, `${command} remains available in full checks`);
    for (const job of jobs) {
      assert.ok(job.if?.includes("github.event_name == 'schedule'"));
      assert.ok(job.if?.includes("github.event_name == 'workflow_dispatch'"));
      assert.ok(!job.if?.includes("pull_request"));
    }
  }
});

void test("trusted main and scheduled runs seed the CI profiles while candidate mounts stay read-only", () => {
  const seed = workflow("rust-pr-target-seed.yml");
  assert.deepEqual(Object.keys(seed.on ?? {}).sort(), ["push", "schedule"]);
  assert.deepEqual((seed.on?.push as { branches?: string[] })?.branches, ["main"]);
  assert.equal(seed.concurrency?.["cancel-in-progress"], false);
  const seeded = commands(seed.jobs["rust-pr-target-seed"]);
  for (const command of [
    "cargo clippy @packages@ --profile ci -- -D warnings -D clippy::wildcard_imports",
    "cargo test @packages@ --profile ci --no-run",
  ])
    assert.ok(seeded.includes(command), command);
  const action = parse(
    readRepoFile(".github", "actions", "setup-rust-sticky-cache", "action.yml"),
  ) as {
    runs: { steps: Step[] };
  };
  const mounts = action.runs.steps.filter((step) =>
    step.uses?.startsWith("useblacksmith/stickydisk@"),
  );
  assert.equal(mounts.length, 4, "registry, git, primary and secondary mounts need isolation");
  for (const mount of mounts) {
    assert.equal(mount.with?.commit, "${{ steps.cache-policy.outputs.sticky-commit }}");
    assert.ok(String(mount.with?.key).startsWith("${{ steps.cache-policy.outputs.sticky-"));
    assert.ok(mount.if?.includes("steps.cache-policy.outputs.sticky == 'true'"));
  }
});

void test("the native seed reuses the PR build profile and cache without running package tests", () => {
  const seed = workflow("rust-pr-target-seed.yml");
  const job = seed.jobs["js-native-target-seed"];
  const candidate = js.jobs["pr-js-package-tests"];
  assert.equal(job["runs-on"], candidate["runs-on"]);
  assert.equal(job.env?.CARGO_BUILD_JOBS, candidate.env?.CARGO_BUILD_JOBS);
  assert.equal(commands(job).trim(), "vp run --workspace-root build:native:test");
  assert.ok(job.steps?.some((step) => step.uses === "./.github/actions/setup-js-package-tests"));
  const bootstrap = job.steps?.find((step) => step.uses === "./.github/actions/cache-vp-bootstrap");
  assert.ok(bootstrap);
  assert.notEqual(bootstrap.with?.save, "true", "only the runtime seed saves the bootstrap key");

  const setup = parse(
    readRepoFile(".github", "actions", "setup-js-package-tests", "action.yml"),
  ) as {
    runs: { steps: Step[] };
  };
  for (const action of [
    "dtolnay/rust-toolchain@",
    "wild-linker/action@",
    "./.github/actions/setup-rust-sticky-cache",
  ]) {
    const seeded = setup.runs.steps.find((step) => step.uses?.startsWith(action));
    const checked = candidate.steps?.find((step) => step.uses?.startsWith(action));
    assert.ok(seeded && checked, `${action} is shared with the candidate build`);
    assert.deepEqual(seeded.with, checked.with);
  }
  assert.equal(
    setup.runs.steps.find((step) => step.uses === "./.github/actions/setup-rust-sticky-cache")?.with
      ?.key,
    "test-js-packages",
  );
  const push = seed.on?.push as { paths: string[] } | undefined;
  assert.ok(push);
  for (const path of [
    "npm/native/**",
    "tools/config/vite-plus/tasks/build.ts",
    "tools/config/vite-plus/task-*.ts",
    ".github/actions/setup-js-package-tests/**",
  ])
    assert.ok(push.paths.includes(path), `${path} refreshes the native seed`);
});
