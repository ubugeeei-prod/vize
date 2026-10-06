import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { expectedBuildIdentity, validateBuildReceipt } from "../differential/build-receipt.mjs";
import { loadCorpus, sha256, wholeJson } from "./lint-default-correctness-reference.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
await test("configless CLI defaults retain whole correctness findings and explicit controls", () => {
  const artifact = path.join(root, "target/differential/lint-default-correctness-cli.json");
  fs.mkdirSync(path.dirname(artifact), { recursive: true });
  const evidence = { schema: "vize.lint.default-correctness-cli", version: 1, runs: [] };
  const persist = () => fs.writeFileSync(artifact, JSON.stringify(evidence, null, 2) + "\n");
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-lint-default-"));
  try {
    const build = expectedBuildIdentity(root);
    const receipt = JSON.parse(
      fs.readFileSync(path.join(root, build.binaryPath + ".differential-build.json"), "utf8"),
    );
    evidence.build = { expected: build, receipt };
    persist();
    validateBuildReceipt(receipt, build);
    const corpus = loadCorpus(root);
    evidence.corpus = corpus;
    const scenarios = corpus.cases
      .filter((entry) => entry.kind === "vue")
      .flatMap((entry) => [
        { entry, preset: null, config: null, findings: entry.diagnostics },
        { entry, preset: "recommended", config: null, findings: entry.diagnostics },
        { entry, preset: "ecosystem", config: null, findings: entry.diagnostics },
        { entry, preset: "incremental", config: null, findings: [] },
      ]);
    for (const entry of corpus.cases.slice(0, 6)) {
      scenarios.push({
        entry,
        preset: null,
        findings: entry.baselineDiagnostics,
        config:
          JSON.stringify({
            linter: {
              rules: Object.fromEntries(corpus.promotedRules.map((name) => [name, "off"])),
            },
          }) + "\n",
      });
    }
    for (const version of corpus.configuredVueVersions) {
      for (const entry of corpus.cases.slice(0, 6)) {
        scenarios.push({
          entry,
          preset: null,
          config: JSON.stringify({ vue: { version } }) + "\n",
          findings: version === "3" ? entry.diagnostics : entry.baselineDiagnostics,
        });
      }
    }
    for (const { entry, preset, config, findings } of scenarios) {
      const file = path.join(directory, entry.filename);
      fs.writeFileSync(file, entry.source);
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
      const observation = {
        id: entry.id,
        argv,
        config,
        status: run.status,
        signal: run.signal,
        error: run.error ? { name: run.error.name, message: run.error.message } : null,
        stdout: run.stdout ?? "",
        stderr: run.stderr ?? "",
        expected: wholeJson(entry, findings),
        expectedStderr: "",
        expectedStatus: findings.some((finding) => finding.severity === "error") ? 1 : 0,
      };
      evidence.runs.push(observation);
      persist();
      const bytes = fs.readFileSync(file);
      observation.input = {
        text: bytes.toString("utf8"),
        bytes: bytes.length,
        sha256: sha256(bytes),
      };
      persist();
      assert.equal(observation.error, null, JSON.stringify(observation));
      assert.equal(observation.signal, null, JSON.stringify(observation));
      assert.equal(observation.status, observation.expectedStatus, JSON.stringify(observation));
      assert.equal(observation.stderr, "", JSON.stringify(observation));
      assert.equal(observation.input.text, entry.source);
      observation.actual = JSON.parse(observation.stdout);
      persist();
      assert.deepEqual(observation.actual, observation.expected);
      if (config) {
        assert.equal(fs.readFileSync(path.join(directory, "vize.config.json"), "utf8"), config);
        fs.unlinkSync(path.join(directory, "vize.config.json"));
      }
      fs.unlinkSync(file);
    }
    assert.equal(evidence.runs.length, 80);
    console.log(
      "VIZE_DEFAULT_CORRECTNESS",
      JSON.stringify({ ...build, runs: evidence.runs.length }),
    );
  } finally {
    try {
      persist();
    } finally {
      fs.rmSync(directory, { recursive: true, force: true });
    }
  }
});
