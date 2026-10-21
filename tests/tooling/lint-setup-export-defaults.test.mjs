import { writeVueConfigFixture } from "./support/vue-config-fixture.ts";
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import { expectedBuildIdentity, validateBuildReceipt } from "../differential/build-receipt.ts";
import { sha256 } from "./lint-default-correctness-reference.mjs";
import { withoutHelp } from "./support/lint-help-policy.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const fixture = path.join(root, "tests/_fixtures/differential/lint/setup-export-default-8090");
await test("setup export defaults preserve seventy whole source CLI packets", () => {
  const corpusBytes = fs.readFileSync(path.join(fixture, "cases.json"));
  const controlsBytes = fs.readFileSync(path.join(fixture, "default-route-controls.json"));
  const corpus = JSON.parse(corpusBytes);
  const controls = JSON.parse(controlsBytes);
  assert.equal(corpus.cases.length, 18);
  assert.equal(controls.cases.length, 4);
  assert.equal(controls.corpusSha256, sha256(corpusBytes));
  const scoped =
    JSON.stringify({
      linter: { preset: "incremental", rules: { [corpus.rule]: "error" } },
    }) + "\n";
  const vectors = corpus.cases.map((entry) => ({
    suite: "scoped",
    entry,
    route: {
      id: "scoped",
      config: scoped,
      argv: [
        "lint",
        "--config",
        "vite.config.mjs",
        "--format",
        "json",
        "--locale",
        "en",
        "--help-level",
        "none",
        entry.filename,
      ],
      afterCliPacket: entry.afterCliPacket,
      afterStatus: entry.status,
    },
  }));
  for (const entry of controls.cases) {
    assert.equal(entry.routes.length, 13);
    for (const route of entry.routes) vectors.push({ suite: "full-route", entry, route });
  }
  assert.equal(vectors.length, 70);
  const artifact = path.join(root, "target/differential/lint-setup-export-defaults.json");
  fs.mkdirSync(path.dirname(artifact), { recursive: true });
  const evidence = {
    schema: "vize.lint.setup-export-default-source-cli",
    version: 1,
    corpusSha256: sha256(corpusBytes),
    controlsSha256: sha256(controlsBytes),
    corpus,
    controls,
    runs: [],
  };
  const persist = () => fs.writeFileSync(artifact, JSON.stringify(evidence, null, 2) + "\n");
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-setup-export-default-"));
  try {
    const build = expectedBuildIdentity(root);
    const receipt = JSON.parse(
      fs.readFileSync(path.join(root, build.binaryPath + ".differential-build.json"), "utf8"),
    );
    evidence.build = { expected: build, receipt };
    persist();
    validateBuildReceipt(receipt, build);
    for (const { suite, entry, route } of vectors) {
      const source = fs.readFileSync(path.join(fixture, entry.sourceFile));
      assert.equal(source.toString("utf8"), entry.source);
      assert.equal(sha256(source), entry.sourceSha256);
      const filename = path.join(directory, entry.filename);
      fs.writeFileSync(filename, source);
      if (route.config !== null) writeVueConfigFixture(directory, route.config);
      const argv = route.argv.map((arg) => (arg === "vize.config.json" ? "vite.config.mjs" : arg));
      const run = spawnSync(path.join(root, build.binaryPath), argv, {
        cwd: directory,
        env: process.env,
        timeout: 60_000,
        maxBuffer: 4 * 1024 * 1024,
      });
      const after = fs.readFileSync(filename);
      const record = {
        id: entry.id,
        suite,
        route: route.id,
        source: entry.source,
        sourceSha256: entry.sourceSha256,
        argv,
        originalArgv: route.argv,
        config: route.config,
        expected: withoutHelp(route.afterCliPacket),
        expectedStatus: route.afterStatus,
        status: run.status,
        signal: run.signal,
        error: run.error ? { name: run.error.name, message: run.error.message } : null,
        stdoutBase64: run.stdout?.toString("base64") ?? "",
        stderrBase64: run.stderr?.toString("base64") ?? "",
        finalSource: after.toString("utf8"),
        finalSourceSha256: sha256(after),
      };
      evidence.runs.push(record);
      persist();
      assert.equal(record.error, null);
      assert.equal(record.signal, null);
      assert.equal(record.status, record.expectedStatus, entry.id + " / " + route.id);
      assert.equal(record.stderrBase64, "");
      assert.deepEqual(after, source);
      record.actual = JSON.parse(Buffer.from(record.stdoutBase64, "base64").toString("utf8"));
      persist();
      assert.deepEqual(record.actual, record.expected, entry.id + " / " + route.id);
    }
    assert.equal(evidence.runs.length, 70);
  } finally {
    persist();
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
