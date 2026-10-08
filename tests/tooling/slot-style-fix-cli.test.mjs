import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { expectedBuildIdentity, validateBuildReceipt } from "../differential/build-receipt.ts";
import { fixedSource, loadCases, RULE, sha256, wholeJson } from "./slot-style-fix-reference.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

await test("source CLI retains complete original slot findings, fixed files and repeated-pass outputs", () => {
  const artifact = path.join(root, "target/differential/slot-style-fix-cli.json");
  fs.mkdirSync(path.dirname(artifact), { recursive: true });
  const evidence = { schema: "vize.lint.slot-style-fix", version: 1, runs: [] };
  const persist = () => fs.writeFileSync(artifact, JSON.stringify(evidence, null, 2) + "\n");
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-slot-fix-"));
  try {
    const build = expectedBuildIdentity(root);
    const receipt = JSON.parse(
      fs.readFileSync(path.join(root, build.binaryPath + ".differential-build.json"), "utf8"),
    );
    evidence.build = { receipt, expected: build };
    persist();
    validateBuildReceipt(receipt, build);
    const cli = path.join(root, build.binaryPath);
    const loaded = loadCases(root);
    evidence.fixture = loaded.manifest;
    evidence.originals = loaded.originals;
    persist();
    fs.writeFileSync(
      path.join(directory, "MyCard.vue"),
      loaded.originals.find((input) => input.file.endsWith("/MyCard.vue.fixture")).text,
    );
    const scenarios = [
      { ...loaded.cases[0], originalRules: true },
      ...loaded.cases.map((entry) => ({ ...entry, originalRules: false })),
    ];
    for (const entry of scenarios) {
      const config = entry.originalRules
        ? loaded.originals.find((input) => input.file.endsWith("/vize.config.json")).text
        : JSON.stringify({ linter: { preset: "incremental", rules: { [RULE]: "warn" } } }) + "\n";
      fs.writeFileSync(path.join(directory, "vize.config.json"), config);
      fs.writeFileSync(path.join(directory, entry.filename), entry.source);
      for (let pass = 0; pass < 4; pass++) {
        const input = fs.readFileSync(path.join(directory, entry.filename));
        const args = [
          "lint",
          "-c",
          "vize.config.json",
          "--format",
          "json",
          "--locale",
          "en",
          "--help-level",
          "none",
          ...(pass === 1 || pass === 3 ? ["--fix"] : []),
          entry.filename,
        ];
        const run = spawnSync(cli, args, {
          cwd: directory,
          env: process.env,
          encoding: "utf8",
          timeout: 60_000,
          maxBuffer: 4 * 1024 * 1024,
        });
        const expected = wholeJson(entry, pass > 0, entry.originalRules);
        const observation = {
          id: entry.id,
          originalRules: entry.originalRules,
          pass,
          argv: args,
          input: { text: input.toString("utf8"), bytes: input.length, sha256: sha256(input) },
          config: { text: config, bytes: Buffer.byteLength(config), sha256: sha256(config) },
          status: run.status,
          signal: run.signal,
          error: run.error ? { name: run.error.name, message: run.error.message } : null,
          stdout: run.stdout ?? "",
          stderr: run.stderr ?? "",
          expected,
          expectedStderr: "",
        };
        evidence.runs.push(observation);
        persist();
        const output = fs.readFileSync(path.join(directory, entry.filename));
        observation.outputFile = {
          text: output.toString("utf8"),
          bytes: output.length,
          sha256: sha256(output),
        };
        persist();
        assert.equal(observation.error, null, JSON.stringify(observation));
        assert.equal(observation.signal, null, JSON.stringify(observation));
        assert.equal(observation.status, 0, JSON.stringify(observation));
        assert.equal(observation.stderr, observation.expectedStderr, JSON.stringify(observation));
        assert.equal(
          observation.outputFile.text,
          pass ? fixedSource(entry, entry.originalRules) : entry.source,
          entry.id,
        );
        observation.actual = JSON.parse(observation.stdout);
        persist();
        assert.deepEqual(observation.actual, expected);
        assert.equal(fs.readFileSync(path.join(directory, "vize.config.json"), "utf8"), config);
      }
      fs.unlinkSync(path.join(directory, entry.filename));
    }
    assert.equal(evidence.runs.length, 120);
    console.log(
      "VIZE_SLOT_STYLE_FIX",
      JSON.stringify({
        sourceRevision: build.sourceRevision,
        binarySha256: build.binarySha256,
        runs: evidence.runs.length,
      }),
    );
  } finally {
    try {
      persist();
    } finally {
      fs.rmSync(directory, { recursive: true, force: true });
    }
  }
});
