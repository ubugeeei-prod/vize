import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
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

test("fuzz CI workflow runs strict nightly and dispatched campaigns", () => {
  const workflow = readRepoFile(".github/workflows/fuzz.yml");
  const parsed = parse(workflow) as {
    "run-name": string;
    jobs: {
      fuzz: {
        "continue-on-error": boolean;
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
  assert.match(workflow, /workflow_dispatch:/);
  assert.doesNotMatch(workflow, /\n  pull_request:/);

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
  assert.equal(fuzzJob["continue-on-error"], false);
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

test("seed_corpus.rs writes seeds for every declared fuzz target", () => {
  const script = readRepoFile("tools/commands/ci/fuzz/seed_corpus.rs");
  const manifest = readRepoFile(fuzzManifestPath);
  const targets = [...manifest.matchAll(/\[\[bin\]\]\s+name = "([^"]+)"/g)].map(([, name]) => name);

  assert.match(script, /tests\/fuzz\/corpus/);
  for (const target of targets) {
    assert.match(
      script,
      new RegExp(`reset_corpus\\(&corpus_root, "${target}"\\)`),
      `seed_corpus.rs must seed corpus/${target}/`,
    );
  }
});

test("SFC crash regressions survive corpus regeneration byte-for-byte", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-fuzz-regression-seeds-"));
  try {
    fs.writeFileSync(path.join(root, "Cargo.toml"), "[workspace]\n");
    fs.writeFileSync(path.join(root, "pnpm-workspace.yaml"), "packages: []\n");
    const regression = readRepoFile("tests/fuzz/regressions/sfc_parse/issue-6277.txt");
    const digest = createHash("sha1").update(regression).digest("hex");
    assert.equal(digest, "cb5e9e231223f2ca26ea0c8afa91c13ebc01b75e");
    const directory = path.join(root, "tests/fuzz/regressions/sfc_parse");
    fs.mkdirSync(directory, { recursive: true });
    const fixture = path.join(directory, "issue-6277.txt");
    const corpus = path.join(root, "tests/fuzz/corpus/sfc_parse");
    const seed = () => {
      const result = spawnSync(
        "rust-script",
        [path.join(repoRoot, "tools/commands/ci/fuzz/seed_corpus.rs")],
        { encoding: "utf8", env: { ...process.env, VIZE_REPO_ROOT: root } },
      );
      assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);
      assert.match(result.stdout, /Seeded 1 sfc_parse entries/);
    };
    fs.writeFileSync(fixture, regression);
    seed();
    assert.equal(fs.readFileSync(path.join(corpus, digest.slice(0, 16)), "utf8"), regression);
    const mutated = `${regression}\n<!-- independent corpus mutation -->\n`;
    fs.writeFileSync(fixture, mutated);
    seed();
    assert.equal(fs.existsSync(path.join(corpus, digest.slice(0, 16))), false);
    const changedDigest = createHash("sha1").update(mutated).digest("hex").slice(0, 16);
    assert.deepEqual(fs.readdirSync(corpus), [changedDigest]);
    assert.equal(fs.readFileSync(path.join(corpus, changedDigest), "utf8"), mutated);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

test("opaque native Program regressions seed all eight profiles with exact source bytes", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-program-regression-seeds-"));
  try {
    fs.writeFileSync(path.join(root, "Cargo.toml"), "[workspace]\n");
    fs.writeFileSync(path.join(root, "pnpm-workspace.yaml"), "packages: []\n");
    const directory = path.join(root, "tests/fuzz/regressions/l1_program");
    fs.mkdirSync(directory, { recursive: true });
    const witnesses = [
      [
        "native-module.js.input",
        "734d3c83a5d641f07b083ba97036a71daa5fa74c09ef8de7897df0afa2c3243e",
      ],
      [
        "native-module.tsx.input",
        "a2fa37c7423b29f92f797b76abb06fab6ad3b365bbc2d854b3cb279086534d6e",
      ],
      [
        "pure-recovery.js.input",
        "70fa10164b219c26375100fa28b763bc12fce85423b19932c2969cdcfa2fb979",
      ],
      ["pure-rewind.js.input", "4d512b93149545b991c03f143efabaf7510feacff8125724edc6b0610ccf1ec7"],
      [
        "pure-ts-operator.ts.input",
        "cb91e97e41d2202545925f335bd74794a9176bd2d0fd8c6dd29803ae7cce5e54",
      ],
      [
        "type-argument-backtracking.tsx.input",
        "3611249f2fbc19493b3f5170c372265bbfe804ef0472bd54b27782ee2aaa25e5",
      ],
      [
        "type-argument-slow-unit.tsx.input",
        "2c66018e8af55e8555b14198db25bd8b18e986f0ccbeaa8f0616b5bbaa615cd1",
      ],
      [
        "type-argument-computed-key.ts.input",
        "795604dd30265beeeec984c6c5500943dd5c8cf1b2bad9aed4b55ceb9d3ca04a",
      ],
    ] as const;
    const inputs = witnesses.map(([name, digest]) => {
      const bytes = fs.readFileSync(path.join(repoRoot, "tests/fuzz/regressions/l1_program", name));
      assert.equal(createHash("sha256").update(bytes).digest("hex"), digest);
      fs.writeFileSync(path.join(directory, name), bytes);
      return bytes;
    });
    const result = spawnSync(
      "rust-script",
      [path.join(repoRoot, "tools/commands/ci/fuzz/seed_corpus.rs")],
      { encoding: "utf8", env: { ...process.env, VIZE_REPO_ROOT: root } },
    );
    assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);
    assert.match(result.stdout, /8 native Program sources in eight explicit profiles/);
    const corpus = path.join(root, "tests/fuzz/corpus/l1_program");
    assert.equal(fs.readdirSync(corpus).length, 64);
    for (const source of inputs) {
      for (let profile = 0; profile < 8; profile++) {
        const bytes = Buffer.concat([Buffer.from([profile]), source]);
        const digest = createHash("sha1").update(bytes).digest("hex").slice(0, 16);
        assert.deepEqual(fs.readFileSync(path.join(corpus, digest)), bytes);
      }
    }
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});
