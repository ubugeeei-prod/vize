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
await test("public Vue imports preserve whole CLI packets and safe final file bytes", () => {
  const fixture = path.join(root, "tests/_fixtures/differential/lint/public-vue-imports-8090");
  const corpusBytes = fs.readFileSync(path.join(fixture, "cases.json"));
  const corpus = JSON.parse(corpusBytes);
  const controlsBytes = fs.readFileSync(path.join(fixture, "default-route-controls.json"));
  const controls = JSON.parse(controlsBytes);
  const projectionBytes = fs.readFileSync(path.join(root, "tests/_fixtures/n8n-cli-adoption.json"));
  const projection = JSON.parse(projectionBytes);
  const primaryBytes = fs.readFileSync(path.join(fixture, "primary-sources.json"));
  const primary = JSON.parse(primaryBytes);
  const referenceCustody = [
    primary.corpus,
    primary.routes,
    primary.providerEnvelope,
    primary.initialReference,
    primary.releasedFixContract,
    ...primary.references,
  ].map((entry) => {
    const bytes = fs.readFileSync(path.join(fixture, entry.path));
    assert.equal(sha256(bytes), entry.sha256);
    assert.equal(bytes.length, entry.bytes);
    return entry;
  });
  assert.equal(corpus.cases.length, 29);
  assert.equal(controls.cases.length, 4);
  assert.equal(controls.corpusSha256, sha256(corpusBytes));
  assert.equal(controls.n8nProjectionSha256, sha256(projectionBytes));
  assert.equal(Object.keys(projection.linter.rules).length, 51);
  assert.equal(projection.linter.rules[corpus.rule], undefined);
  const artifact = path.join(root, "target/differential/lint-public-vue-imports.json");
  fs.mkdirSync(path.dirname(artifact), { recursive: true });
  const evidence = {
    schema: "vize.lint.public-vue-imports-cli",
    version: 1,
    corpusSha256: sha256(corpusBytes),
    controlsSha256: sha256(controlsBytes),
    corpus,
    controls,
    referenceCustody: { primarySourcesSha256: sha256(primaryBytes), files: referenceCustody },
    runs: [],
  };
  const persist = () => fs.writeFileSync(artifact, JSON.stringify(evidence, null, 2) + "\n");
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-public-vue-imports-"));
  try {
    const build = expectedBuildIdentity(root);
    const receipt = JSON.parse(
      fs.readFileSync(path.join(root, build.binaryPath + ".differential-build.json"), "utf8"),
    );
    evidence.build = { expected: build, receipt };
    persist();
    validateBuildReceipt(receipt, build);
    const scoped =
      JSON.stringify({
        linter: { preset: "incremental", rules: { [corpus.rule]: "warn" } },
      }) + "\n";
    const scenarios = corpus.cases.map((entry) => ({
      entry,
      kind: "scoped",
      preset: null,
      config: scoped,
      fix: false,
      findings: entry.diagnostics,
      finalSource: entry.source,
    }));
    for (const control of controls.cases) {
      const entry = corpus.cases.find((entry) => entry.id === control.id);
      assert.equal(entry.sourceSha256, control.sourceSha256);
      for (const route of control.routes) {
        const config =
          route.id === "explicit-off"
            ? JSON.stringify({ linter: { rules: { [corpus.rule]: "off" } } }) + "\n"
            : route.id === "n8n-projection"
              ? JSON.stringify({ linter: projection.linter }) + "\n"
              : null;
        scenarios.push({
          entry,
          kind: route.id,
          preset: ["default", "explicit-off", "n8n-projection"].includes(route.id)
            ? null
            : route.id,
          config,
          fix: false,
          findings: route.diagnostics,
          finalSource: entry.source,
        });
      }
    }
    for (const id of [
      "original-element-plus",
      "public-ref",
      "mixed-named",
      "namespace",
      "default-import",
      "private-alias",
    ]) {
      const entry = corpus.cases.find((entry) => entry.id === id);
      scenarios.push({
        entry,
        kind: "actual-fix",
        preset: null,
        config: scoped,
        fix: true,
        findings: entry.fixedDiagnostics,
        finalSource: entry.fixedSource,
      });
    }
    for (const { entry, kind, preset, config, fix, findings, finalSource } of scenarios) {
      const filename = path.join(directory, entry.filename);
      const bytes = fs.readFileSync(path.join(fixture, entry.sourceFile));
      assert.equal(bytes.toString("utf8"), entry.source);
      assert.equal(sha256(bytes), entry.sourceSha256);
      fs.writeFileSync(filename, bytes);
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
        ...(fix ? ["--fix"] : []),
        entry.filename,
      ];
      const run = spawnSync(path.join(root, build.binaryPath), argv, {
        cwd: directory,
        env: process.env,
        timeout: 60_000,
        maxBuffer: 8 * 1024 * 1024,
      });
      const after = fs.readFileSync(filename);
      const observed = {
        id: entry.id,
        kind,
        argv,
        config,
        configSource,
        input: { source: entry.source, sha256: sha256(bytes) },
        expected: wholeJson({ ...entry, source: finalSource }, findings),
        expectedStatus: findings.some((finding) => finding.severity === "error") ? 1 : 0,
        expectedFinalSource: finalSource,
        status: run.status,
        signal: run.signal,
        error: run.error ? { name: run.error.name, message: run.error.message } : null,
        stdout: run.stdout?.toString("utf8") ?? "",
        stdoutBase64: run.stdout?.toString("base64") ?? "",
        stderr: run.stderr?.toString("utf8") ?? "",
        stderrBase64: run.stderr?.toString("base64") ?? "",
        finalSource: after.toString("utf8"),
        finalSourceSha256: sha256(after),
      };
      evidence.runs.push(observed);
      persist();
      assert.equal(observed.error, null);
      assert.equal(observed.signal, null);
      assert.equal(observed.status, observed.expectedStatus, entry.id + " / " + kind);
      assert.equal(observed.stderrBase64, "");
      observed.actual = JSON.parse(observed.stdout);
      persist();
      assert.deepEqual(observed.actual, observed.expected, entry.id + " / " + kind);
      assert.equal(observed.finalSource, observed.expectedFinalSource, entry.id + " / " + kind);
      assert.equal(observed.finalSourceSha256, sha256(finalSource));
      if (configFile) {
        assert.equal(fs.readFileSync(configFile, "utf8"), configSource);
        fs.unlinkSync(configFile);
      }
      fs.unlinkSync(filename);
    }
    assert.equal(evidence.runs.length, 63);
    assert.equal(sha256(fs.readFileSync(path.join(fixture, "cases.json"))), evidence.corpusSha256);
    assert.equal(
      sha256(fs.readFileSync(path.join(fixture, "default-route-controls.json"))),
      evidence.controlsSha256,
    );
  } finally {
    persist();
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
