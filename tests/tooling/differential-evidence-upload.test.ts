import assert from "node:assert/strict";
import { spawnSync, type SpawnSyncReturns } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { parse } from "yaml";
import { checkWorkflowJobSources, readRepoFile } from "./support/github-workflows.ts";

type Step = {
  name?: string;
  if?: string;
  uses?: string;
  run?: string;
  with?: Record<string, string>;
};
type Workflow = {
  jobs: Record<string, { steps: Step[]; needs?: string | string[]; uses?: string }>;
};
const memberlessEvidenceFiles = [
  "without-editor/materialized-tsconfig.json",
  "without-editor/materialized-ui-package.json",
  "without-editor/prepared-vs-native.json",
  "with-editor/materialized-tsconfig.json",
  "with-editor/materialized-ui-package.json",
  "with-editor/prepared-vs-native.json",
  "ordered-without-editor/prepared-vs-native.json",
  "ordered-without-editor/materialized-tsconfig.json",
  "ordered-without-editor/materialized-ui-package.json",
  "ordered-without-editor/world.json",
  "ordered-without-editor/other-before-app/world.json",
  "ordered-without-editor/other-before-app/materialized-tsconfig.json",
  "ordered-without-editor/other-prepared-vs-native.json",
  "ordered-with-editor/prepared-vs-native.json",
  "ordered-with-editor/materialized-tsconfig.json",
  "ordered-with-editor/materialized-ui-package.json",
  "ordered-with-editor/world.json",
  "ordered-with-editor/other-before-app/world.json",
  "ordered-with-editor/other-before-app/materialized-tsconfig.json",
  "ordered-with-editor/other-prepared-vs-native.json",
  "ordered-without-editor/other-before-app/materialized-ui-package.json",
  "ordered-with-editor/other-before-app/materialized-ui-package.json",
] as const;
const actionPath = "./.github/actions/upload-formatter-api-corpus-evidence";
const action = parse(readRepoFile(actionPath.slice(2), "action.yml")) as {
  inputs: { shard: { default: string } };
  runs: { using: string; steps: Step[] };
};

test("shared evidence uploader retains every product raw tree and the selected CLI receipt", () => {
  assert.equal(action.runs.using, "composite");
  assert.equal(action.runs.steps.length, 1, "retention must not introduce another build");
  const upload = action.runs.steps[0];
  assert.match(upload.uses ?? "", /^actions\/upload-artifact@[0-9a-f]{40}$/);
  assert.deepEqual(upload.with?.path.trim().split("\n"), [
    "target/differential/",
    "target/ci/vize.differential-build.json",
  ]);
  assert.equal(upload.with?.["if-no-files-found"], "warn");
  assert.equal(upload.run, undefined);
});

test("evidence artifact identity preserves its prefix and separates jobs, shards and attempts", () => {
  assert.equal(action.inputs.shard.default, "full");
  const template = action.runs.steps[0].with?.name ?? "";
  assert.equal(
    template,
    "formatter-api-corpus-${{ github.run_id }}-${{ github.run_attempt }}-${{ github.job }}-${{ inputs.shard }}",
  );
  const identities = new Set<string>();
  for (const run of ["101", "102"]) {
    for (const attempt of ["1", "2"]) {
      for (const job of ["test-scripts", "pr-native-runtime"]) {
        for (const shard of ["full", "1", "2", "3", "4"]) {
          const identity = template
            .replace("${{ github.run_id }}", run)
            .replace("${{ github.run_attempt }}", attempt)
            .replace("${{ github.job }}", job)
            .replace("${{ inputs.shard }}", shard);
          assert.ok(!identities.has(identity), `colliding evidence upload ${identity}`);
          identities.add(identity);
        }
      }
    }
  }
  assert.equal(identities.size, 40);
});

test("the full tooling gate retains failed observations after its extracted tests", () => {
  const full = parse(checkWorkflowJobSources()) as Workflow;
  const fullProducer = parse(
    readRepoFile(".github", "workflows", "full-source-cli.yml"),
  ) as Workflow;
  const fullWorker = {
    ...full.jobs["test-scripts"],
    steps: (
      parse(readRepoFile(".github", "actions", "test-full-tooling", "action.yml")) as {
        runs: { steps: Step[] };
      }
    ).runs.steps,
  };
  assert.deepEqual(fullWorker.needs, ["full-source-cli", "tooling-plan"]);
  assert.equal(full.jobs["full-source-cli"].uses, "./.github/workflows/full-source-cli.yml");
  const call = full.jobs["test-scripts"].steps.find(
    (step) => step.uses === "./.github/actions/test-full-tooling",
  );
  assert.ok(call, "Check must invoke the extracted full tooling worker");
  assert.deepEqual(call.with, {
    producer: "${{ toJSON(needs.full-source-cli.outputs) }}",
    shard: "${{ matrix.index }}",
    total: "${{ matrix.total }}",
  });
  const restore = parse(readRepoFile(".github", "actions", "restore-source-cli", "action.yml")) as {
    runs: { steps: Step[] };
  };
  const job = fullWorker;
  const condition = "${{ always() }}";
  const shard = "${{ inputs.shard }}";
  const uploads = job.steps.filter((step) => step.uses === actionPath);
  assert.equal(uploads.length, 1);
  assert.equal(uploads[0].if, condition);
  assert.equal(uploads[0].with?.shard ?? action.inputs.shard.default, shard);
  assert.deepEqual(Object.keys(uploads[0].with ?? {}), ["shard"]);
  const uploadIndex = job.steps.indexOf(uploads[0]);
  const tests = job.steps.filter((step) => /test:scripts(?::pr)?\b/.test(step.run ?? ""));
  assert.ok(tests.length > 0);
  for (const step of tests) assert.ok(job.steps.indexOf(step) < uploadIndex);
  const build = job.steps.find((step) => step.uses === "./.github/actions/restore-source-cli");
  assert.ok(build, "uploaded CLI receipt must be made by the actual selected source build");
  assert.ok(job.steps.indexOf(build) < uploadIndex);
  assert.deepEqual(build.with, { producer: "${{ inputs.producer }}" });
  const originalBuild = fullProducer.jobs.cli.steps.find((step) =>
    /tests\/differential\/build-receipt\.ts/.test(step.run ?? ""),
  );
  assert.ok(originalBuild, "same-run producer makes the original uploaded receipt");
  assert.match(
    originalBuild.run ?? "",
    /cargo build --profile ci -p vize && vp exec node tests\/differential\/build-receipt\.ts/,
  );
  assert.equal(originalBuild.if, undefined);
  const downloads = restore.runs.steps.filter((step) =>
    step.uses?.startsWith("actions/download-artifact@"),
  );
  assert.equal(downloads.length, 1);
  assert.equal(downloads[0].with?.["digest-mismatch"], "error");
  const restoreRun = restore.runs.steps.at(-1)?.run ?? "";
  assert.match(restoreRun, /source-cli-artifact\.ts restore/);
  assert.match(restoreRun, /cat target\/ci\/vize\.differential-build\.json/);
  assert.doesNotMatch(restoreRun, /build-receipt\.ts/);
});

// #9198 retired the broad PR tooling shard. The live PR path still executes
// selected native product actions and remains required by both report levels.
test("the current PR native runtime lane remains connected to required source and Check reports", () => {
  const full = parse(checkWorkflowJobSources()) as Workflow;
  const pr = parse(readRepoFile(".github", "workflows", "pr-source-checks.yml")) as Workflow;
  assert.equal(full.jobs["pr-source-checks"].uses, "./.github/workflows/pr-source-checks.yml");
  const outerNeeds = full.jobs["test-report"].needs;
  assert.ok(Array.isArray(outerNeeds));
  assert.ok(outerNeeds.includes("pr-source-checks"));
  const sourceNeeds = pr.jobs["source-report"].needs;
  assert.ok(Array.isArray(sourceNeeds));
  assert.ok(sourceNeeds.includes("pr-native-runtime"));
  const runtime = pr.jobs["pr-native-runtime"];
  assert.ok(runtime);
  assert.equal(runtime.needs, "pr-source-plan");
  assert.deepEqual(
    runtime.steps
      .filter((step) => step.uses?.startsWith("./.github/actions/test-native-"))
      .map((step) => step.uses),
    [
      "./.github/actions/test-native-canon",
      "./.github/actions/test-native-lsp",
      "./.github/actions/test-native-runtime-selection",
      "./.github/actions/test-native-ssr",
      "./.github/actions/test-native-scoped-css",
      "./.github/actions/test-native-js-setup",
      "./.github/actions/test-native-vapor",
      "./.github/actions/test-native-attribute-values-7502",
      "./.github/actions/test-native-selected-template",
    ],
  );
});

test("native compiler-kind read qualification preserves the complete original snapshot command", () => {
  const action = parse(readRepoFile(".github", "actions", "test-native-canon", "action.yml")) as {
    runs: { steps: Array<{ name: string; run?: string; env?: Record<string, string> }> };
  };
  const step = action.runs.steps[0];
  assert.equal(step.name, "Test complete native snapshot text and owner refusals");
  assert.deepEqual(step.env, {
    VIZE_TEST_REQUIRE_TSGO: "1",
    VIZE_SNAPSHOT_SOURCE_CAPTURE_DIR: "${{ runner.temp }}/native-snapshot-source",
  });
  assert.deepEqual(step.run?.trim().split("\n"), [
    "unset VIZE_TEST_DISABLE_TSGO",
    "cargo test --locked --profile ci -p vize_canon --lib lsp_client::editor_lsp::snapshot_source::tests::native::actual_native_snapshot_retains_complete_stdlib_and_overlay_text_with_owner_refusals -- --ignored --exact --nocapture",
  ]);
  const real = action.runs.steps[1];
  assert.equal(
    real.name,
    "Test real compiler-kind diagnostics against complete native pull packets",
  );
  assert.deepEqual(real.env, step.env);
  assert.ok(
    real.run?.includes(
      'cargo test --locked --profile ci -p vize_canon --lib lsp_client::editor_lsp::snapshot_source::tests::native::actual_native_compiler_kind_reads_match_whole_ts_and_generated_vue_pull_packets -- --ignored --exact --nocapture --color never >"$capture/test.stdout" 2>"$capture/test.stderr"',
    ),
  );
  assert.ok(real.run?.includes("if (status !== 0) process.exit(status);"));
  assert.ok(real.run?.includes("if (executed !== 1) throw new Error"));
  assert.ok(real.run?.includes("1 passed; 0 failed; 0 ignored;"));
  assert.ok(real.run?.includes("'provider.json', 'source.ts.json', 'View.vue.ts.json'"));
  assert.ok(real.run?.includes("stdoutSha256: digest(stdout), stderrSha256: digest(stderr)"));
});

test("memberless preparation is enrolled after real Vue installation and requires whole same-owner evidence", () => {
  const action = parse(readRepoFile(".github", "actions", "test-native-canon", "action.yml")) as {
    runs: { steps: Array<{ name?: string; run?: string; env?: Record<string, string> }> };
  };
  const steps = action.runs.steps;
  const index = steps.findIndex(
    (step) =>
      step.name === "Test memberless Vue preparation against its acknowledged same-owner pull",
  );
  const install = steps.findIndex(
    (step) => step.name === "Install locked Vue dependency and pinned syntax corpus",
  );
  assert.ok(index > install && install > 1);
  const law = steps[index];
  assert.deepEqual(law.env, steps[1].env);
  assert.ok(
    law.run?.includes(
      'cargo test --locked --profile ci -p vize_canon --lib lsp_client::editor_lsp::configured_project::shared_transport::tests::native::replay::actual_memberless_vue_preparation_reattaches_same_editor_owner -- --ignored --exact --nocapture --color never >"$capture/test.stdout" 2>"$capture/test.stderr"',
    ),
  );
  assert.ok(
    law.run?.includes(
      "'without-editor/prepared-vs-native.json', 'without-editor/materialized-tsconfig.json', 'without-editor/materialized-ui-package.json', 'with-editor/prepared-vs-native.json', 'with-editor/materialized-tsconfig.json', 'with-editor/materialized-ui-package.json'",
    ),
  );
  assert.ok(law.run?.includes("['ordered-without-editor', 'ordered-with-editor']"));
  assert.ok(
    law.run?.includes(
      "'other-before-app/world.json', 'other-before-app/materialized-tsconfig.json', 'other-prepared-vs-native.json'",
    ),
  );
  const retention = steps.find(
    (step) => step.name === "Preserve actual native inputs and responses",
  );
  assert.ok(retention);
});

test("the actual compiler-kind carrier refuses zero execution and preserves whole output/status", () => {
  const action = parse(readRepoFile(".github", "actions", "test-native-canon", "action.yml")) as {
    runs: { steps: Array<{ run?: string }> };
  };
  for (const [name, step, files] of [
    [
      "lsp_client::editor_lsp::snapshot_source::tests::native::actual_native_compiler_kind_reads_match_whole_ts_and_generated_vue_pull_packets",
      action.runs.steps[1],
      ["source.ts.json", "View.vue.ts.json", "provider.json"],
    ],
    [
      "lsp_client::editor_lsp::configured_project::shared_transport::tests::native::replay::actual_memberless_vue_preparation_reattaches_same_editor_owner",
      action.runs.steps.find((step) =>
        step.run?.includes(
          "const name = 'lsp_client::editor_lsp::configured_project::shared_transport",
        ),
      ),
      memberlessEvidenceFiles,
    ],
  ] as const) {
    assert.ok(step);
    const source = step.run?.match(/<<'NODE'\n([\s\S]*?)\nNODE/)?.[1];
    assert.ok(source);
    const full = `test ${name} ... ok\n\ntest result: ok. 1 passed; 0 failed; 0 ignored; 123 filtered out; finished in 0.01s\n`;
    for (const [stdout, status, provider, expected] of [
      [full, 0, true, 0],
      [full.replace("1 passed", "0 passed"), 0, true, 1],
      [full.replace(name, "another_test"), 0, true, 1],
      [full, 17, true, 17],
      [full, 0, false, 1],
    ] as const) {
      const directory = mkdtempSync(join(tmpdir(), "vize-native-carrier-"));
      const stderr = "whole authored compiler stderr\n";
      try {
        writeFileSync(join(directory, "test.stdout"), stdout);
        writeFileSync(join(directory, "test.stderr"), stderr);
        writeFileSync(join(directory, "test.status"), `${status}\n`);
        for (const file of provider ? files : files.slice(0, -1)) {
          mkdirSync(dirname(join(directory, file)), { recursive: true });
          writeFileSync(join(directory, file), "{}\n");
        }
        const child: SpawnSyncReturns<string> = spawnSync(
          process.execPath,
          ["--input-type=module", "-", directory],
          {
            input: source,
            encoding: "utf8",
          },
        );
        assert.equal(child.status, expected, child.stderr);
        const receipt = JSON.parse(readFileSync(join(directory, "test-run.json"), "utf8"));
        assert.equal(receipt.status, status);
        assert.equal(receipt.name, name);
        assert.equal(receipt.stdoutSha256, createHash("sha256").update(stdout).digest("hex"));
        assert.equal(receipt.stderrSha256, createHash("sha256").update(stderr).digest("hex"));
        assert.equal(readFileSync(join(directory, "test.stdout"), "utf8"), stdout);
        assert.equal(readFileSync(join(directory, "test.stderr"), "utf8"), stderr);
      } finally {
        rmSync(directory, { recursive: true, force: true });
      }
    }
  }
});

test("ordered preparation requires exactly one complete first-world UI observation", () => {
  const action = parse(readRepoFile(".github", "actions", "test-native-canon", "action.yml")) as {
    runs: { steps: Array<{ run?: string }> };
  };
  const name =
    "lsp_client::editor_lsp::configured_project::shared_transport::tests::native::replay::actual_memberless_vue_preparation_reattaches_same_editor_owner";
  const step = action.runs.steps.find((step) => step.run?.includes(`const name = '${name}'`));
  assert.ok(step);
  const source = step.run?.match(/<<'NODE'\n([\s\S]*?)\nNODE/)?.[1];
  assert.ok(source);
  for (const [mode, expected] of [
    ["missing", 1],
    ["both", 1],
    ["typed-absence", 0],
    ["invalid-absence", 1],
  ] as const) {
    const directory = mkdtempSync(join(tmpdir(), "vize-first-world-ui-"));
    try {
      for (const file of memberlessEvidenceFiles) {
        mkdirSync(dirname(join(directory, file)), { recursive: true });
        writeFileSync(join(directory, file), "{}\n");
      }
      const world = join(directory, "ordered-without-editor", "other-before-app");
      if (mode !== "both") rmSync(join(world, "materialized-ui-package.json"));
      if (mode !== "missing")
        writeFileSync(
          join(world, "materialized-ui-package.absent.json"),
          JSON.stringify({
            status: mode === "invalid-absence" ? "unknown" : "absent-after-materialization",
            path: "/native/project/node_modules/ui/package.json",
          }),
        );
      writeFileSync(
        join(directory, "test.stdout"),
        `test ${name} ... ok\n\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 filtered out; finished in 0.1s\n`,
      );
      writeFileSync(join(directory, "test.stderr"), "whole first-world custody stderr\n");
      writeFileSync(join(directory, "test.status"), "0\n");
      const child: SpawnSyncReturns<string> = spawnSync(
        process.execPath,
        ["--input-type=module", "-", directory],
        { input: source, encoding: "utf8" },
      );
      assert.equal(child.status, expected, child.stderr);
    } finally {
      rmSync(directory, { recursive: true, force: true });
    }
  }
});
