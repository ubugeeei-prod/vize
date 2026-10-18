import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { runInNewContext } from "node:vm";
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
const products = workflow("pr-rust-products.yml");
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
    "pr-rust-products",
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
  for (const name of ["pr-source-plan", "canonical-corpus"])
    assert.ok(needs(source.jobs["source-report"]).includes(name), name);
  assert.equal(source.jobs["pr-rust-source"].uses, "./.github/workflows/pr-rust-checks.yml");
  assert.equal(source.jobs["pr-rust-source"].with?.["run-rust"], true);
  assert.equal(
    source.jobs["pr-rust-source"].with?.["rust-plan"],
    "${{ needs.pr-source-plan.outputs.rust-plan }}",
  );
  assert.equal(source.jobs["pr-rust-products"].uses, "./.github/workflows/pr-rust-products.yml");
  assert.equal(source.jobs["pr-rust-products"].with?.["run-rust"], true);
  assert.equal(
    source.jobs["pr-rust-products"].with?.["rust-plan"],
    "${{ needs.pr-source-plan.outputs.rust-plan }}",
  );
  assert.equal(source.jobs["pr-js-packages"].uses, "./.github/workflows/pr-js-checks.yml");
  assert.equal(
    source.jobs["source-report"].if,
    "${{ always() && !contains(needs.*.result, 'cancelled') }}",
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
  assert.doesNotMatch(validation, /nextest archive/);
  const producer = products.jobs["merge-rust-source"];
  assert.match(producer.if ?? "", /github\.event_name == 'merge_group'/);
  assert.match(commands(producer), /nextest archive @packages@ --cargo-profile ci/);
  const producerSteps = producer.steps ?? [];
  const build = producerSteps.findIndex((step) => /nextest archive/.test(step.run ?? ""));
  const pkl = producerSteps.findIndex((step) =>
    /npm install --global @pkl-community\/pkl@/.test(step.run ?? ""),
  );
  assert.ok(pkl >= 0 && pkl < build, "Pkl is available before archive compilation");
  for (const [report, names] of [
    [rust.jobs["rust-source-report"], ["merge-rust-differential", "pr-rust-build"]],
    [products.jobs["rust-product-report"], ["merge-rust-source", "pr-rust-shard"]],
  ] as const)
    for (const name of names) assert.ok(needs(report).includes(name), `${name} is required`);
  assert.match(commands(rust.jobs["rust-source-report"]), /require-rust-tier\.ts --source/);
  assert.match(commands(products.jobs["rust-product-report"]), /require-rust-tier\.ts --products/);
});

void test("named Rust cohorts preserve filtered execution and receipt-bound queue archives", () => {
  const workers = products.jobs["pr-rust-shard"];
  assert.equal(workers.strategy?.matrix, "${{ fromJSON(inputs.rust-plan).testMatrix }}");
  assert.equal(workers.strategy?.["fail-fast"], false);
  assert.equal(workers.env?.COHORT, "${{ matrix.cohort }}");
  assert.equal(workers.env?.NEXTEST_FILTER, "${{ matrix.filter }}");
  assert.deepEqual(needs(workers), ["merge-rust-source"]);
  const steps = workers.steps ?? [];
  const prRun = steps.find((step) => /cargo nextest run @packages@/.test(step.run ?? ""));
  const archiveRun = steps.find((step) => /cargo nextest run --archive-file/.test(step.run ?? ""));
  assert.ok(prRun && archiveRun);
  assert.match(prRun.if ?? "", /pull_request/);
  assert.match(archiveRun.if ?? "", /merge_group/);
  for (const step of [prRun, archiveRun]) {
    assert.match(step.run ?? "", /-E "\$NEXTEST_FILTER"/);
    assert.match(step.run ?? "", /--profile "\$NEXTEST_PROFILE"/);
    assert.match(step.run ?? "", /else phase_exit=\$\?/);
    assert.match(step.run ?? "", /exit "\$phase_exit"/);
    assert.doesNotMatch(step.run ?? "", /--partition/);
  }
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
  const caller = check.jobs["full-qualification"];
  assert.equal(caller.uses, "./.github/workflows/full-qualification.yml");
  assert.ok(caller.if?.includes("github.event_name == 'schedule'"));
  assert.ok(caller.if?.includes("github.event_name == 'workflow_dispatch'"));
  assert.ok(!caller.if?.includes("pull_request"));
  const full = workflow("full-qualification.yml");
  for (const [physical, command] of [
    [full, /check-skeleton-todos\.rs --check/],
    [check, /write-coverage-summary\.rs/],
  ] as const) {
    const jobs = Object.values(physical.jobs).filter((job) => command.test(commands(job)));
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

// Whole parsed producer vectors from 3d93303bb615c71987394528276bd1bed7e3bc40.
// Only the explicit product-qualification job guard is removed for comparison.
type RustJob = { if: string; needs: string[]; steps: Array<Record<string, unknown>> };
const workflowRoot = fileURLToPath(new URL("../../.github/workflows/", import.meta.url));
const yaml = createRequire(import.meta.url)("yaml") as {
  parse: (source: string) => { jobs: Record<string, RustJob> };
};
const rustWorkflow = (name: string) => yaml.parse(readFileSync(join(workflowRoot, name), "utf8"));
const digest = (value: unknown) => createHash("sha256").update(JSON.stringify(value)).digest("hex");
const productJobs = ["merge-rust-source", "pr-rust-shard"];
const sourceJobs = ["merge-rust-differential", "pr-rust-build"];

test("split Rust workflows preserve whole original producer and corpus recipes", () => {
  const source = rustWorkflow("pr-rust-checks.yml").jobs;
  const products = rustWorkflow("pr-rust-products.yml").jobs;
  assert.deepEqual(Object.keys(source), [...sourceJobs, "rust-source-report"]);
  assert.deepEqual(Object.keys(products), [...productJobs, "rust-product-report"]);
  const oldHashes = {
    "merge-rust-source": "d607059b87aae591d6f83cb6be640da086eb70cda213fcbbef47732d9ff5a754",
    "merge-rust-differential": "13c78bc5c1872dffc8af00f380e4b65177e524cf516b5914d808aa0bdea1194e",
    "pr-rust-build": "b6196b2390e511fcbe60021124f07e84642e0d3d377d489316241575df3ffea5",
    "pr-rust-shard": "fac12385c01201bbb2645d0424c6ceb5a3083aad90e58fdc8c4d605ffdc93397",
  };
  for (const [id, expected] of Object.entries(oldHashes)) {
    const job = structuredClone(source[id] ?? products[id]);
    if (productJobs.includes(id)) {
      assert.equal(job.if.split(" && inputs.product-qualification == ''").length, 2);
      job.if = job.if.replace(" && inputs.product-qualification == ''", "");
    }
    assert.equal(digest(job), expected, id);
  }
  for (const report of [source["rust-source-report"], products["rust-product-report"]]) {
    assert.equal(report.if, "${{ always() && !contains(needs.*.result, 'cancelled') }}");
    assert.equal(
      digest(report.steps.slice(0, 3)),
      "649b94e358e52ae35218f47ec18138006bbc76d571f9f8bbb0aa58af7156acc7",
    );
  }
  assert.deepEqual(source["rust-source-report"].needs, sourceJobs);
  assert.deepEqual(products["rust-product-report"].needs, productJobs);
  assert.equal(source["rust-source-report"].steps.length, 4);
  assert.equal(
    source["rust-source-report"].steps[3].run,
    "node tools/support/compat/github/require-rust-tier.ts --source",
  );
  const productReport = products["rust-product-report"];
  assert.deepEqual(Reflect.get(productReport, "permissions"), {
    contents: "read",
    actions: "read",
    "pull-requests": "read",
  });
  assert.equal(
    productReport.steps[3].run,
    "node tools/support/compat/github/require-rust-tier.ts --products",
  );
  assert.deepEqual(productReport.steps[3].env, {
    RUN_RUST: "${{ inputs.run-rust }}",
    NEEDS_JSON: "${{ toJSON(needs) }}",
    PRODUCT_QUALIFICATION: "${{ inputs.product-qualification }}",
    RELEASE_QUALIFICATION: "${{ inputs.release-qualification }}",
    GH_TOKEN: "${{ github.token }}",
  });
  const artifactSteps = structuredClone(productReport.steps.slice(4));
  for (const step of artifactSteps) {
    assert.equal(typeof step.if, "string");
    const guard = String(step.if);
    assert.equal(guard.split(" && inputs.product-qualification == ''").length, 2);
    step.if = guard.replace(" && inputs.product-qualification == ''", "");
  }
  assert.equal(
    digest(artifactSteps),
    "f66b569dbbaa0f609d5dfa5bec576e5e4eab3dc845c7d74f19b4ff4c0b04a40e",
  );
});

test("actual job selectors keep ordinary workers and source checks while only footprint products can skip", () => {
  const source = rustWorkflow("pr-rust-checks.yml").jobs;
  const products = rustWorkflow("pr-rust-products.yml").jobs;
  const selector =
    (job: RustJob) =>
    (context: Record<string, string | boolean>): unknown =>
      runInNewContext(
        `(${job.if
          .slice(3, -2)
          .replaceAll("github.event_name", "event")
          .replaceAll("inputs.run-rust", "runRust")
          .replaceAll("inputs.product-qualification", "product")
          .replaceAll("needs.merge-rust-source.result", "result")
          .replaceAll("always()", "true")
          .replaceAll("cancelled()", "cancelled")})`,
        context,
      );
  const archive = selector(products["merge-rust-source"]),
    cohort = selector(products["pr-rust-shard"]);
  const clippy = selector(source["pr-rust-build"]),
    differential = selector(source["merge-rust-differential"]);
  for (const event of ["pull_request", "merge_group"])
    for (const runRust of [true, false])
      for (const product of ["", "untrusted-nonempty"])
        for (const result of ["success", "failure", "skipped"])
          for (const cancelled of [true, false]) {
            const args = { event, runRust, product, result, cancelled };
            assert.equal(archive(args), event === "merge_group" && runRust && product === "");
            assert.equal(
              cohort(args),
              !cancelled &&
                runRust &&
                product === "" &&
                (event === "pull_request" || result === "success"),
            );
            assert.equal(clippy(args), runRust);
            assert.equal(differential(args), event === "merge_group" && runRust);
          }
});
