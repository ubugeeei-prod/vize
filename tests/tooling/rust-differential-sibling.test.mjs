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
import { parse } from "yaml";

const root = fileURLToPath(new URL("../../", import.meta.url));
const workflow = (path) => parse(readFileSync(join(root, path), "utf8"));
const rust = workflow(".github/workflows/pr-rust-checks.yml");
const differential = workflow(".github/workflows/pr-rust-differential.yml");
const job = differential.jobs.differential;
const recipePath = ".github/actions/test-rust-workspace-differential";
const runnerPath = "tools/support/compat/github/rust-differential-lanes.ts";
const recipe = workflow(`${recipePath}/action.yml`);
const fixturePath = "tests/_fixtures/_git/element-plus";
const hydrationArgs = ["submodule", "update", "--init", "--depth", "1", "--", fixturePath];
const realGit = execFileSync("/bin/sh", ["-c", "command -v git"], { encoding: "utf8" }).trim();
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
  const producer = rust.jobs["merge-rust-source"];
  assert.equal(sibling.needs, undefined);
  assert.equal(sibling.if, "${{ github.event_name == 'merge_group' && inputs.run-rust }}");
  assert.equal(sibling.uses, "./.github/workflows/pr-rust-differential.yml");
  assert.equal(sibling.with["rust-plan"], "${{ inputs.rust-plan }}");
  assert.equal(sibling.with["run-rust"], "${{ inputs.run-rust }}");
  for (const required of [
    "merge-rust-source",
    "merge-rust-differential",
    "pr-rust-build",
    "pr-rust-shard",
  ])
    assert(rust.jobs["rust-source-report"].needs.includes(required), required);
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
  for (const prefix of ["dtolnay/rust-toolchain@", "wild-linker/action@"]) {
    const original = producer.steps.find((step) => step.uses?.startsWith(prefix));
    const actual = job.steps.find((step) => step.uses?.startsWith(prefix));
    assert.equal(actual.uses, original.uses);
    assert.deepEqual(actual.with, original.with);
  }
  assert.equal(disk(producer.steps), "nextest-ci");
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
  const seed = workflow(".github/workflows/rust-pr-target-seed.yml");
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
const quote = (value) => `'${String(value).replaceAll("'", "'\\''")}'`;

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
    `#!/bin/sh
if [ "$1" = submodule ]; then
exec "$NODE" -e 'const fs=require("node:fs"),cp=require("node:child_process"),assert=require("node:assert/strict"),path=require("node:path"),args=process.argv.slice(1);assert.deepEqual(args,${JSON.stringify(hydrationArgs)});const tree=cp.execFileSync(process.env.REAL_GIT,["ls-tree","HEAD",args.at(-1)],{encoding:"utf8"}).trim();assert.equal(tree,"160000 commit "+process.env.FIXTURE_PIN+"\\t"+args.at(-1));fs.appendFileSync(process.env.GIT_LOG,JSON.stringify(args)+"\\n");if(process.env.FAIL==="fixture-hydration")process.exit(42);fs.mkdirSync(args.at(-1),{recursive:true});fs.writeFileSync(path.join(args.at(-1),"fixture-pin"),process.env.FIXTURE_PIN);' -- "$@"
fi
exec "$REAL_GIT" "$@"
`,
    { mode: 0o755 },
  );
  writeFileSync(
    join(bin, "cargo"),
    `#!/bin/sh
exec "$NODE" -e 'const fs=require("node:fs"),assert=require("node:assert/strict"),args=process.argv.slice(1);if(process.env.REQUIRE_HYDRATION==="1")assert.equal(fs.readFileSync(${JSON.stringify(`${fixturePath}/fixture-pin`)},"utf8"),process.env.FIXTURE_PIN);fs.appendFileSync(process.env.LOG,JSON.stringify({args,tsgo:process.env.VIZE_TEST_REQUIRE_TSGO,disable:process.env.VIZE_TEST_DISABLE_TSGO,iterations:process.env.VIZE_NUXT_CONFIG_ITERATIONS,production:process.env.VIZE_VUE_RUNTIME_PRODUCTION,corpus:process.env.VIZE_DAVINCI_DIFFERENTIAL_CORPUS})+"\\n");if(process.env.VIZE_TEST_REQUIRE_TSGO!=="1"||Object.hasOwn(process.env,"VIZE_TEST_DISABLE_TSGO")||process.env.VIZE_NUXT_CONFIG_ITERATIONS!=="100")process.exit(92);if(args.includes(process.env.FAIL))process.exit(42);' -- "$@"
`,
    { mode: 0o755 },
  );
  const script = selectedSteps(lane)
    .map((step) => {
      const env = Object.entries(step.env ?? {})
        .map(
          ([key, value]) =>
            `export ${key}=${quote(String(value).replaceAll("${{ matrix.lane }}", lane))}`,
        )
        .join("\n");
      return `(\n${env}\n${step.run}\n)`;
    })
    .join("\n");
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
    return spawnSync("/bin/bash", ["--noprofile", "--norc", "-eo", "pipefail", "-c", script], {
      cwd,
      encoding: "utf8",
      env,
    });
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
