import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import { expectedBuildIdentity, validateBuildReceipt } from "../differential/build-receipt.ts";
import { sha256, wholeJson } from "./lint-default-correctness-reference.mjs";
import { withoutHelp } from "./support/lint-help-policy.mjs";
import { vueConfigFixtureSource, writeVueConfigFixture } from "./support/vue-config-fixture.ts";
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
await test("registered reserved-key suppression preserves whole CLI packets and source boundaries", () => {
  const fixture = path.join(root, "tests/_fixtures/differential/lint/reserved-suppression-8090");
  const corpus = JSON.parse(fs.readFileSync(path.join(fixture, "cases.json")));
  const successors = JSON.parse(
    fs.readFileSync(path.join(fixture, "default-route-successors.json")),
  );
  const projection = JSON.parse(
    fs.readFileSync(path.join(root, "tests/_fixtures/n8n-cli-adoption.json")),
  );
  const original = JSON.parse(
    fs.readFileSync(path.join(fixture, "original-vue-select-successor.json")),
  );
  original.source = fs.readFileSync(path.join(fixture, original.sourceFile), "utf8");
  assert.equal(corpus.cases.length, 24);
  assert.equal(
    corpus.cases.reduce((n, c) => n + c.beforeDiagnostics.length, 0),
    22,
  );
  assert.equal(
    corpus.cases.reduce((n, c) => n + c.diagnostics.length, 0),
    12,
  );
  assert.equal(projection.linter.rules[corpus.rule], undefined);
  const custodyFiles = [
    "cases.json",
    "original-public-before.json",
    "observe-public-before.mjs",
    "initial-twenty-two-before.json.gz",
    "original-vue-select-successor.json",
    "original-prefix-successors.json",
    "original-suppression-oracle.json",
    "source-authority.json",
    "default-route-successors.json",
    "default-route-public-before.json",
    "observe-default-route-before.mjs",
    "default-route-authority.json",
    "initial-source-failure.json",
    "initial-default-route-before.json.gz",
    "second-default-route-before.json.gz",
    "../reserved-prefix-8090/original-vue-select-src-components-select.vue.fixture",
  ];
  const custody = () => [
    ...custodyFiles.map((file) => ({
      path: path.relative(root, path.join(fixture, file)),
      sha256: sha256(fs.readFileSync(path.join(fixture, file))),
    })),
    ...[
      "tests/tooling/lint-reserved-suppression.test.mjs",
      "tests/tooling/lint-default-correctness-reference.mjs",
      "tests/differential/build-receipt.ts",
      "tests/_fixtures/n8n-cli-adoption.json",
    ].map((file) => ({ path: file, sha256: sha256(fs.readFileSync(path.join(root, file))) })),
  ];
  const artifact = path.join(root, "target/differential/lint-reserved-suppression.json");
  fs.mkdirSync(path.dirname(artifact), { recursive: true });
  const evidence = {
    schema: "vize.lint.reserved-suppression-cli",
    version: 1,
    corpus,
    original,
    custody: custody(),
    runs: [],
  };
  const persist = () => fs.writeFileSync(artifact, JSON.stringify(evidence, null, 2) + "\n");
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-reserved-suppression-"));
  try {
    const build = expectedBuildIdentity(root);
    const receipt = JSON.parse(
      fs.readFileSync(path.join(root, build.binaryPath + ".differential-build.json")),
    );
    evidence.build = { expected: build, receipt };
    persist();
    validateBuildReceipt(receipt, build);
    const scenarios = corpus.cases.flatMap((entry, index) => {
      const successor = successors.cases[index];
      assert.equal(successor.id, entry.id);
      assert.equal(successor.sourceSha256, entry.sourceSha256);
      return [
        ...[null, "recommended", "essential", "ecosystem"].map((preset) => ({
          entry,
          preset,
          config: null,
          findings:
            preset === "essential" ? successor.essentialDiagnostics : successor.defaultDiagnostics,
        })),
        { entry, preset: "incremental", config: null, findings: successor.incrementalDiagnostics },
        {
          entry,
          preset: null,
          config: JSON.stringify({ linter: { rules: { [corpus.rule]: "off" } } }) + "\n",
          findings: successor.offDiagnostics,
        },
        {
          entry,
          preset: null,
          config: JSON.stringify({ linter: projection.linter }) + "\n",
          findings: successor.projectionDiagnostics,
        },
      ];
    });
    scenarios.push({
      entry: original,
      preset: null,
      config:
        JSON.stringify({ linter: { preset: "incremental", rules: { [corpus.rule]: "error" } } }) +
        "\n",
      findings: original.diagnostics,
    });
    for (const { entry, preset, config, findings } of scenarios) {
      const bytes = fs.readFileSync(
        path.join(fixture, entry.sourceFile ?? entry.filename + ".fixture"),
      );
      assert.equal(bytes.toString("utf8"), entry.source);
      assert.equal(sha256(bytes), entry.sourceSha256);
      fs.writeFileSync(path.join(directory, entry.filename), bytes);
      const configSource = config ? vueConfigFixtureSource(config) : null;
      const configFile = config ? writeVueConfigFixture(directory, config) : null;
      const argv = [
        "lint",
        ...(configFile ? ["-c", path.basename(configFile)] : ["--no-config"]),
        ...(preset ? ["--preset", preset] : []),
        "--format",
        "json",
        "--locale",
        "en",
        "--help-level",
        "none",
        entry.filename,
      ];
      const result = spawnSync(path.join(root, build.binaryPath), argv, {
        cwd: directory,
        env: process.env,
        encoding: "utf8",
        timeout: 60000,
        maxBuffer: 4 * 1024 * 1024,
      });
      const run = {
        id: entry.id,
        argv,
        config,
        configSource,
        input: { text: bytes.toString("utf8"), sha256: sha256(bytes) },
        status: result.status,
        error: result.error ? { name: result.error.name, message: result.error.message } : null,
        signal: result.signal,
        stdout: result.stdout ?? "",
        stderr: result.stderr ?? "",
        expected: withoutHelp(wholeJson(entry, findings)),
        expectedStatus: findings.some((d) => d.severity === "error") ? 1 : 0,
      };
      evidence.runs.push(run);
      persist();
      assert.equal(run.error, null);
      assert.equal(run.signal, null);
      assert.equal(run.stderr, "");
      assert.equal(run.status, run.expectedStatus, entry.id);
      run.actual = JSON.parse(run.stdout);
      persist();
      assert.deepEqual(run.actual, run.expected, entry.id);
      if (configFile) {
        assert.equal(fs.readFileSync(configFile, "utf8"), configSource);
        fs.unlinkSync(configFile);
      }
    }
    assert.equal(evidence.runs.length, 169);
    assert.deepEqual(custody(), evidence.custody);
    persist();
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
