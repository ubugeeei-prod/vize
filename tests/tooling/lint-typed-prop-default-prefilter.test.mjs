import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import { expectedBuildIdentity, validateBuildReceipt } from "../differential/build-receipt.ts";
import { sha256, wholeJson } from "./lint-default-correctness-reference.mjs";
import { vueConfigFixtureSource, writeVueConfigFixture } from "./support/vue-config-fixture.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
await test("typed prop defaults reach the whole default CLI packet without policy leakage", () => {
  const fixture = path.join(
    root,
    "tests/_fixtures/differential/lint/typed-prop-default-prefilter-8564",
  );
  const corpus = JSON.parse(
    fs.readFileSync(path.join(fixture, "current-range-cases.json"), "utf8"),
  );
  const projection = JSON.parse(
    fs.readFileSync(path.join(root, "tests/_fixtures/n8n-cli-adoption.json"), "utf8"),
  );
  assert.equal(corpus.cases.length, 8);
  assert.deepEqual(
    corpus.cases.filter((entry) => entry.diagnostics.length > 0).map((entry) => entry.id),
    ["original-vaul", "with-defaults", "destructure-alias", "unicode-crlf"],
    "fixture must retain the four authored positive cases",
  );
  assert.equal(Object.keys(projection.linter.rules).length, 51);
  assert.equal(projection.linter.rules[corpus.rule], undefined);
  const artifact = path.join(root, "target/differential/lint-typed-prop-default-prefilter.json");
  fs.mkdirSync(path.dirname(artifact), { recursive: true });
  const evidence = {
    schema: "vize.lint.typed-prop-default-prefilter-cli",
    version: 1,
    corpus,
    runs: [],
  };
  const persist = () => fs.writeFileSync(artifact, JSON.stringify(evidence, null, 2) + "\n");
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-typed-prop-default-"));
  try {
    const build = expectedBuildIdentity(root);
    const receipt = JSON.parse(
      fs.readFileSync(path.join(root, build.binaryPath + ".differential-build.json"), "utf8"),
    );
    evidence.build = { expected: build, receipt };
    persist();
    validateBuildReceipt(receipt, build);
    const scenarios = corpus.cases.flatMap((entry) => [
      ...[null, "recommended", "essential", "ecosystem"].map((preset) => ({
        entry,
        preset,
        config: null,
        findings: entry.diagnostics,
      })),
      { entry, preset: "incremental", config: null, findings: [] },
      {
        entry,
        preset: null,
        config: JSON.stringify({ linter: { rules: { [corpus.rule]: "off" } } }) + "\n",
        findings: [],
      },
      {
        entry,
        preset: null,
        config: JSON.stringify({ linter: projection.linter }) + "\n",
        findings: [],
      },
    ]);
    for (const { entry, preset, config, findings } of scenarios) {
      const bytes = fs.readFileSync(path.join(fixture, entry.filename + ".fixture"));
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
      const run = spawnSync(path.join(root, build.binaryPath), argv, {
        cwd: directory,
        env: process.env,
        encoding: "utf8",
        timeout: 60_000,
        maxBuffer: 4 * 1024 * 1024,
      });
      const observed = {
        id: entry.id,
        argv,
        config,
        configSource,
        status: run.status,
        signal: run.signal,
        error: run.error ? { name: run.error.name, message: run.error.message } : null,
        stdout: run.stdout ?? "",
        stderr: run.stderr ?? "",
        expected: wholeJson(entry, findings),
        expectedStatus: findings.length ? 1 : 0,
      };
      evidence.runs.push(observed);
      persist();
      observed.input = {
        text: fs.readFileSync(path.join(directory, entry.filename), "utf8"),
        sha256: sha256(fs.readFileSync(path.join(directory, entry.filename))),
      };
      persist();
      assert.equal(observed.error, null);
      assert.equal(observed.signal, null);
      assert.equal(observed.status, observed.expectedStatus, JSON.stringify(observed));
      assert.equal(observed.stderr, "");
      observed.actual = JSON.parse(observed.stdout);
      persist();
      assert.deepEqual(observed.actual, observed.expected, entry.id);
      assert.equal(observed.input.text, entry.source);
      assert.equal(observed.input.sha256, entry.sourceSha256);
      if (configFile) {
        assert.equal(fs.readFileSync(configFile, "utf8"), configSource);
        fs.unlinkSync(configFile);
      }
      fs.unlinkSync(path.join(directory, entry.filename));
    }
    assert.equal(evidence.runs.length, 56);
  } finally {
    persist();
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
