import assert from "node:assert/strict";
import { spawnSync, type SpawnSyncReturns } from "node:child_process";
import {
  copyFileSync,
  existsSync,
  mkdtempSync,
  mkdirSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import { parse as parseToml } from "@iarna/toml";
import { parse } from "yaml";
import { readRepoFile } from "./support/github-workflows.ts";

const corsaDeferred = [
  "ide::rename::corsa_session_tests::concurrent_real_corsa_rename_sessions_are_isolated",
  "ide::rename::corsa_session_tests::direct_first::dependency_change_rearms_the_shared_editor_transport_once",
  "ide::rename::corsa_session_tests::direct_first::direct_first_renames_survive_twenty_session_shutdown_overlap",
];
const canonDeferred = [
  { name: "inline_event_assignments_preserve_exact_diagnostics", pack: "event-handler-narrowing" },
  {
    name: "deferred_template_reads_preserve_script_diagnostics",
    pack: "template-definite-assignment",
  },
  { name: "required_props_keep_exact_unicode_diagnostics", pack: "required-props-edges" },
  { name: "unicode_reserved_props_preserve_value_diagnostics", pack: "unicode-reserved-props" },
  {
    name: "reserved_prop_shapes_preserve_literal_and_member_diagnostics",
    pack: "reserved-expression-shapes",
  },
  {
    name: "component_event_tuples_preserve_all_argument_diagnostics",
    pack: "component-event-tuples",
  },
  { name: "typed_import_meta_preserves_exact_authored_diagnostics", pack: "typed-import-meta" },
  { name: "slot_outlet_keys_preserve_complete_project_diagnostics", pack: "slot-outlet-key" },
  {
    name: "options_api_any_instance_preserves_complete_original_diagnostics",
    pack: "options-api-any-instance",
  },
  {
    name: "authored_unused_symbols_preserve_complete_original_diagnostics",
    pack: "authored-unused-symbols",
  },
  {
    name: "split_script_setup_preserves_complete_original_diagnostics",
    pack: "split-script-original",
  },
  {
    name: "v_for_source_callbacks_preserve_complete_original_diagnostics",
    pack: "v-for-source-original",
  },
  {
    name: "declared_prop_names_preserve_complete_original_diagnostics",
    pack: "declared-prop-names-original",
  },
  {
    name: "component_v_for_source_preserves_complete_original_diagnostics",
    pack: "v-for-component-source-original",
  },
  {
    name: "template_suppression_preserves_complete_original_diagnostics",
    pack: "template-suppression-original",
  },
  {
    name: "native_click_events_preserve_pointer_event_diagnostics",
    pack: "native-pointer-events-original",
  },
];
type Step = { name?: string; run?: string; if?: string; with?: Record<string, string> };
const workflow = parse(readRepoFile(".github", "workflows", "pr-rust-products.yml")) as {
  jobs: Record<string, { steps: Step[]; env: Record<string, string> }>;
};
const sourceWorkflow = parse(
  readRepoFile(".github", "workflows", "pr-rust-checks.yml"),
) as typeof workflow;

test("only the present audited real-TSGO cases leave the PR default filter", () => {
  const config = parseToml(readRepoFile(".config", "nextest.toml")) as {
    profile: { pr: { "default-filter": string }; default?: { "default-filter"?: string } };
  };
  assert.equal(
    config.profile.pr["default-filter"].replace(/\s/g, ""),
    `not((package(=vize_maestro)&(${corsaDeferred.map((name) => `test(=${name})`).join("|")}))|(package(=vize_canon)&binary(=fix_history_diagnostics)&(${canonDeferred.map(({ name }) => `test(=${name})`).join("|")})))`,
  );
  assert.equal(config.profile.default?.["default-filter"], undefined);
  const source = readRepoFile("crates/vize_maestro/src/ide/rename/corsa_session_tests.rs");
  const direct = readRepoFile(
    "crates/vize_maestro/src/ide/rename/corsa_session_tests/direct_first.rs",
  );
  for (const name of corsaDeferred) {
    const functionName = name.split("::").at(-1);
    const body = name.includes("::direct_first::") ? direct : source;
    assert.ok(body.includes(`fn ${functionName}()`), `${name} must exist`);
  }
  assert.ok(source.includes("real-Corsa isolation coverage cannot be skipped"));
  const canon = readRepoFile("crates/vize_canon/tests/fix_history_diagnostics.rs");
  const runner = readRepoFile("crates/vize_canon/tests/support/fix_history_runner.rs");
  assert.ok(canon.includes("pub(crate) mod fix_history_runner;"));
  assert.ok(canon.includes("use support::fix_history_runner::check_pack;"));
  assert.equal(canon.includes("fn check_pack("), false);
  assert.ok(readRepoFile("crates/vize_canon/Cargo.toml").includes('name = "vize_canon"'));
  assert.equal(new Set(canonDeferred.map(({ name }) => name)).size, canonDeferred.length);
  for (const { name, pack } of canonDeferred) {
    assert.ok(canon.includes(`fn ${name}()`), `${name} must exist in this test binary`);
    const fixture = JSON.parse(
      readRepoFile(`tests/_fixtures/differential/typechecker/${pack}/cases.json`),
    );
    assert.equal(fixture.diagnosticContract.requiredTier, "T1");
  }
  assert.ok(runner.includes('assert_eq!(pack.diagnostic_contract.required_tier, "T1")'));

  const cohorts = workflow.jobs["pr-rust-shard"].steps;
  assert.ok(
    sourceWorkflow.jobs["pr-rust-build"].steps.some(
      (step) => step.name === "Run affected Rust doctests",
    ),
  );
  const full = (
    parseToml(readRepoFile(".config", "nextest.toml")) as {
      profile: { full: Record<string, unknown> };
    }
  ).profile.full;
  assert.equal(full["default-filter"], undefined);
  assert.equal(full.retries, 0);
  assert.equal(
    cohorts.some((step) => step.run?.includes("--profile pr")),
    false,
  );
});

const plan = {
  schemaVersion: 1,
  scope: "affected",
  packages: ["vize_armature", "vize_l1"],
  cargoArgs: ["--package", "vize_armature", "--package", "vize_l1"],
};

const stepName = "Build and run the affected Rust test cohort";
for (const event of ["merge_group", "pull_request"] as const) {
  const profile = "full";
  test(`${event} typechecker cohort requires tsgo and keeps the full filter`, () => {
    const cwd = mkdtempSync(join(tmpdir(), "vize-shard-prepare-"));
    try {
      const bin = join(cwd, "bin");
      mkdirSync(bin);
      const runtimeDir = join(cwd, "node_modules/@typescript/typescript-linux-x64/lib");
      mkdirSync(runtimeDir, { recursive: true });
      const nativeRuntime = join(runtimeDir, "tsc");
      writeFileSync(nativeRuntime, "#!/bin/sh\nexit 0\n", { mode: 0o755 });
      const argv = join(cwd, "argv.json");
      writeFileSync(
        join(bin, "cargo"),
        `#!/bin/sh
exec "$NODE" -e 'const fs=require("node:fs");const args=process.argv.slice(1);if(process.env.VIZE_TEST_REQUIRE_TSGO!=="1"||Object.hasOwn(process.env,"VIZE_TEST_DISABLE_TSGO"))process.exit(92);if(process.env.VIZE_NUXT_CONFIG_ITERATIONS!=="100")process.exit(93);if(process.env.CORSA_PATH!==process.env.EXPECTED_CORSA_PATH)process.exit(94);try{fs.accessSync(process.env.CORSA_PATH,fs.constants.X_OK)}catch{process.exit(95)}if(args.includes("--archive-file"))process.exit(91);fs.mkdirSync("target/nextest/"+process.env.NEXTEST_PROFILE,{recursive:true});fs.writeFileSync(process.env.ARGV,JSON.stringify(args));' -- "$@"
`,
        { mode: 0o755 },
      );
      const step = workflow.jobs["pr-rust-shard"].steps.find(
        (candidate) => candidate.name === stepName,
      );
      assert.ok(step?.run);
      const runtime = workflow.jobs["pr-rust-shard"].steps.find(
        (candidate) => candidate.name === "Select Rust runtime envelope",
      );
      assert.ok(runtime?.run);
      assert.equal(
        Object.hasOwn(workflow.jobs["pr-rust-shard"].env, "VIZE_TEST_DISABLE_TSGO"),
        false,
      );
      assert.equal(
        workflow.jobs["pr-rust-shard"].env.NEXTEST_PROFILE,
        "${{ (github.event_name == 'merge_group' || matrix.cohort == 'typechecker') && 'full' || 'pr' }}",
      );
      const environment: Record<string, string | undefined> = {
        ...process.env,
        NODE: process.execPath,
        PATH: `${bin}:${process.env.PATH ?? ""}`,
        GITHUB_WORKSPACE: cwd,
        GITHUB_ENV: join(cwd, "github-env"),
        GITHUB_EVENT_NAME: event,
        COHORT: "typechecker",
        EXPECTED_CORSA_PATH: nativeRuntime,
        NEXTEST_FILTER: "package(=vize_armature) | package(=vize_l1)",
        NEXTEST_PROFILE: profile,
        VIZE_NUXT_CONFIG_ITERATIONS: "100",
        ARGV: argv,
        RUNNER_TEMP: cwd,
        RUST_PLAN_JSON: JSON.stringify(plan),
      };
      // The PR step resolves the checked-out runner relative to the workspace.
      mkdirSync(join(cwd, "tools/support/compat/github"), { recursive: true });
      for (const name of [
        "run-affected-rust.ts",
        "rust-build-evidence.ts",
        "compare-unused-bindings-profiles.ts",
      ])
        copyFileSync(
          fileURLToPath(new URL(`../../tools/support/compat/github/${name}`, import.meta.url)),
          join(cwd, "tools/support/compat/github", name),
        );
      delete environment.VIZE_TEST_DISABLE_TSGO;
      delete environment.VIZE_TEST_REQUIRE_TSGO;
      delete environment.VIZE_TEST_CASES;
      delete environment.TEST_CASES_JOB;
      const result = spawnSync(
        "/bin/bash",
        [
          "--noprofile",
          "--norc",
          "-eo",
          "pipefail",
          "-c",
          `${runtime.run}\nset -a\nsource "$GITHUB_ENV"\nset +a\n${step.run}`,
        ],
        {
          cwd,
          encoding: "utf8",
          env: environment,
        },
      );
      assert.equal(result.status, 0, result.stderr);
      const filter = ["--profile", profile, "-E", environment.NEXTEST_FILTER, "--no-tests=pass"];
      // Both events build the same affected packages in place.
      {
        assert.deepEqual(JSON.parse(readFileSync(argv, "utf8")), [
          "nextest",
          "run",
          ...plan.cargoArgs,
          "--cargo-profile",
          "ci",
          "--timings",
          "--cargo-message-format=json-render-diagnostics",
          ...filter,
        ]);
      }
      assert.equal(
        (
          JSON.parse(readFileSync(join(cwd, `target/nextest/${profile}/timing.json`), "utf8")) as {
            cohort: string;
            exit_code: number;
          }
        ).exit_code,
        0,
      );
    } finally {
      rmSync(cwd, { recursive: true, force: true });
    }
  });
}

test("only the typechecker cohort requires tsgo on pull_request", () => {
  const runtime = workflow.jobs["pr-rust-shard"].steps.find(
    (candidate) => candidate.name === "Select Rust runtime envelope",
  );
  const script = runtime?.run;
  assert.ok(script);
  const cases = [
    ["pull_request", "typechecker", "require"],
    ["merge_group", "typechecker", "require"],
    ["pull_request", "compiler", "disable"],
    ["merge_group", "compiler", "require"],
    ["pull_request", "language-server", "disable"],
    ["merge_group", "language-server", "require"],
  ] as const;
  for (const [event, cohort, mode] of cases) {
    const cwd = mkdtempSync(join(tmpdir(), "vize-tsgo-envelope-"));
    try {
      const runtimeDir = join(cwd, "node_modules/@typescript/typescript-linux-x64/lib");
      mkdirSync(runtimeDir, { recursive: true });
      const nativeRuntime = join(runtimeDir, "tsc");
      writeFileSync(nativeRuntime, "#!/bin/sh\nexit 0\n", { mode: 0o755 });
      const githubEnv = join(cwd, "github-env");
      const result: SpawnSyncReturns<string> = spawnSync(
        "/bin/bash",
        ["--noprofile", "--norc", "-eo", "pipefail", "-c", script],
        {
          cwd,
          encoding: "utf8",
          env: {
            ...process.env,
            GITHUB_WORKSPACE: cwd,
            GITHUB_ENV: githubEnv,
            GITHUB_EVENT_NAME: event,
            COHORT: cohort,
            NEXTEST_PROFILE: cohort === "typechecker" || event === "merge_group" ? "full" : "pr",
            GITHUB_RUN_ID: "1",
            GITHUB_RUN_ATTEMPT: "1",
            GITHUB_SHA: "abc",
            INSTA_FORCE_PASS: "0",
          },
        },
      );
      assert.equal(result.status, 0, `${event} ${cohort}\n${result.stderr}`);
      const text = readFileSync(githubEnv, "utf8");
      if (mode === "require") {
        assert.match(text, /^VIZE_TEST_REQUIRE_TSGO=1$/m, `${event} ${cohort}`);
        assert.doesNotMatch(text, /VIZE_TEST_DISABLE_TSGO/, `${event} ${cohort}`);
      } else {
        assert.match(text, /^VIZE_TEST_DISABLE_TSGO=1$/m, `${event} ${cohort}`);
        assert.doesNotMatch(text, /VIZE_TEST_REQUIRE_TSGO/, `${event} ${cohort}`);
      }
      if (cohort === "typechecker") assert.match(text, /^CORSA_PATH=.*\/tsc$/m);
      else assert.doesNotMatch(text, /CORSA_PATH/);
    } finally {
      rmSync(cwd, { recursive: true, force: true });
    }
  }
});

const mapperSetupName = "Prepare the mandatory full-profile Content Mapper host";
test("the full-profile CLI cohort installs the independently pinned mapper host and retains its proof", () => {
  const steps = workflow.jobs["pr-rust-shard"].steps;
  const setup = steps.find((step) => step.name === mapperSetupName);
  assert.ok(setup);
  assert.equal(
    setup?.if,
    "${{ env.NEXTEST_PROFILE == 'full' && matrix.cohort == 'cli-and-integrations' }}",
  );
  assert.ok(steps.indexOf(setup) < steps.findIndex((step) => step.name === stepName));
  const artifact = steps.find((step) => step.name === "Retain the Content Mapper host proof");
  assert.equal(
    artifact?.if,
    "${{ always() && env.NEXTEST_PROFILE == 'full' && matrix.cohort == 'cli-and-integrations' }}",
  );
  assert.equal(artifact?.with?.path, "${{ runner.temp }}/mq-content-mapper-host-proof/");
});

for (const mode of ["success", "bootstrap-failure", "missing-executable"] as const) {
  test(`MQ mapper host wiring ${mode} preserves the bootstrap failure boundary`, () => {
    const script = workflow.jobs["pr-rust-shard"].steps.find(
      (step) => step.name === mapperSetupName,
    )?.run;
    assert.ok(script);
    const cwd = mkdtempSync(join(tmpdir(), "vize-mq-mapper-host-"));
    const secrets = [
      "GH_TOKEN",
      "GITHUB_TOKEN",
      "NODE_AUTH_TOKEN",
      "NPM_TOKEN",
      "NODE_OPTIONS",
      "NODE_PATH",
    ];
    try {
      const bin = join(cwd, "bin");
      mkdirSync(bin);
      // This shell stub tests wiring only: it performs no install or native execution.
      writeFileSync(
        join(bin, "node"),
        `#!/bin/sh
set -eu
test "$#" = 3
test "$1" = tools/support/release/type-admission-mapper-host.ts
test "$2" = "$RUNNER_TEMP/mq-content-mapper-host"
test "$3" = "$RUNNER_TEMP/mq-content-mapper-host-proof"
${secrets.map((name) => `test -z "\${${name}+x}"`).join("\n")}
if test "$MOCK_MODE" = bootstrap-failure; then exit 23; fi
mkdir -p "$3"
if test "$MOCK_MODE" = success; then
  mkdir -p "$2/node_modules/@typescript/typescript-linux-x64/lib"
  printf '#!/bin/sh\\nexit 0\\n' > "$2/node_modules/@typescript/typescript-linux-x64/lib/tsc"
  chmod +x "$2/node_modules/@typescript/typescript-linux-x64/lib/tsc"
fi
`,
        { mode: 0o755 },
      );
      const githubEnv = join(cwd, "github-env");
      const result = spawnSync(
        "/bin/bash",
        ["--noprofile", "--norc", "-eo", "pipefail", "-c", script],
        {
          cwd,
          encoding: "utf8",
          env: {
            ...process.env,
            ...Object.fromEntries(secrets.map((name) => [name, "must-be-removed"])),
            PATH: `${bin}:/usr/bin:/bin`,
            RUNNER_TEMP: cwd,
            GITHUB_ENV: githubEnv,
            MOCK_MODE: mode,
          },
        },
      );
      assert.equal(result.error, undefined);
      assert.equal(result.signal, null);
      if (mode === "success") {
        assert.equal(result.status, 0, result.stderr);
        assert.equal(
          readFileSync(githubEnv, "utf8"),
          [
            `VIZE_TEST_MEMO_CONTENT_MAPPER_TSGO=${cwd}/mq-content-mapper-host/node_modules/@typescript/typescript-linux-x64/lib/tsc`,
            `VIZE_TEST_RECOVERED_ATTRIBUTE_CONTENT_MAPPER_TSGO=${cwd}/mq-content-mapper-host/node_modules/@typescript/typescript-linux-x64/lib/tsc`,
            "",
          ].join("\n"),
        );
      } else {
        assert.equal(result.status, mode === "bootstrap-failure" ? 23 : 1, result.stderr);
        assert.equal(existsSync(githubEnv), false, "a failed preparation must not publish a host");
      }
    } finally {
      rmSync(cwd, { recursive: true, force: true });
    }
  });
}
