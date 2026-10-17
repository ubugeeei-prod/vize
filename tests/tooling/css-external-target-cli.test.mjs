import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { expectedBuildIdentity, validateBuildReceipt } from "../differential/build-receipt.ts";
import { fixture, frozenCorpus, root, sha256 } from "./css-external-target-reference.mjs";
import { vueConfigFixtureSource, writeVueConfigFixture } from "./support/vue-config-fixture.ts";

await test("source CLI preserves both CSS originals and complete subject-owned reports", () => {
  const { source, cases } = frozenCorpus();
  const build = expectedBuildIdentity(root);
  const binary = path.join(root, build.binaryPath);
  const receipt = JSON.parse(fs.readFileSync(binary + ".differential-build.json", "utf8"));
  validateBuildReceipt(receipt, build);
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-css-targets-"));
  const artifact = path.join(root, "target/differential/css-external-targets.json");
  const evidence = { source, build, receipt, observations: [] };
  // Keep the reporter's configuration bytes as evidence outside discovery.
  const originalConfig = path.join(directory, "authored-vue-settings.json");
  const inputPath = (row) =>
    row.path === "vize.config.json" ? originalConfig : path.join(directory, row.path);
  fs.mkdirSync(path.dirname(artifact), { recursive: true });
  const persist = () => fs.writeFileSync(artifact, JSON.stringify(evidence, null, 2) + "\n");
  const run = (id, args, config, expected) => {
    const actual = spawnSync(binary, args, {
      cwd: directory,
      env: process.env,
      timeout: 30_000,
      maxBuffer: 2 * 1024 * 1024,
      encoding: "utf8",
    });
    const raw = {
      args,
      status: actual.status,
      signal: actual.signal,
      error: actual.error?.message ?? null,
      stdout: actual.stdout ?? "",
      stderr: actual.stderr ?? "",
    };
    const carrierBytes = fs.readFileSync(path.join(directory, "vite.config.mjs"));
    evidence.observations.push({
      id,
      config,
      carrier: {
        path: "vite.config.mjs",
        source: carrierBytes.toString("utf8"),
        sha256: sha256(carrierBytes),
      },
      expected,
      raw,
    });
    persist();
    assert.equal(raw.error, null, id);
    assert.equal(raw.signal, null, id);
    assert.equal(raw.status, 0, JSON.stringify(raw));
    assert.equal(raw.stderr, "", id);
    return raw;
  };
  try {
    fs.writeFileSync(path.join(directory, ".git"), "authored fixture workspace boundary\n");
    for (const issue of source.issues) {
      const inputs = source.inputs.filter((row) => row.issue === issue.number);
      for (const row of inputs) fs.copyFileSync(path.join(fixture, row.file), inputPath(row));
      const config = fs.readFileSync(originalConfig, "utf8");
      const carrier = writeVueConfigFixture(directory, config);
      const original = cases.find((row) => row.id === `original-${issue.number}`);
      const raw = run(
        original.id + "-literal-plain",
        ["lint", "-f", "plain", "--help-level", "none", original.filename],
        config,
        "Patina lint report: No problems found in 1 file(s)\n",
      );
      assert.equal(raw.stdout, "Patina lint report: No problems found in 1 file(s)\n");
      for (const row of inputs) assert.equal(sha256(fs.readFileSync(inputPath(row))), row.sha256);
      assert.equal(fs.readFileSync(carrier, "utf8"), vueConfigFixtureSource(config));
    }
    const config = fs.readFileSync(path.join(fixture, "7976/vize.config.json.fixture"), "utf8");
    for (const row of cases) {
      fs.writeFileSync(path.join(directory, row.filename), row.source);
      fs.writeFileSync(originalConfig, config);
      const carrier = writeVueConfigFixture(directory, config);
      const args = ["lint", "-f", "json", "--locale", "en", "--help-level", "full", row.filename];
      for (let pass = 0; pass < 3; pass++) {
        const raw = run(`${row.id}-pass-${pass}`, args, config, row.expectedCli);
        assert.deepEqual(JSON.parse(raw.stdout), row.expectedCli, row.id);
        assert.equal(fs.readFileSync(path.join(directory, row.filename), "utf8"), row.source);
        assert.equal(fs.readFileSync(originalConfig, "utf8"), config);
        assert.equal(fs.readFileSync(carrier, "utf8"), vueConfigFixtureSource(config));
      }
      const off = { linter: { preset: "incremental", rules: { "css/no-display-none": "off" } } };
      writeVueConfigFixture(directory, off);
      const expected = [{ file: row.filename, messages: [], errorCount: 0, warningCount: 0 }];
      const raw = run(row.id + "-off", args, off, expected);
      assert.deepEqual(JSON.parse(raw.stdout), expected, row.id);
      assert.equal(fs.readFileSync(path.join(directory, row.filename), "utf8"), row.source);
      assert.equal(fs.readFileSync(carrier, "utf8"), vueConfigFixtureSource(off));
      assert.equal(fs.readFileSync(originalConfig, "utf8"), config);
    }
  } finally {
    try {
      persist();
    } finally {
      fs.rmSync(directory, { recursive: true, force: true });
    }
  }
});
