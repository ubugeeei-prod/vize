import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { parse } from "yaml";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const fuzzWorkspace = "tests/fuzz";
const fuzzManifestPath = `${fuzzWorkspace}/Cargo.toml`;

function readRepoFile(relativePath: string): string {
  return fs.readFileSync(path.join(repoRoot, relativePath), "utf8");
}

test("fuzz workspace declares libfuzzer-sys and an isolated [workspace]", () => {
  const manifest = readRepoFile(fuzzManifestPath);

  // The fuzz crate must be its own workspace so the root workspace stable
  // toolchain is not pinned to libfuzzer-sys's nightly requirement.
  assert.match(manifest, /^\[workspace\]\s*$/m);

  assert.match(manifest, /libfuzzer-sys\s*=\s*"0\.4"/);
  assert.match(manifest, /cargo-fuzz\s*=\s*true/);

  // Every declared bin target must have a corresponding harness file so
  // `cargo fuzz run <target>` resolves cleanly on CI.
  const binMatches = [...manifest.matchAll(/\[\[bin\]\]\s+name = "([^"]+)"\s+path = "([^"]+)"/g)];
  assert.ok(binMatches.length > 0, `${fuzzManifestPath} must declare at least one [[bin]] target`);
  for (const [, , relativePath] of binMatches) {
    const fullPath = path.join(repoRoot, fuzzWorkspace, relativePath);
    assert.ok(
      fs.existsSync(fullPath),
      `fuzz target file ${relativePath} declared in Cargo.toml is missing`,
    );
  }
});

test("fuzz CI workflow gates short PR fuzz and schedules long nightly fuzz", () => {
  const workflow = readRepoFile(".github/workflows/fuzz.yml");
  const parsed = parse(workflow) as {
    "run-name": string;
    jobs: {
      fuzz: {
        "continue-on-error": string;
        steps: Array<{
          "continue-on-error"?: boolean;
          env?: Record<string, string>;
          id?: string;
          if?: string;
          run?: string;
        }>;
      };
    };
  };

  assert.match(workflow, /name:\s*Fuzz/);
  // Assert the resolved `run-name` value, not how the YAML wraps it: the
  // release gate correlates evidence by expanded display title.
  assert.match(parsed["run-name"], /^Fuzz \$\{\{/);
  assert.match(parsed["run-name"], /inputs\.mode == 'replay'/);
  assert.match(parsed["run-name"], /inputs\.max-total-time/);
  assert.match(parsed["run-name"], /@ \$\{\{ github\.sha \}\}$/);
  assert.match(workflow, /schedule:[\s\S]*?-\s*cron:/);
  assert.match(workflow, /pull_request:[\s\S]*paths:/);
  assert.match(workflow, /"tests\/fuzz\/\*\*"/);
  assert.doesNotMatch(workflow, /"fuzz\/\*\*"/);

  // The matrix must drive each fuzz_target declared in tests/fuzz/Cargo.toml.
  const manifest = readRepoFile(fuzzManifestPath);
  const targets = [...manifest.matchAll(/\[\[bin\]\]\s+name = "([^"]+)"/g)].map(([, name]) => name);
  for (const target of targets) {
    assert.match(
      workflow,
      new RegExp(`target:\\s*\\[[^\\]]*${target}[^\\]]*\\]`),
      `fuzz workflow matrix missing ${target}`,
    );
  }

  const fuzzJob = parsed.jobs.fuzz;
  assert.equal(fuzzJob["continue-on-error"], "${{ github.event_name == 'pull_request' }}");
  const budgetStep = fuzzJob.steps.find((step) => step.id === "budget");
  assert.deepEqual(budgetStep?.env, {
    REQUESTED_MAX_TOTAL_TIME: "${{ inputs.max-total-time }}",
    FUZZ_MODE: "${{ inputs.mode || 'campaign' }}",
  });
  assert.match(budgetStep?.run ?? "", /seconds="\$REQUESTED_MAX_TOTAL_TIME"/);
  assert.match(budgetStep?.run ?? "", /max-total-time must be an integer from 1 to 3600 seconds/);
  assert.doesNotMatch(budgetStep?.run ?? "", /seconds=\$\{\{/);
  const dispatchBudgetScript = (budgetStep?.run ?? "").replaceAll(
    "${{ github.event_name }}",
    "workflow_dispatch",
  );
  for (const [input, expectedStatus] of [
    ["abc", 1],
    ["0", 1],
    ["3601", 1],
    ["1", 0],
    ["3600", 0],
  ] as const) {
    const output = path.join(fs.mkdtempSync(path.join(os.tmpdir(), "vize-fuzz-budget-")), "output");
    const result = spawnSync("bash", ["-c", dispatchBudgetScript], {
      encoding: "utf8",
      env: {
        ...process.env,
        GITHUB_OUTPUT: output,
        REQUESTED_MAX_TOTAL_TIME: input,
      },
    });
    assert.equal(result.status, expectedStatus, `${input}: ${result.stderr}${result.stdout}`);
    if (expectedStatus === 0) {
      assert.equal(fs.readFileSync(output, "utf8"), `seconds=${input}\n`);
    } else {
      assert.match(`${result.stderr}${result.stdout}`, /Invalid fuzz budget/);
    }
  }
  const fuzzStep = fuzzJob.steps.find((step) => step.id === "fuzz");
  assert.equal(fuzzStep?.["continue-on-error"], true);
  assert.match(
    fuzzStep?.run ?? "",
    /cargo \+nightly fuzz run --fuzz-dir tests\/fuzz "\$FUZZ_TARGET"/,
  );
  assert.match(fuzzStep?.run ?? "", /-rss_limit_mb=4096/);
  assert.match(fuzzStep?.run ?? "", /-malloc_limit_mb=2048/);
  // Mode selection belongs to this step: replay executes each corpus input once
  // and exits, campaigns keep their budget. `release-fuzz-gate.test.ts` runs the
  // script per mode to pin which argument each one actually gets.
  assert.match(fuzzStep?.run ?? "", /budget="-runs=0"/);
  assert.match(fuzzStep?.run ?? "", /budget="-max_total_time=\$FUZZ_MAX_TOTAL_TIME"/);
  assert.deepEqual(fuzzStep?.env, {
    FUZZ_MAX_TOTAL_TIME: "${{ steps.budget.outputs.seconds }}",
    FUZZ_TARGET: "${{ matrix.target }}",
    FUZZ_MODE: "${{ inputs.mode || 'campaign' }}",
  });
  const enforceStep = fuzzJob.steps.find((step) => step.id === "enforce");
  assert.match(enforceStep?.run ?? "", /tools\/commands\/ci\/fuzz\/enforce-result\.rs/);
  assert.deepEqual(enforceStep?.env, {
    FUZZ_EVENT_NAME: "${{ github.event_name }}",
    FUZZ_OUTCOME: "${{ steps.fuzz.outcome || 'skipped' }}",
    FUZZ_TARGET: "${{ matrix.target }}",
  });

  // Reproducers on failure must be uploaded so triage does not have to
  // re-run the fuzzer to recover the failing input.
  assert.match(workflow, /upload-artifact[\s\S]*tests\/fuzz\/artifacts\//);
  assert.match(workflow, /issues:\s*write/);
  const uploadStep = fuzzJob.steps.find((step) => step.id === "upload-reproducers");
  assert.equal(
    uploadStep?.if,
    "always() && hashFiles(format('tests/fuzz/artifacts/{0}/**', matrix.target)) != ''",
  );
  const triageStep = fuzzJob.steps.find((step) => step.id === "triage");
  assert.equal(
    triageStep?.if,
    "always() && (github.event_name == 'schedule' || github.event_name == 'workflow_dispatch') && hashFiles(format('tests/fuzz/artifacts/{0}/**', matrix.target)) != ''",
  );
  assert.match(workflow, /gh issue (create|comment)/);
});

test("fuzz workspace covers parser, lexer, and compiler harnesses", () => {
  const manifest = readRepoFile(fuzzManifestPath);

  for (const target of [
    "sfc_parse",
    "template_lexer",
    "js_ts_expression",
    "css_parse",
    "template_compile",
  ]) {
    assert.match(
      manifest,
      new RegExp(`name = "${target}"[\\s\\S]*path = "fuzz_targets/${target}\\.rs"`),
      `fuzz workspace missing ${target}`,
    );
  }

  assert.match(manifest, /oxc_parser\s*=/);
  assert.match(manifest, /features = \[\s*"native",?\s*\]/);
});

test("seed_corpus.mjs writes seeds for every declared fuzz target", () => {
  const script = readRepoFile("tools/fuzz/seed_corpus.mjs");
  const manifest = readRepoFile(fuzzManifestPath);
  const targets = [...manifest.matchAll(/\[\[bin\]\]\s+name = "([^"]+)"/g)].map(([, name]) => name);

  assert.match(script, /tests", "fuzz", "corpus"/);
  for (const target of targets) {
    assert.match(
      script,
      new RegExp(`resetCorpus\\("${target}"\\)`),
      `seed_corpus.mjs must seed corpus/${target}/`,
    );
  }
});
