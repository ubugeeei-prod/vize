import { readToolingProgram } from "./support/program-path.ts";
import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import {
  copyFileSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  realpathSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import { runInNewContext } from "node:vm";

import { invocationWords } from "../../tools/support/ci/automation-commands.ts";
import { automationShellPath, readAutomationPrograms } from "./support/automation-source.ts";
import { parse } from "yaml";

const root = fileURLToPath(new URL("../../", import.meta.url));
const workflow = (path) => parse(readAutomationPrograms(join(root, path)));
const rust = workflow(".github/workflows/pr-rust-checks.yml");
const products = workflow(".github/workflows/pr-rust-products.yml");
const differential = workflow(".github/workflows/pr-rust-differential.yml");
const job = differential.jobs.differential;
const recipePath = ".github/actions/test-rust-workspace-differential";
const runnerPath = "tools/support/compat/github/rust-differential-lanes.ts";
const recipe = workflow(`${recipePath}/action.yml`);
const fixturePath = "tests/_fixtures/_git/element-plus";
const hydrationArgs = ["submodule", "update", "--init", "--depth", "1", "--", fixturePath];
const realGit = execFileSync("which", ["git"], { encoding: "utf8" }).trim();
const fixturePin = execFileSync(realGit, ["ls-tree", "HEAD", fixturePath], {
  cwd: root,
  encoding: "utf8",
})
  .trim()
  .split(/\s+/)[2];
const commands = (steps) => steps.map((step) => step.run ?? "").join("\n");
const disk = (steps) =>
  steps.find((step) => step.uses === "./.github/actions/setup-rust-sticky-cache").with.key;

await test("affected differential lanes share the full recipe and remain required beside the producer", () => {
  const sibling = rust.jobs["merge-rust-differential"];
  const cohort = products.jobs["pr-rust-shard"];
  const seed = workflow(".github/workflows/rust-pr-target-seed.yml");
  const producer = seed.jobs["rust-differential-target-seed"];
  assert.equal(sibling.needs, undefined);
  assert.equal(sibling.if, "${{ github.event_name == 'merge_group' && inputs.run-rust }}");
  assert.equal(sibling.uses, "./.github/workflows/pr-rust-differential.yml");
  assert.equal(sibling.with["rust-plan"], "${{ inputs.rust-plan }}");
  assert.equal(sibling.with["run-rust"], "${{ inputs.run-rust }}");
  for (const [report, requiredJobs] of [
    [rust.jobs["rust-source-report"], ["merge-rust-differential", "pr-rust-build"]],
    [products.jobs["rust-product-report"], ["pr-rust-shard"]],
  ])
    for (const required of requiredJobs) assert(report.needs.includes(required), required);
  assert.equal(job.if, sibling.if);
  assert.equal(job.needs, undefined);
  assert.equal(job["continue-on-error"], undefined);
  assert.equal(job.strategy["fail-fast"], false);
  assert.equal(job.strategy.matrix, "${{ fromJSON(inputs.rust-plan).differentialMatrix }}");
  assert.equal(differential.permissions.contents, "read");
  for (const key of ["VIZE_TEST_REQUIRE_TSGO", "VIZE_NUXT_CONFIG_ITERATIONS", "RUST_TEST_THREADS"])
    assert.equal(job.env[key], producer.env[key], key);
  assert.equal(job.env.VIZE_TEST_REQUIRE_TSGO, "1");
  assert.equal(job.env.VIZE_TEST_DISABLE_TSGO, undefined);
  for (const prefix of ["dtolnay/rust-toolchain@", "./.github/actions/setup-wild-http-recovery"]) {
    const original = producer.steps.find((step) => step.uses?.startsWith(prefix));
    const actual = job.steps.find((step) => step.uses?.startsWith(prefix));
    assert.equal(actual.uses, original.uses);
    assert.deepEqual(actual.with, original.with);
  }
  assert.equal(disk(cohort.steps), "nextest-ci");
  assert.equal(disk(job.steps), "clippy-test");
  assert.match(
    commands(job.steps),
    /rust-differential-lanes\.ts "\$DIFFERENTIAL_LANE" "\$RUNNER_TEMP\/rust-plan\.json"/,
  );
  assert.match(commands(recipe.runs.steps), /rust-differential-lanes\.ts --all/);
  assert(job.steps.some((step) => step.uses === "./.github/actions/test-native-navigation"));
  assert(
    recipe.runs.steps.some((step) => step.uses === "./.github/actions/test-native-navigation"),
  );
  assert(!job.steps.some((step) => step.uses?.startsWith("actions/download-artifact@")));
  assert.doesNotMatch(commands(job.steps), /nextest|rust-test-archive|write-coverage-summary/);
  assert.equal(seed.concurrency["cancel-in-progress"], false);
  assert.equal(disk(seed.jobs["rust-pr-target-seed"].steps), "nextest-ci");
  assert.equal(disk(seed.jobs["rust-differential-target-seed"].steps), "clippy-test");
  for (const key of ["VIZE_TEST_REQUIRE_TSGO", "VIZE_NUXT_CONFIG_ITERATIONS", "RUST_TEST_THREADS"])
    assert.equal(seed.jobs["rust-differential-target-seed"].env[key], job.env[key], key);
  const seedTail = seed.jobs["rust-differential-target-seed"].steps.find(
    (step) => step.uses === `./${recipePath}`,
  );
  assert.equal(seedTail.with["workspace-already-tested"], "true");
});

function selectedSteps(lane) {
  return job.steps
    .filter(
      (step) =>
        step.run?.startsWith("git submodule ") ||
        step.name === "Materialize affected Rust plan" ||
        step.run?.includes(runnerPath) ||
        step.uses === "./.github/actions/test-native-navigation",
    )
    .filter((step) => !step.if || runInNewContext(step.if.slice(3, -3), { matrix: { lane } }))
    .flatMap((step) => (step.uses ? workflow(`${step.uses}/action.yml`).runs.steps : [step]));
}

function fixture(lane) {
  const cwd = realpathSync(mkdtempSync(join(tmpdir(), "vize-rust-differential-")));
  const git = (...args) => execFileSync(realGit, args, { cwd, encoding: "utf8" }).trim();
  git("init", "-q");
  git("config", "user.name", "CI Test");
  git("config", "user.email", "ci@example.invalid");
  writeFileSync(join(cwd, "source"), "source\n");
  git("add", "source");
  git("update-index", "--add", "--cacheinfo", `160000,${fixturePin},${fixturePath}`);
  git("commit", "-qm", "fixture");
  const bin = join(cwd, "bin");
  const log = join(cwd, "commands.jsonl");
  const gitLog = join(cwd, "git-commands.jsonl");
  mkdirSync(bin);
  writeFileSync(log, "");
  writeFileSync(gitLog, "");
  mkdirSync(dirname(join(cwd, runnerPath)), { recursive: true });
  copyFileSync(join(root, runnerPath), join(cwd, runnerPath));
  writeFileSync(
    join(bin, "git"),
    readToolingProgram("commands/rust-differential-sibling/git.sh", {
      HYDRATION_ARGS: JSON.stringify(hydrationArgs),
    }),
    { mode: 0o755 },
  );
  writeFileSync(
    join(bin, "cargo"),
    readToolingProgram("commands/rust-differential-sibling/cargo.sh", {
      FIXTURE_PATH_FIXTURE_PIN: JSON.stringify(`${fixturePath}/fixture-pin`),
    }),
    { mode: 0o755 },
  );
  const execute = (failure) => {
    const env = {
      ...process.env,
      ...job.env,
      PATH: `${bin}:${dirname(process.execPath)}:${process.env.PATH ?? ""}`,
      NODE: process.execPath,
      LOG: log,
      REAL_GIT: realGit,
      GIT_LOG: gitLog,
      FIXTURE_PIN: fixturePin,
      GITHUB_WORKSPACE: cwd,
      RUNNER_TEMP: cwd,
      FAIL: failure,
      REQUIRE_HYDRATION: ["program", "sfc-project", "navigation"].includes(lane) ? "1" : "0",
      RUST_PLAN_JSON: JSON.stringify({
        packages: ["vize_maestro", "vize_l1_to_l2", "vize_atelier_ssr", "vize_atelier_sfc"],
      }),
    };
    delete env.VIZE_TEST_DISABLE_TSGO;
    delete env.TEST_CASES_JOB;
    delete env.VIZE_TEST_CASES;
    const scripts = join(root, ".github/scripts/workflows/pr-rust-differential");
    const action = join(root, ".github/actions/test-native-navigation");
    let result;
    for (const step of selectedSteps(lane)) {
      const stepEnv = {
        ...env,
        GITHUB_ACTION_PATH: action,
        VIZE_CI_FILES: scripts,
        ...Object.fromEntries(
          Object.entries(step.env ?? {}).map(([key, value]) => [
            key,
            String(value).replaceAll("${{ matrix.lane }}", lane),
          ]),
        ),
      };
      const words = invocationWords(step.run);
      const [program, ...args] =
        words && !step.run.includes("$VIZE_CI_FILES")
          ? words.map((word) =>
              word.replace(/\$(\w+)/gu, (_, key) => {
                assert.ok(key in stepEnv, key);
                return stepEnv[key];
              }),
            )
          : [
              "/bin/bash",
              "--noprofile",
              "--norc",
              "-eo",
              "pipefail",
              automationShellPath(
                step.run,
                step.name === "Materialize affected Rust plan"
                  ? "pr-rust-differential"
                  : "test-native-navigation",
              ),
            ];
      result = spawnSync(program, args, { cwd, encoding: "utf8", env: stepEnv });
      if (result.status !== 0) break;
    }
    assert.ok(result);
    return result;
  };
  const records = (filename) =>
    readFileSync(filename, "utf8").trim().split("\n").filter(Boolean).map(JSON.parse);
  return {
    cwd,
    execute,
    calls: () => records(log),
    gitCalls: () => records(gitLog),
    cleanup: () => rmSync(cwd, { recursive: true, force: true }),
  };
}

await test("selected sibling lanes keep pinned hydration, oracle environments and command failures", () => {
  for (const [lane, failure] of [
    ["program", "no-such-command"],
    ["program", "fixture-hydration"],
    ["program", "source_folding::tests::"],
    ["lowering", "davinci_dom_corpus"],
    ["sfc", "no-such-command"],
    ["sfc", "compile_template::duplicates::"],
    ["ssr", "no-such-command"],
    ["ssr", "vue_ssr_render"],
    ["navigation", "server::native_navigation::"],
  ]) {
    const f = fixture(lane);
    try {
      const result = f.execute(failure);
      assert.equal(result.status, failure === "no-such-command" ? 0 : 42, result.stderr);
      const calls = f.calls();
      if (failure !== "fixture-hydration")
        assert.ok(calls.length > 0, `${lane} executes its product checks`);
      assert.deepEqual(
        f.gitCalls(),
        ["program", "navigation"].includes(lane) ? [hydrationArgs] : [],
      );
      assert(
        calls.every(
          (call) => call.tsgo === "1" && call.disable === undefined && call.iterations === "100",
        ),
      );
      assert(
        !calls.some((call) => call.args[0] === "nextest" || call.args.includes("--workspace")),
      );
      if (failure === "fixture-hydration")
        assert.equal(calls.length, 0, "no feature gate runs after hydration failure");
      else if (failure !== "no-such-command")
        assert(calls.at(-1).args.includes(failure), "no later command runs after failure");
      if (lane === "sfc") {
        const gate = calls.find((call) => call.args.includes("compile_template::duplicates::"));
        assert(
          gate,
          "feature-enabled duplicate preflight unit laws execute in the existing SFC lane",
        );
        assert.deepEqual(gate.args, [
          "test",
          "--locked",
          "-p",
          "vize_atelier_sfc",
          "--features",
          "legacy-dom-differential",
          "--lib",
          "compile_template::duplicates::",
          "--",
          "--nocapture",
        ]);
        if (failure === "no-such-command")
          assert(calls.at(-1).args.includes("vapor_runtime_contracts"));
      }
      if (lane === "ssr") {
        assert.equal(calls[0].corpus, f.cwd);
        assert(calls.at(-1).args.includes("vue_ssr_render"));
        assert.equal(calls.at(-1).production, "1");
      }
    } finally {
      f.cleanup();
    }
  }
});
