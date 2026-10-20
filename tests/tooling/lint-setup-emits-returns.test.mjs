import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import { expectedBuildIdentity, validateBuildReceipt } from "../differential/build-receipt.ts";
import { sha256, wholeJson, withoutHelp } from "./lint-default-correctness-reference.mjs";
import { vueConfigFixtureSource, writeVueConfigFixture } from "./support/vue-config-fixture.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
await test("setup emits validators preserve compiler ownership and complete CLI packets", () => {
  const fixture = path.join(root, "tests/_fixtures/differential/lint/setup-emits-6097");
  const corpus = JSON.parse(fs.readFileSync(path.join(fixture, "cases.json"), "utf8"));
  const projection = JSON.parse(
    fs.readFileSync(path.join(root, "tests/_fixtures/n8n-cli-adoption.json"), "utf8"),
  );
  assert.equal(corpus.cases.length, 51);
  assert.equal(Object.keys(projection.linter.rules).length, 51);
  for (const rule of corpus.rules) assert.equal(projection.linter.rules[rule], undefined);
  const artifact = path.join(root, "target/differential/lint-setup-emits.json");
  fs.mkdirSync(path.dirname(artifact), { recursive: true });
  const evidence = { schema: "vize.lint.setup-emits-cli", version: 1, corpus, runs: [] };
  const persist = () => fs.writeFileSync(artifact, JSON.stringify(evidence, null, 2) + "\n");
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-setup-emits-"));
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
        findings: entry.disabledDiagnostics,
        config:
          JSON.stringify({
            linter: { rules: Object.fromEntries(corpus.rules.map((rule) => [rule, "off"])) },
          }) + "\n",
      },
      {
        entry,
        preset: null,
        findings: entry.projectionDiagnostics,
        config: JSON.stringify({ linter: projection.linter }) + "\n",
      },
    ]);
    for (const { entry, preset, config, findings } of scenarios) {
      const file = path.join(directory, entry.filename);
      const bytes = fs.readFileSync(path.join(fixture, entry.filename + ".fixture"));
      assert.equal(bytes.toString("utf8"), entry.source);
      assert.equal(sha256(bytes), entry.sourceSha256);
      fs.writeFileSync(file, bytes);
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
        expected: withoutHelp(wholeJson(entry, findings)),
        expectedStatus: findings.some((finding) => finding.severity === "error") ? 1 : 0,
      };
      evidence.runs.push(observed);
      persist();
      observed.input = {
        text: fs.readFileSync(file, "utf8"),
        sha256: sha256(fs.readFileSync(file)),
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
      fs.unlinkSync(file);
    }
    assert.equal(evidence.runs.length, 357);
  } finally {
    persist();
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
