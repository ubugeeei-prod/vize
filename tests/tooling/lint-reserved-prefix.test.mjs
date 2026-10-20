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
await test("exact reserved membership and groups preserve complete CLI packets", () => {
  const fixture = path.join(root, "tests/_fixtures/differential/lint/reserved-prefix-8090");
  const corpus = JSON.parse(fs.readFileSync(path.join(fixture, "cases.json"), "utf8"));
  const originals = JSON.parse(fs.readFileSync(path.join(fixture, "originals.json"), "utf8"));
  const successorPath = "../reserved-suppression-8090/original-prefix-successors.json";
  const successors = JSON.parse(fs.readFileSync(path.join(fixture, successorPath), "utf8"));
  assert.equal(successors.cases.length, 21);
  for (const [index, entry] of originals.cases.entries()) {
    assert.equal(successors.cases[index].id, entry.id);
    assert.equal(successors.cases[index].sourceSha256, entry.sourceSha256);
  }
  const companions = JSON.parse(
    fs.readFileSync(path.join(fixture, "default-route-companions.json"), "utf8"),
  );
  const companionPackets = new Map(companions.cases.map((entry) => [entry.id, entry]));
  assert.equal(companions.cases.length, 20);
  assert.equal(
    companions.targetCorpus.sha256,
    sha256(fs.readFileSync(path.join(fixture, "cases.json"))),
  );
  assert.equal(
    companions.cases.reduce((count, entry) => count + entry.diagnostics.length, 0),
    59,
  );
  for (const entry of corpus.cases) {
    const companion = companionPackets.get(entry.id);
    assert.equal(companion.filename, entry.filename);
    assert.equal(companion.sourceSha256, entry.sourceSha256);
  }
  const sourceIndex = JSON.parse(
    fs.readFileSync(path.join(fixture, "original-sources.json"), "utf8"),
  );
  const projection = JSON.parse(
    fs.readFileSync(path.join(root, "tests/_fixtures/n8n-cli-adoption.json"), "utf8"),
  );
  assert.equal(corpus.schema, "vize.lint.reserved-prefix");
  assert.equal(corpus.version, 1);
  assert.equal(corpus.cases.length, 20);
  assert.equal(corpus.reservedDollarNames.length, 24);
  assert.equal(
    corpus.cases.reduce((count, entry) => count + entry.diagnostics.length, 0),
    152,
  );
  assert.equal(originals.cases.length, 21);
  assert.equal(
    originals.cases.reduce((count, entry) => count + entry.diagnostics.length, 0),
    1,
  );
  assert.equal(Object.keys(projection.linter.rules).length, 51);
  assert.equal(projection.linter.rules[corpus.rule], undefined);

  const custodyPaths = [
    "api-order-successors.json",
    "api-order-main-source-custody.json",
    "api-order-main-source-refusal.log.gz",
    "cases.json",
    "originals.json",
    successorPath,
    "original-sources.json",
    "bare-parent-successors.json",
    "default-route-companions.json",
    "primary-sources.json",
    "observe-before.mjs",
    "original-public-before.json",
    "official-provider.json",
    "provider-supplement.json",
    "initial-provider-expectations.json.gz",
    "initial-observer.mjs.fixture",
    "historical-computed-underscore.json",
    "historical-computed-underscore.snap.fixture",
    "historical-computed-underscore.ts.fixture",
    ...sourceIndex.matrices.map((entry) => entry.path),
    ...sourceIndex.sources.map((entry) => entry.sourceFile),
  ];
  const custody = () => [
    ...custodyPaths.map((file) => ({
      path: path.relative(root, path.join(fixture, file)),
      sha256: sha256(fs.readFileSync(path.join(fixture, file))),
    })),
    ...[
      "tests/tooling/lint-default-correctness-reference.mjs",
      "tests/tooling/lint-reserved-prefix.test.mjs",
      "tests/tooling/reserved-prefix-reference.test.mjs",
      "tests/differential/build-receipt.ts",
      "tests/_fixtures/n8n-cli-adoption.json",
    ].map((file) => ({ path: file, sha256: sha256(fs.readFileSync(path.join(root, file))) })),
  ];
  const artifact = path.join(root, "target/differential/lint-reserved-prefix.json");
  fs.mkdirSync(path.dirname(artifact), { recursive: true });
  const evidence = {
    schema: "vize.lint.reserved-prefix-cli",
    version: 1,
    corpus,
    originals,
    companions,
    custody: custody(),
    runs: [],
  };
  const persist = () => fs.writeFileSync(artifact, JSON.stringify(evidence, null, 2) + "\n");
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-reserved-prefix-"));
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
        findings: companionPackets.get(entry.id).combinedCliDiagnostics,
      })),
      { entry, preset: "incremental", config: null, findings: [] },
      {
        entry,
        preset: null,
        findings: companionPackets.get(entry.id).diagnostics,
        config: JSON.stringify({ linter: { rules: { [corpus.rule]: "off" } } }) + "\n",
      },
      {
        entry,
        preset: null,
        findings: [],
        config: JSON.stringify({ linter: projection.linter }) + "\n",
      },
    ]);
    scenarios.push(
      ...originals.cases.map((entry, index) => ({
        entry,
        preset: null,
        findings: successors.cases[index].diagnostics,
        config:
          JSON.stringify({ linter: { preset: "incremental", rules: { [corpus.rule]: "error" } } }) +
          "\n",
      })),
    );
    for (const { entry, preset, config, findings } of scenarios) {
      const file = path.join(directory, entry.filename);
      const bytes = fs.readFileSync(
        path.join(fixture, entry.sourceFile ?? entry.filename + ".fixture"),
      );
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
    assert.equal(evidence.runs.length, 161);
    assert.deepEqual(custody(), evidence.custody);
  } finally {
    persist();
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
