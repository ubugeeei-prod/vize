import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { parse as parseToml } from "@iarna/toml";
import { parse } from "yaml";
import { readRepoFile } from "./support/github-workflows.ts";

const deferred = [
  "ide::rename::corsa_session_tests::concurrent_real_corsa_rename_sessions_are_isolated",
  "ide::rename::corsa_session_tests::direct_first::dependency_change_rearms_the_shared_editor_transport_once",
  "ide::rename::corsa_session_tests::direct_first::direct_first_renames_survive_twenty_session_shutdown_overlap",
];
type Step = { name?: string; run?: string };
const workflow = parse(readRepoFile(".github", "workflows", "pr-rust-checks.yml")) as {
  jobs: Record<string, { steps: Step[]; env: Record<string, string> }>;
};

test("only the three audited real-TSGO cases leave the PR default filter", () => {
  const config = parseToml(readRepoFile(".config", "nextest.toml")) as {
    profile: { pr: { "default-filter": string }; default?: { "default-filter"?: string } };
  };
  assert.equal(
    config.profile.pr["default-filter"].replace(/\s/g, ""),
    `not(package(=vize_maestro)&(${deferred.map((name) => `test(=${name})`).join("|")}))`,
  );
  assert.equal(config.profile.default?.["default-filter"], undefined);
  const source = readRepoFile("crates/vize_maestro/src/ide/rename/corsa_session_tests.rs");
  const direct = readRepoFile(
    "crates/vize_maestro/src/ide/rename/corsa_session_tests/direct_first.rs",
  );
  for (const name of deferred) {
    const functionName = name.split("::").at(-1);
    const body = name.includes("::direct_first::") ? direct : source;
    assert.ok(body.includes(`fn ${functionName}()`), `${name} must exist`);
  }
  assert.ok(source.includes("real-Corsa isolation coverage cannot be skipped"));
  const merge = workflow.jobs["merge-rust-source"].steps;
  assert.ok(merge.some((step) => step.name === "Run Rust workspace doctests"));
  const full = (
    parseToml(readRepoFile(".config", "nextest.toml")) as {
      profile: { full: Record<string, unknown> };
    }
  ).profile.full;
  assert.equal(full["default-filter"], undefined);
  assert.equal(full.retries, 0);
  assert.equal(
    merge.some((step) => step.run?.includes("--profile pr")),
    false,
  );
});

for (const [event, profile] of [
  ["merge_group", "full"],
  ["pull_request", "pr"],
]) {
  test(`${profile} shard shell restores Cargo paths with the exclusive runtime flag`, () => {
    const cwd = mkdtempSync(join(tmpdir(), "vize-shard-prepare-"));
    try {
      const bin = join(cwd, "bin");
      mkdirSync(bin);
      const argv = join(cwd, "argv.json");
      writeFileSync(
        join(bin, "cargo"),
        `#!/bin/sh
exec "$NODE" -e 'const fs=require("node:fs");const args=process.argv.slice(1);if(process.env.NEXTEST_PROFILE==="full"?(process.env.VIZE_TEST_REQUIRE_TSGO!=="1"||Object.hasOwn(process.env,"VIZE_TEST_DISABLE_TSGO")):(process.env.VIZE_TEST_DISABLE_TSGO!=="1"||Object.hasOwn(process.env,"VIZE_TEST_REQUIRE_TSGO")))process.exit(92);if(process.env.VIZE_NUXT_CONFIG_ITERATIONS!=="100")process.exit(93);if(!args.includes("--extract-overwrite"))process.exit(91);fs.writeFileSync("target/tmp/observation","writable");fs.mkdirSync("target/nextest/"+process.env.NEXTEST_PROFILE,{recursive:true});fs.writeFileSync(process.env.ARGV,JSON.stringify(args));' -- "$@"
`,
        { mode: 0o755 },
      );
      const step = workflow.jobs["pr-rust-shard"].steps.find(
        (candidate) => candidate.name === "Run Rust test shard without rebuilding",
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
      const environment: Record<string, string | undefined> = {
        ...process.env,
        NODE: process.execPath,
        PATH: `${bin}:${process.env.PATH ?? ""}`,
        GITHUB_WORKSPACE: cwd,
        GITHUB_ENV: join(cwd, "github-env"),
        GITHUB_EVENT_NAME: event,
        SHARD: "2",
        NEXTEST_PROFILE: profile,
        VIZE_NUXT_CONFIG_ITERATIONS: "100",
        ARGV: argv,
      };
      delete environment.VIZE_TEST_DISABLE_TSGO;
      delete environment.VIZE_TEST_REQUIRE_TSGO;
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
      assert.equal(readFileSync(join(cwd, "target/tmp/observation"), "utf8"), "writable");
      assert.deepEqual(JSON.parse(readFileSync(argv, "utf8")), [
        "nextest",
        "run",
        "--archive-file",
        ".artifacts/rust-test-archive/tests.tar.zst",
        "--extract-to",
        cwd,
        "--extract-overwrite",
        "--workspace-remap",
        cwd,
        "--profile",
        profile,
        "--partition",
        "hash:2/4",
        "--no-tests=pass",
      ]);
    } finally {
      rmSync(cwd, { recursive: true, force: true });
    }
  });
}
