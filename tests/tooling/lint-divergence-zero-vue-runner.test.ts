import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";

import {
  resolveBaselineRuntime,
  runBaseline,
} from "../../tools/support/compat/fixtures/lint-divergence-baseline.mjs";

import { runLintDivergenceReport } from "../../tools/support/compat/fixtures/lint-divergence-report.mjs";

const root = path.resolve(import.meta.dirname, "../..");

function fixture() {
  fs.mkdirSync(path.join(root, "target"), { recursive: true });
  const directory = fs.mkdtempSync(path.join(root, "target", "lint-zero-vue-"));
  const marker = path.join(directory, "lint-invocations.json");
  const binary = path.join(directory, "vize.mjs");
  fs.writeFileSync(
    binary,
    [
      "#!/usr/bin/env node",
      'import fs from "node:fs";',
      "if (process.argv[2] === '--version') process.exit(0);",
      "if (process.argv[2] !== 'lint') process.exit(2);",
      `fs.appendFileSync(${JSON.stringify(marker)}, JSON.stringify(process.argv.slice(2)) + '\\n');`,
      "process.stdout.write(JSON.stringify(fs.existsSync('App.vue') ? [{ file: 'App.vue', messages: [] }] : []));",
      "",
    ].join("\n"),
  );
  fs.chmodSync(binary, 0o755);
  return {
    directory,
    marker,
    writeRegistry(expectedVueFileCount?: unknown) {
      const registry = path.join(directory, "registry.json");
      fs.writeFileSync(
        registry,
        JSON.stringify({
          projects: [
            {
              id: "zero-vue",
              revision: "0".repeat(40),
              fixturePath: path.relative(root, directory),
              vueGlobs: ["**/*.vue"],
              coverage: ["linter"],
              expectedVueFileCount,
            },
          ],
        }),
      );
      return [
        "--registry",
        registry,
        "--output-dir",
        path.join(directory, "report"),
        "--vize-bin",
        binary,
        "--budget-mode",
        "enforce",
        "--timeout-ms",
        "30000",
      ];
    },
    clean() {
      fs.rmSync(directory, { recursive: true, force: true });
    },
  };
}

function rust(args: string[]) {
  const result = spawnSync(
    "rust-script",
    ["tools/commands/fixtures/lint-divergence-report.rs", ...args],
    {
      cwd: root,
      encoding: "utf8",
      env: { ...process.env, LANG: "C", LC_ALL: "C" },
    },
  );
  assert.equal(result.error, undefined, result.error?.message);
  return result;
}

test("both real reporters enforce an explicitly expected zero-Vue corpus", async () => {
  const input = fixture();
  try {
    fs.writeFileSync(path.join(input.directory, "Unrelated.js"), "export const =\n");
    const args = input.writeRegistry(0);
    const measured = rust(args);
    assert.equal(measured.status, 0, `${measured.stdout}\n${measured.stderr}`);
    const artifactPath = path.join(input.directory, "report", "zero-vue-lint-divergence.json");
    const actual = JSON.parse(fs.readFileSync(artifactPath, "utf8"));
    const [compatible] = await runLintDivergenceReport(args);
    for (const artifact of [actual, compatible]) {
      assert.deepEqual(artifact.files, { comparedCount: 0, expectedCount: 0 });
      assert.ok(artifact.baseline.comparedRuleCount > 0);
      assert.equal(artifact.divergence.summary.falsePositiveCount, 0);
      assert.equal(artifact.divergence.summary.falseNegativeCount, 0);
      assert.equal(artifact.divergence.summary.baselineParseErrorCount, 0);
      assert.equal(artifact.divergence.summary.baselineExcludedNonVueCount, 0);
      assert.equal(artifact.budget.unusableReason, null);
      assert.equal(artifact.budget.passed, true);
      assert.equal(artifact.budget.maxFalsePositiveCount, 0);
      assert.equal(artifact.budget.maxFalseNegativeCount, 0);
    }
    const index = JSON.parse(
      fs.readFileSync(path.join(input.directory, "report", "lint-divergence-summary.json"), "utf8"),
    );
    assert.equal(index.projectCount, 1);
    assert.equal(index.budget.passed, true);
    assert.equal(index.budget.unusableCount, 0);
    assert.equal(fs.readFileSync(input.marker, "utf8").trim().split("\n").length, 2);
  } finally {
    input.clean();
  }
});

test("neither reporter grants empty-input authority without the explicit zero expectation", async () => {
  for (const expected of [undefined, 1, -1, "0", null]) {
    const input = fixture();
    try {
      const args = input.writeRegistry(expected);
      const measured = rust(args);
      assert.equal(measured.status, 1, `${measured.stdout}\n${measured.stderr}`);
      assert.match(measured.stderr, /zero-vue matched no Vue files/u);
      await assert.rejects(runLintDivergenceReport(args), /zero-vue matched no Vue files/u);
      assert.equal(fs.existsSync(input.marker), false);
      assert.equal(
        fs.existsSync(path.join(input.directory, "report", "zero-vue-lint-divergence.json")),
        false,
      );
    } finally {
      input.clean();
    }
  }
});

test("an expected zero-Vue fixture containing a Vue input fails before either linter", async () => {
  const input = fixture();
  try {
    fs.writeFileSync(path.join(input.directory, "Unexpected.vue"), "<template><p/></template>\n");
    const args = input.writeRegistry(0);
    const measured = rust(args);
    assert.equal(measured.status, 1, `${measured.stdout}\n${measured.stderr}`);
    assert.match(measured.stderr, /zero-vue matched 1 Vue files, expected 0/u);
    await assert.rejects(
      runLintDivergenceReport(args),
      /zero-vue matched 1 Vue files, expected 0/u,
    );
    assert.equal(fs.existsSync(input.marker), false);
  } finally {
    input.clean();
  }
});

test("a nonempty Vue surface still enforces its actual mapped diagnostic budget", async () => {
  const input = fixture();
  try {
    fs.writeFileSync(
      path.join(input.directory, "App.vue"),
      '<template><p v-html="html"/></template>\n',
    );
    const args = input.writeRegistry(1);
    const measured = rust(args);
    assert.equal(measured.status, 1, `${measured.stdout}\n${measured.stderr}`);
    assert.match(measured.stderr, /Lint divergence budget breached for zero-vue/u);
    const artifactPath = path.join(input.directory, "report", "zero-vue-lint-divergence.json");
    const actual = JSON.parse(fs.readFileSync(artifactPath, "utf8"));
    await assert.rejects(
      runLintDivergenceReport(args),
      /Lint divergence budget breached for zero-vue/u,
    );
    const compatible = JSON.parse(fs.readFileSync(artifactPath, "utf8"));
    for (const artifact of [actual, compatible]) {
      assert.deepEqual(artifact.files, { comparedCount: 1, expectedCount: 1 });
      assert.equal(artifact.divergence.summary.falseNegativeCount, 1);
      assert.equal(artifact.divergence.falseNegatives[0].upstreamRuleId, "vue/no-v-html");
      assert.equal(artifact.budget.maxFalseNegativeCount, 0);
      assert.equal(artifact.budget.passed, false);
      assert.equal(artifact.budget.verdict, "breached");
    }
  } finally {
    input.clean();
  }
});

test("both actual ESLint entry points validate mapped rules without linting an empty corpus", async () => {
  const input = fixture();
  try {
    const source = fs.readFileSync(
      path.join(root, "tools/commands/fixtures/lint-divergence-report.rs"),
      "utf8",
    );
    const script = source.match(/let script = r#"([\s\S]*?)"#;/u)?.[1];
    assert.ok(script, "the actual Rust runner's retained ESLint entry point must exist");
    const runtime = resolveBaselineRuntime();
    for (const rules of [
      { "vue/no-such-mapped-rule": "warn" },
      { "vue/no-v-html": ["warn", { invalidOption: true }] },
    ]) {
      await assert.rejects(runBaseline(runtime, input.directory, [], rules), /Key "rules"/u);
      const measured = spawnSync(process.execPath, ["--input-type=module", "--eval", script], {
        cwd: root,
        encoding: "utf8",
        input: JSON.stringify({
          benchPackageJson: path.join(root, "tools/benchmarks/scripts/package.json"),
          cwd: input.directory,
          files: [],
          rules,
        }),
      });
      assert.equal(measured.error, undefined, measured.error?.message);
      assert.equal(measured.status, 1, `${measured.stdout}\n${measured.stderr}`);
      assert.match(measured.stderr, /Key "rules"/u);
      assert.equal(measured.stdout, "");
    }
    assert.equal(fs.existsSync(path.join(input.directory, "__vize_empty_vue_config__.vue")), false);
  } finally {
    input.clean();
  }
});
