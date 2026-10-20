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
const fixture = path.join(root, "tests/_fixtures/differential/lint/props-deep-destructure-8090");
const rule = "script/no-deep-destructure-in-props";
await test("owned prop destructuring preserves thirty six whole source CLI packets", async () => {
  const bytes = fs.readFileSync(path.join(fixture, "cases.json"));
  const corpus = JSON.parse(bytes);
  assert.equal(sha256(bytes), "d522ae6c3fc8a04403c43661c7d66584188a2168b1e01947093044f4239de8c8");
  assert.equal(corpus.cases.length, 33);
  const vectors = corpus.cases.map((entry) => ({
    entry,
    route: "scoped",
    config: JSON.stringify({ linter: entry.config }, null, 2) + "\n",
    expected: withoutHelp(entry.afterCli),
    expectedStatus: entry.afterStatus,
  }));
  for (const id of ["vue35-nested-object", "vue35-nested-array", "standalone-local-function"]) {
    const entry = corpus.cases.find((item) => item.id === id);
    assert(entry);
    vectors.push({
      entry,
      route: "explicit-off",
      config:
        JSON.stringify({ linter: { preset: "incremental", rules: { [rule]: "off" } } }) + "\n",
      expected: [{ file: entry.filename, messages: [], errorCount: 0, warningCount: 0 }],
      expectedStatus: 0,
    });
  }
  const artifact = path.join(root, "target/differential/lint-props-deep-destructure.json");
  fs.mkdirSync(path.dirname(artifact), { recursive: true });
  const evidence = {
    schema: "vize.lint.props-deep-destructure-source-cli",
    version: 1,
    corpusSha256: sha256(bytes),
    corpus,
    runs: [],
  };
  const persist = () => fs.writeFileSync(artifact, JSON.stringify(evidence, null, 2) + "\n");
  persist();
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-props-deep-destructure-"));
  try {
    const build = expectedBuildIdentity(root);
    const receipt = JSON.parse(
      fs.readFileSync(path.join(root, build.binaryPath + ".differential-build.json"), "utf8"),
    );
    evidence.build = { expected: build, receipt };
    persist();
    validateBuildReceipt(receipt, build);
    for (const { entry, route, config, expected, expectedStatus } of vectors) {
      const source = fs.readFileSync(path.join(fixture, entry.sourceFile));
      assert.equal(source.toString("utf8"), entry.source);
      assert.equal(sha256(source), entry.sourceSha256);
      const filename = path.join(directory, entry.filename);
      fs.writeFileSync(filename, source);
      writeVueConfigFixture(directory, config);
      const argv = [
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
      ];
      const run = spawnSync(path.join(root, build.binaryPath), argv, {
        cwd: directory,
        env: process.env,
        timeout: 60_000,
        maxBuffer: 4 * 1024 * 1024,
      });
      const after = fs.readFileSync(filename);
      const record = {
        id: entry.id,
        route,
        source: entry.source,
        sourceSha256: entry.sourceSha256,
        argv,
        config,
        configSha256: sha256(Buffer.from(config)),
        expected,
        expectedStatus,
        status: run.status,
        signal: run.signal,
        error: run.error ? { name: run.error.name, message: run.error.message } : null,
        stdoutBase64: run.stdout?.toString("base64") ?? "",
        stderrBase64: run.stderr?.toString("base64") ?? "",
        stdoutSha256: sha256(run.stdout ?? Buffer.alloc(0)),
        stderrSha256: sha256(run.stderr ?? Buffer.alloc(0)),
        finalSource: after.toString("utf8"),
        finalSourceSha256: sha256(after),
      };
      evidence.runs.push(record);
      persist();
      assert.equal(record.error, null);
      assert.equal(record.signal, null);
      assert.equal(record.status, expectedStatus, entry.id + " / " + route);
      assert.equal(record.stderrBase64, "");
      assert.deepEqual(after, source);
      record.actual = JSON.parse(Buffer.from(record.stdoutBase64, "base64").toString("utf8"));
      persist();
      assert.deepEqual(record.actual, expected, entry.id + " / " + route);
    }
    assert.equal(evidence.runs.length, 36);
  } finally {
    persist();
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
