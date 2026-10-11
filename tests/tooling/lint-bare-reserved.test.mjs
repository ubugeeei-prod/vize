import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import { expectedBuildIdentity, validateBuildReceipt } from "../differential/build-receipt.ts";
import { sha256, wholeJson } from "./lint-default-correctness-reference.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
await test("bare instance names preserve complete CLI packets and existing prefix behavior", () => {
  const fixture = path.join(root, "tests/_fixtures/differential/lint/bare-reserved-8090");
  const corpusPath = path.join(fixture, "cases.json");
  const corpus = JSON.parse(fs.readFileSync(corpusPath, "utf8"));
  const companions = JSON.parse(
    fs.readFileSync(path.join(fixture, "default-route-companions.json"), "utf8"),
  );
  assert.equal(companions.cases.length, 16);
  const projectionPath = path.join(root, "tests/_fixtures/n8n-cli-adoption.json");
  const projection = JSON.parse(fs.readFileSync(projectionPath, "utf8"));
  const original = JSON.parse(
    fs.readFileSync(path.join(fixture, "original-wave-ui-accordion.json"), "utf8"),
  );
  original.source = fs.readFileSync(path.join(fixture, original.sourceFile), "utf8");
  assert.equal(corpus.schema, "vize.lint.bare-reserved");
  assert.equal(corpus.version, 1);
  assert.equal(corpus.cases.length, 16);
  assert.equal(corpus.removedBareNames.length, 14);
  assert.equal(
    corpus.cases.reduce((count, entry) => count + entry.beforeDiagnostics.length, 0),
    137,
  );
  assert.equal(
    corpus.cases.reduce((count, entry) => count + entry.diagnostics.length, 0),
    17,
  );
  assert.equal(Object.keys(projection.linter.rules).length, 51);
  assert.equal(projection.linter.rules[corpus.rule], undefined);

  const custodyPaths = [
    "tests/_fixtures/differential/lint/bare-reserved-8090/cases.json",
    "tests/_fixtures/differential/lint/bare-reserved-8090/default-route-companions.json",
    "tests/_fixtures/differential/lint/bare-reserved-8090/default-route-public-before.json",
    "tests/_fixtures/differential/lint/bare-reserved-8090/default-route-initial-failure.json",
    "tests/_fixtures/differential/lint/bare-reserved-8090/observe-before.mjs",
    "tests/_fixtures/differential/lint/bare-reserved-8090/primary-sources.json",
    "tests/_fixtures/differential/lint/bare-reserved-8090/original-public-before.json",
    "tests/_fixtures/differential/lint/bare-reserved-8090/official-provider.json",
    "tests/_fixtures/differential/lint/bare-reserved-8090/original-matrix-packet.json.gz",
    "tests/_fixtures/differential/lint/bare-reserved-8090/original-wave-ui-accordion.vue.fixture",
    "tests/_fixtures/differential/lint/bare-reserved-8090/original-wave-ui-accordion.json",
    "tests/_fixtures/differential/lint/bare-reserved-8090/historical-snapshot.json",
    "tests/_fixtures/differential/lint/bare-reserved-8090/historical-method-bare.snap.fixture",
    "tests/_fixtures/differential/lint/bare-reserved-8090/historical-method-bare.ts.fixture",
    "tests/_fixtures/n8n-cli-adoption.json",
    "tests/tooling/lint-default-correctness-reference.mjs",
    "tests/tooling/lint-bare-reserved.test.mjs",
    "tests/differential/build-receipt.ts",
  ];
  const custody = () =>
    custodyPaths.map((file) => ({
      path: file,
      sha256: sha256(fs.readFileSync(path.join(root, file))),
    }));
  const artifact = path.join(root, "target/differential/lint-bare-reserved.json");
  fs.mkdirSync(path.dirname(artifact), { recursive: true });
  const evidence = {
    schema: "vize.lint.bare-reserved-cli",
    version: 1,
    corpus,
    custody: custody(),
    runs: [],
  };
  const persist = () => fs.writeFileSync(artifact, JSON.stringify(evidence, null, 2) + "\n");
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-bare-reserved-"));
  try {
    const build = expectedBuildIdentity(root);
    const receipt = JSON.parse(
      fs.readFileSync(path.join(root, build.binaryPath + ".differential-build.json"), "utf8"),
    );
    evidence.build = { expected: build, receipt };
    persist();
    validateBuildReceipt(receipt, build);
    const scenarios = corpus.cases.flatMap((entry, index) => {
      const companion = companions.cases[index];
      assert.equal(companion.id, entry.id);
      assert.equal(companion.sourceSha256, entry.sourceSha256);
      return [
        ...[null, "recommended", "essential", "ecosystem"].map((preset) => ({
          entry,
          preset,
          config: null,
          findings: companion.combinedCliDiagnostics,
        })),
        { entry, preset: "incremental", config: null, findings: [] },
        {
          entry,
          preset: null,
          findings: companion.diagnostics,
          config: JSON.stringify({ linter: { rules: { [corpus.rule]: "off" } } }) + "\n",
        },
        {
          entry,
          preset: null,
          findings: [],
          config: JSON.stringify({ linter: projection.linter }) + "\n",
        },
      ];
    });
    scenarios.push({
      entry: original,
      preset: null,
      findings: original.diagnostics,
      config:
        JSON.stringify({
          linter: { preset: "incremental", rules: { [corpus.rule]: "error" } },
        }) + "\n",
    });
    for (const { entry, preset, config, findings } of scenarios) {
      const file = path.join(directory, entry.filename);
      const bytes = fs.readFileSync(
        path.join(fixture, entry.sourceFile ?? entry.filename + ".fixture"),
      );
      assert.equal(bytes.toString("utf8"), entry.source);
      assert.equal(sha256(bytes), entry.sourceSha256);
      fs.writeFileSync(file, bytes);
      if (config) fs.writeFileSync(path.join(directory, "vize.config.json"), config);
      const argv = [
        "lint",
        ...(config ? ["-c", "vize.config.json"] : ["--no-config"]),
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
        status: run.status,
        signal: run.signal,
        error: run.error ? { name: run.error.name, message: run.error.message } : null,
        stdout: run.stdout ?? "",
        stderr: run.stderr ?? "",
        expected: wholeJson(entry, findings),
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
      if (config) {
        assert.equal(fs.readFileSync(path.join(directory, "vize.config.json"), "utf8"), config);
        fs.unlinkSync(path.join(directory, "vize.config.json"));
      }
      fs.unlinkSync(file);
    }
    assert.equal(evidence.runs.filter((run) => run.id !== original.id).length, 112);
    assert.equal(evidence.runs.filter((run) => run.id === original.id).length, 1);
    assert.deepEqual(custody(), evidence.custody);
  } finally {
    persist();
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
