import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { expectedBuildIdentity, validateBuildReceipt } from "../differential/build-receipt.ts";
import { compoundCorpus, root, sha256 } from "./css-global-compound-reference.mjs";
import { fixture as universalFixture, universalCorpus } from "./css-global-universal-reference.mjs";

await test("source CLI preserves whole compound global ownership and refusal reports", () => {
  const { source, cases } = compoundCorpus();
  const universal = universalCorpus();
  const build = expectedBuildIdentity(root);
  const binary = path.join(root, build.binaryPath);
  const receipt = JSON.parse(fs.readFileSync(binary + ".differential-build.json", "utf8"));
  validateBuildReceipt(receipt, build);
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-css-global-compound-"));
  const artifact = path.join(root, "target/differential/css-global-compound-ownership.json");
  const evidence = { source, universalSource: universal.source, build, receipt, observations: [] };
  fs.mkdirSync(path.dirname(artifact), { recursive: true });
  const persist = () => fs.writeFileSync(artifact, JSON.stringify(evidence, null, 2) + "\n");
  const baseConfig = fs.readFileSync(
    path.join(root, "crates/vize_patina/tests/fixtures/issue-7976/7976/vize.config.json.fixture"),
    "utf8",
  );
  const nativeConfig = fs.readFileSync(path.join(universalFixture, "native.config.json"), "utf8");
  const run = (row, id, args, configText, expected) => {
    const input = fs.readFileSync(path.join(directory, row.filename));
    const before = { source: input.toString(), sourceSha256: sha256(input), config: configText };
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
    const observation = { id, before, expected, raw };
    evidence.observations.push(observation);
    persist();
    const after = {
      source: fs.readFileSync(path.join(directory, row.filename), "utf8"),
      config: fs.readFileSync(path.join(directory, "vize.config.json"), "utf8"),
    };
    observation.after = after;
    persist();
    assert.equal(raw.error, null, id);
    assert.equal(raw.signal, null, id);
    assert.equal(raw.status, 0, JSON.stringify(raw));
    assert.equal(raw.stderr, "", id);
    assert.equal(before.source, row.source, id);
    assert.deepEqual(after, { source: row.source, config: configText }, id);
    return raw;
  };
  try {
    fs.writeFileSync(path.join(directory, ".git"), "authored fixture workspace boundary\n");
    for (const [rows, config, prefix] of [
      [cases, baseConfig, ""],
      [universal.cases, nativeConfig, "universal-"],
    ]) {
      for (const row of rows) {
        const id = prefix + row.id;
        fs.writeFileSync(path.join(directory, row.filename), row.source);
        fs.writeFileSync(path.join(directory, "vize.config.json"), config);
        const args = ["lint", "-f", "json", "--locale", "en", "--help-level", "full", row.filename];
        for (let pass = 0; pass < 3; pass++) {
          const raw = run(row, `${id}-json-${pass}`, args, config, row.expectedCli);
          assert.deepEqual(JSON.parse(raw.stdout), row.expectedCli, row.id);
        }
        const plainArgs = [
          "lint",
          "-f",
          "plain",
          "--locale",
          "en",
          "--help-level",
          "full",
          row.filename,
        ];
        assert.equal(
          run(row, id + "-plain", plainArgs, config, row.expectedPlain).stdout,
          row.expectedPlain,
          row.id,
        );
        const offConfig = JSON.parse(config);
        offConfig.linter.rules["css/no-display-none"] = "off";
        const off = JSON.stringify(offConfig);
        fs.writeFileSync(path.join(directory, "vize.config.json"), off);
        const expected = [{ file: row.filename, messages: [], errorCount: 0, warningCount: 0 }];
        assert.deepEqual(
          JSON.parse(run(row, id + "-off", args, off, expected).stdout),
          expected,
          row.id,
        );
      }
    }
    assert.equal(evidence.observations.length, 310 + universal.source.cliObservationCount);
    assert.equal(new Set(evidence.observations.map((row) => row.id)).size, 525);
  } finally {
    try {
      persist();
    } finally {
      fs.rmSync(directory, { recursive: true, force: true });
    }
  }
});
