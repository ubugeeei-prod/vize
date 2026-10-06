import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { expectedBuildIdentity, validateBuildReceipt } from "../differential/build-receipt.mjs";
import { fixture, frozenCorpus, root, sha256 } from "./css-external-target-reference.mjs";

await test("source CLI preserves both CSS originals and complete subject-owned reports", () => {
  const { source, cases } = frozenCorpus();
  const build = expectedBuildIdentity(root);
  const binary = path.join(root, build.binaryPath);
  const receipt = JSON.parse(fs.readFileSync(binary + ".differential-build.json", "utf8"));
  validateBuildReceipt(receipt, build);
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-css-targets-"));
  const artifact = path.join(root, "target/differential/css-external-targets.json");
  const evidence = { source, build, receipt, observations: [] };
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
    evidence.observations.push({ id, config, expected, raw });
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
      for (const row of inputs)
        fs.copyFileSync(path.join(fixture, row.file), path.join(directory, row.path));
      const config = fs.readFileSync(path.join(directory, "vize.config.json"), "utf8");
      const original = cases.find((row) => row.id === `original-${issue.number}`);
      const raw = run(
        original.id + "-literal-plain",
        ["lint", "-f", "plain", "--help-level", "none", original.filename],
        config,
        "Patina lint report: No problems found in 1 file(s)\n",
      );
      assert.equal(raw.stdout, "Patina lint report: No problems found in 1 file(s)\n");
      for (const row of inputs)
        assert.equal(sha256(fs.readFileSync(path.join(directory, row.path))), row.sha256);
    }
    const config = fs.readFileSync(path.join(fixture, "7976/vize.config.json.fixture"), "utf8");
    for (const row of cases) {
      fs.writeFileSync(path.join(directory, row.filename), row.source);
      fs.writeFileSync(path.join(directory, "vize.config.json"), config);
      const args = ["lint", "-f", "json", "--locale", "en", "--help-level", "full", row.filename];
      for (let pass = 0; pass < 3; pass++) {
        const raw = run(`${row.id}-pass-${pass}`, args, config, row.expectedCli);
        assert.deepEqual(JSON.parse(raw.stdout), row.expectedCli, row.id);
        assert.equal(fs.readFileSync(path.join(directory, row.filename), "utf8"), row.source);
        assert.equal(fs.readFileSync(path.join(directory, "vize.config.json"), "utf8"), config);
      }
      const off = { linter: { preset: "incremental", rules: { "css/no-display-none": "off" } } };
      fs.writeFileSync(path.join(directory, "vize.config.json"), JSON.stringify(off));
      const expected = [{ file: row.filename, messages: [], errorCount: 0, warningCount: 0 }];
      const raw = run(row.id + "-off", args, off, expected);
      assert.deepEqual(JSON.parse(raw.stdout), expected, row.id);
      assert.equal(fs.readFileSync(path.join(directory, row.filename), "utf8"), row.source);
      assert.equal(
        fs.readFileSync(path.join(directory, "vize.config.json"), "utf8"),
        JSON.stringify(off),
      );
    }
  } finally {
    try {
      persist();
    } finally {
      fs.rmSync(directory, { recursive: true, force: true });
    }
  }
});
