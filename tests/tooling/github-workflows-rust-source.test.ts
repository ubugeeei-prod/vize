import { readAutomationPrograms, workflowProgram } from "./support/automation-source.ts";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { runInNewContext } from "node:vm";
import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";
import { restoreRustCaseReporting, type RustJob } from "./support/rust-case-reporting.ts";
import { requiredUiConformance } from "./support/ui-conformance-contract.ts";
import {
  readRepoFile,
  requireDailySourceContracts,
  restoreRustToolchainPin,
  restoreVpVersionPin,
  type DailyRustSourceWorkflow,
} from "./support/github-workflows.ts";

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
const workflow = (name: string): Workflow =>
  parse(readAutomationPrograms(".github", "workflows", name));
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
  requiredUiConformance(js.jobs["pr-js-package-conformance"]);
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
  const glyphDocs = parallel.steps?.find((step) => step.name === "Document Glyph public APIs");
  assert.equal(
    glyphDocs?.if,
    "${{ matrix.lane == 'clippy' && contains(fromJSON(inputs.rust-plan).packages, 'vize_glyph') }}",
  );
  assert.match(
    glyphDocs?.run ?? "",
    /rust-test-cases\.ts capture[\s\S]*cargo rustdoc --locked -p vize_glyph --lib -- -D missing-docs/,
  );
  const patinaDocs = parallel.steps?.find((step) => step.name === "Document Patina public APIs");
  assert.equal(
    patinaDocs?.if,
    "${{ matrix.lane == 'clippy' && contains(fromJSON(inputs.rust-plan).packages, 'vize_patina') }}",
  );
  assert.match(
    patinaDocs?.run ?? "",
    /rust-test-cases\.ts capture[\s\S]*cargo rustdoc --locked -p vize_patina --lib -- -D warnings/,
  );
  assert.doesNotMatch(validation, /nextest archive/);
  assert.equal(Object.hasOwn(products.jobs, "merge-rust-source"), false);
  for (const [report, names] of [
    [rust.jobs["rust-source-report"], ["merge-rust-differential", "pr-rust-build"]],
    [products.jobs["rust-product-report"], ["pr-rust-shard"]],
  ] as const)
    for (const name of names) assert.ok(needs(report).includes(name), `${name} is required`);
  assert.match(commands(rust.jobs["rust-source-report"]), /require-rust-tier\.ts --source/);
  assert.match(commands(products.jobs["rust-product-report"]), /require-rust-tier\.ts --products/);
});

void test("PR and queue cohorts build in place from the warm disk and run filtered", () => {
  const workers = products.jobs["pr-rust-shard"];
  assert.equal(workers.strategy?.matrix, "${{ fromJSON(inputs.rust-plan).testMatrix }}");
  assert.equal(workers.strategy?.["fail-fast"], false);
  assert.equal(workers.env?.COHORT, "${{ matrix.cohort }}");
  assert.equal(workers.env?.NEXTEST_FILTER, "${{ matrix.filter }}");
  assert.deepEqual(needs(workers), []);
  const steps = workers.steps ?? [];
  const runs = steps.filter((step) => /cargo nextest run/.test(step.run ?? ""));
  assert.equal(runs.length, 1, "one in-place build and run for both events");
  const [run] = runs;
  assert.equal(run.if, undefined);
  assert.match(run.run ?? "", /cargo nextest run @packages@ --cargo-profile ci/);
  assert.match(run.run ?? "", /-E "\$NEXTEST_FILTER"/);
  assert.match(run.run ?? "", /--profile "\$NEXTEST_PROFILE"/);
  assert.match(run.run ?? "", /else phase_exit=\$\?/);
  assert.match(run.run ?? "", /exit "\$phase_exit"/);
  assert.doesNotMatch(run.run ?? "", /--archive-file|--partition/);
  const cache = steps.find((step) => step.uses === "./.github/actions/setup-rust-sticky-cache");
  assert.equal(cache?.if, undefined);
  assert.equal(cache?.with?.key, "nextest-ci");
  assert.ok(steps.every((step) => !step.uses?.startsWith("actions/download-artifact@")));
});

void test("full repository ratchets and coverage remain scheduled or explicitly requested", () => {
  const caller = check.jobs["full-qualification"];
  assert.equal(caller.uses, "./.github/workflows/full-qualification.yml");
  assert.ok(caller.if?.includes("github.event_name == 'schedule'"));
  assert.ok(caller.if?.includes("github.event_name == 'workflow_dispatch'"));
  assert.ok(!caller.if?.includes("pull_request"));
  const full = workflow("full-qualification.yml");
  for (const [physical, command] of [
    [full, /check-skeleton-todos\.ts --check/],
    [check, /write-coverage-summary\.ts/],
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

// Whole parsed producer vectors (pr-rust-build since its lane matrix split).
// Current daily source policy is checked before reversing its command/evidence additions.
// Every other producer field and the explicit product guard retain the historical whole digest.
const workflowRoot = fileURLToPath(new URL("../../.github/workflows/", import.meta.url));
const yaml = createRequire(import.meta.url)("yaml") as {
  parse: (source: string) => DailyRustSourceWorkflow & { jobs: Record<string, RustJob> };
};
const rustWorkflow = (name: string) => yaml.parse(readAutomationPrograms(join(workflowRoot, name)));
const digest = (value: unknown) => createHash("sha256").update(JSON.stringify(value)).digest("hex");
const productJobs = ["pr-rust-shard", "rust-feature-tests"];
const sourceJobs = ["merge-rust-differential", "pr-rust-build"];

function requireSplitRustRecipes(
  sourceWorkflow = rustWorkflow("pr-rust-checks.yml"),
  productWorkflow = rustWorkflow("pr-rust-products.yml"),
) {
  const contracts = requireDailySourceContracts(
    rustWorkflow("pr-source-checks.yml"),
    sourceWorkflow,
  );
  const source = sourceWorkflow.jobs;
  const products = productWorkflow.jobs;
  assert.deepEqual(Object.keys(source), [...sourceJobs, "rust-source-report"]);
  assert.deepEqual(Object.keys(products), [...productJobs, "rust-product-report"]);
  const legacySteps = source["pr-rust-build"].steps;
  const legacyName = "Test shared legacy transform capabilities";
  assert.equal(legacySteps.filter((step) => step.name === legacyName).length, 1);
  const legacyIndex = legacySteps.findIndex((step) => step.name === legacyName);
  assert.equal(
    legacySteps[legacyIndex].if,
    "${{ matrix.lane == 'clippy' && contains(fromJSON(inputs.rust-plan).packages, 'vize_atelier_core') }}",
  );
  assert.match(legacySteps[legacyIndex].run ?? "", /rust-test-cases\.ts capture/);
  assert.match(legacySteps[legacyIndex].run ?? "", /cargo test --locked --profile ci/);
  assert.equal(legacySteps[legacyIndex - 1]?.name, "Native L1 markup parity");
  assert.equal(legacySteps[legacyIndex + 1]?.name, "Test L1 feature-enabled surface corpus");
  const oldHashes = {
    "merge-rust-differential": "13c78bc5c1872dffc8af00f380e4b65177e524cf516b5914d808aa0bdea1194e",
    "pr-rust-build": "86f6f06586d5cf26e5d6c38d361816467053d003f4fb1d7b069f26cd54fb5b13",
  };
  for (const [id, expected] of Object.entries(oldHashes)) {
    const job = structuredClone(source[id]);
    if (id === "pr-rust-build") {
      const writePlan = job.steps.find((step) => step.name === "Materialize affected Rust plan");
      assert.equal(writePlan?.run, 'node "$VIZE_CI_FILES/materialize-affected-rust-plan.cjs"');
      const program = workflowProgram(
        "pr-rust-checks",
        writePlan.run,
        "materialize-affected-rust-plan.cjs",
      );
      assert.equal(
        createHash("sha256").update(program).digest("hex"),
        "abbe79bebd07e4edb53465ce29aa21a5a7ef6f7569d3e33e3ab258ad19f2b68a",
      );
      writePlan.run = readFileSync(
        new URL("./fixtures/archived-rust-plan-invocation.txt", import.meta.url),
        "utf8",
      ).trimEnd();
      assert.equal(restoreRustToolchainPin(job.steps, "1.99.1"), 1);
      const wild = job.steps.find(
        (step) => step.uses === "./.github/actions/setup-wild-http-recovery",
      );
      assert.deepEqual(wild?.with, { "wild-version": "0.9.0" });
      wild!.uses = "wild-linker/action@0bbbfa5df4380cab8e63cb8505a1ce65e1d10203";
      // The Rust jobs opt into the parallel frontend on the pinned compiler.
      const cache = job.steps.find(
        (step) => step.uses === "./.github/actions/setup-rust-sticky-cache",
      );
      assert.equal(cache?.with?.["frontend-jobs"], "8");
      delete cache!.with!["frontend-jobs"];
      // Authenticate the sole diagnostic-output successor before restoring its
      // exact historical command; retain every other field in the whole hash.
      const doctests = job.steps.filter((step) => step.name === "Run affected Rust doctests");
      assert.equal(doctests.length, 1);
      const originalDoctest =
        'node tools/support/compat/github/run-affected-rust.ts "$RUNNER_TEMP/rust-plan.json" cargo test @packages@ --profile ci --doc';
      assert.deepEqual(doctests[0], {
        name: "Run affected Rust doctests",
        if: "${{ matrix.lane == 'clippy' }}",
        env: { VIZE_RUST_CASE_LABEL: "Affected Rust doctests" },
        run: originalDoctest + " -- --nocapture",
      });
      doctests[0].run = originalDoctest;
      restoreRustCaseReporting(job, contracts);
      job.steps[contracts].run =
        "node --test --test-concurrency=1 tests/tooling/davinci-assertion-lint.test.ts tests/tooling/davinci-consumer-migration-surfaces.test.mjs tests/tooling/docs-reviewed-translations.test.ts";
      job.steps.splice(contracts + 1, 1);
      // PRs also lint the test targets of touched crates on the Clippy ratchet.
      const ratchet = job.steps.findIndex(
        (step) => step.name === "Clippy test targets of touched ratchet crates",
      );
      assert.notEqual(ratchet, -1);
      job.steps.splice(ratchet, 1);
      // Reverse only the complete, separately authenticated legacy acceptance step.
      job.steps.splice(
        job.steps.findIndex((step) => step.name === legacyName),
        1,
      );
    }
    assert.equal(digest(job), expected, id);
  }
  for (const report of [source["rust-source-report"], products["rust-product-report"]]) {
    assert.equal(report.if, "${{ always() && !contains(needs.*.result, 'cancelled') }}");
    const prefix = structuredClone(report.steps.slice(0, 3)); // + sparse checkout
    assert.equal(prefix[0].with?.["sparse-checkout-cone-mode"], false, "closure test pins paths");
    delete prefix[0].with;
    assert.equal(restoreVpVersionPin(prefix), 1);
    assert.equal(
      digest(prefix),
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
    FEATURE_TESTS: "${{ fromJSON(inputs.rust-plan).featureTests }}",
    NEEDS_JSON: "${{ toJSON(needs) }}",
    GH_TOKEN: "${{ github.token }}",
  });
  assert.equal(productReport.steps.length, 4, "no archive custody follows the tier gate");
}

test("split Rust workflows preserve whole original producer and corpus recipes", () => {
  requireSplitRustRecipes();
});

test("whole split preservation refuses changed current policy and unrelated producer fields", () => {
  for (const mutation of [
    "missing",
    "extra",
    "missing-step",
    "extra-step",
    "guard",
    "setup",
    "timeout",
    "hidden",
    "evidence-extra",
    "legacy-missing",
    "legacy-command",
    "legacy-guard",
    "legacy-extra",
    "legacy-position",
    "legacy-duplicate",
    "doctest-missing",
    "doctest-command",
    "doctest-prefix",
    "doctest-old-output",
    "doctest-flag",
    "doctest-duplicate-flag",
    "doctest-guard",
    "doctest-env",
    "doctest-extra",
    "doctest-position",
    "doctest-duplicate",
    "unrelated-command",
  ]) {
    const changed = rustWorkflow("pr-rust-checks.yml");
    const job = changed.jobs["pr-rust-build"];
    const contracts = job.steps.findIndex(
      (step) => step.name === "Require current-tree source, API and reviewed-doc contracts",
    );
    const step = job.steps[contracts];
    if (mutation === "missing")
      step.run = step.run?.replace(" tests/tooling/docs-reviewed-translations.test.ts", "");
    if (mutation === "extra") step.run += " tests/tooling/unregistered-contract.test.ts";
    if (mutation === "missing-step") job.steps.splice(contracts, 1);
    if (mutation === "extra-step") job.steps.push({ run: "echo unapproved" });
    if (mutation === "guard") step.if = "${{ false }}";
    if (mutation === "setup")
      job.steps.splice(
        job.steps.findIndex((value) => value.uses?.startsWith("actions/setup-node@")),
        1,
      );
    if (mutation === "timeout") Reflect.set(job, "timeout-minutes", 999);
    if (mutation === "hidden") job.steps[contracts + 1].with!["include-hidden-files"] = false;
    if (mutation === "evidence-extra") job.steps[contracts + 1]["continue-on-error"] = true;
    const legacyIndex = job.steps.findIndex(
      (step) => step.name === "Test shared legacy transform capabilities",
    );
    const legacy = job.steps[legacyIndex];
    if (mutation === "legacy-missing") job.steps.splice(legacyIndex, 1);
    if (mutation === "legacy-command")
      legacy.run = legacy.run?.replace("--features legacy", "--features legacy-differential");
    if (mutation === "legacy-guard") legacy.if = "${{ false }}";
    if (mutation === "legacy-extra") legacy["continue-on-error"] = true;
    if (mutation === "legacy-position") job.steps.unshift(...job.steps.splice(legacyIndex, 1));
    if (mutation === "legacy-duplicate") job.steps.push(structuredClone(legacy));
    const doctestIndex = job.steps.findIndex((step) => step.name === "Run affected Rust doctests");
    const doctest = job.steps[doctestIndex];
    if (mutation === "doctest-missing") job.steps.splice(doctestIndex, 1);
    if (mutation === "doctest-command") doctest.run += " --ignored";
    if (mutation === "doctest-prefix") doctest.run = "true\n" + doctest.run;
    if (mutation === "doctest-old-output")
      doctest.run = doctest.run?.replace(" -- --nocapture", "");
    if (mutation === "doctest-flag")
      doctest.run = doctest.run?.replace("--nocapture", "--show-output");
    if (mutation === "doctest-duplicate-flag") doctest.run += " --nocapture";
    if (mutation === "doctest-guard") doctest.if = "${{ false }}";
    if (mutation === "doctest-env") doctest.env!.VIZE_RUST_CASE_LABEL = "Changed doctests";
    if (mutation === "doctest-extra") doctest["continue-on-error"] = true;
    if (mutation === "doctest-position") job.steps.unshift(...job.steps.splice(doctestIndex, 1));
    if (mutation === "doctest-duplicate") job.steps.push(structuredClone(doctest));
    if (mutation === "unrelated-command")
      job.steps.find((step) => step.name === "Clippy affected crates")!.run += " --no-deps";
    assert.throws(() => requireSplitRustRecipes(changed), assert.AssertionError, mutation);
  }
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
          .replaceAll("always()", "true")
          .replaceAll("cancelled()", "cancelled")})`,
        context,
      );
  const cohort = selector(products["pr-rust-shard"]);
  const clippy = selector(source["pr-rust-build"]),
    differential = selector(source["merge-rust-differential"]);
  for (const event of ["pull_request", "merge_group"])
    for (const runRust of [true, false])
      for (const product of ["", "untrusted-nonempty"])
        for (const result of ["success", "failure", "skipped"])
          for (const cancelled of [true, false]) {
            const args = { event, runRust, product, result, cancelled };
            assert.equal(cohort(args), runRust);
            assert.equal(clippy(args), runRust);
            assert.equal(differential(args), event === "merge_group" && runRust);
          }
});
