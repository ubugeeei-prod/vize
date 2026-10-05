import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { expectedBuildIdentity, validateBuildReceipt } from "../differential/build-receipt.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const fixture = path.join(root, "crates/vize_patina/tests/fixtures/css-id-selector-ranges");
const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");

await test("source-built CLI locates every original and authored CSS ID selector", () => {
  const manifest = JSON.parse(fs.readFileSync(path.join(fixture, "cases.json"), "utf8"));
  assert.equal(manifest.issue, 7981);
  assert.equal(manifest.reporter.id, 71201308);
  assert.equal(manifest.cases.length, 17);
  assert.equal(new Set(manifest.cases.map((entry) => entry.id)).size, 17);
  const config = fs.readFileSync(path.join(fixture, manifest.config.file));
  assert.equal(config.length, manifest.config.bytes);
  assert.equal(sha256(config), manifest.config.sha256);
  const build = expectedBuildIdentity(root);
  const receipt = JSON.parse(
    fs.readFileSync(path.join(root, `${build.binaryPath}.differential-build.json`), "utf8"),
  );
  validateBuildReceipt(receipt, build);
  const cli = path.join(root, build.binaryPath);
  const output = path.join(root, "target/differential/css-id-selector-ranges-cli.json");
  fs.mkdirSync(path.dirname(output), { recursive: true });
  const evidence = { schema: "vize.css-id-selector-ranges", version: 1, build, receipt, runs: [] };
  const persist = () => fs.writeFileSync(output, JSON.stringify(evidence, null, 2) + "\n");
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-css-id-ranges-"));
  try {
    fs.writeFileSync(path.join(directory, "vize.config.json"), config);
    for (const entry of manifest.cases) {
      const source = fs.readFileSync(path.join(fixture, entry.source));
      assert.equal(source.length, entry.bytes);
      assert.equal(sha256(source), entry.sha256);
      const expected = JSON.parse(fs.readFileSync(path.join(fixture, entry.expected), "utf8"));
      fs.writeFileSync(path.join(directory, entry.filename), source);
      for (const format of entry.id === "original-7981"
        ? ["json", "plain", "json"]
        : ["json", "plain"]) {
        const args = [
          "lint",
          "-f",
          format,
          "--help-level",
          "none",
          "--locale",
          "en",
          entry.filename,
        ];
        const result = spawnSync(cli, args, {
          cwd: directory,
          encoding: "utf8",
          timeout: 60_000,
          maxBuffer: 8 * 1024 * 1024,
        });
        const row = {
          case: entry,
          source: source.toString("utf8"),
          config: config.toString("utf8"),
          args,
          status: result.status,
          signal: result.signal,
          error: result.error ? { name: result.error.name, message: result.error.message } : null,
          stdout: result.stdout ?? "",
          stderr: result.stderr ?? "",
          expected: expected[format],
        };
        evidence.runs.push(row);
        persist();
        assert.equal(row.error, null, JSON.stringify(row));
        assert.equal(row.signal, null, JSON.stringify(row));
        assert.equal(row.status, 0, JSON.stringify(row));
        assert.equal(row.stderr, "", JSON.stringify(row));
        if (format === "json") {
          row.actual = JSON.parse(row.stdout);
          persist();
          assert.deepEqual(row.actual, expected.json, entry.id);
        } else {
          assert.equal(row.stdout, expected.plain, entry.id);
        }
      }
      fs.unlinkSync(path.join(directory, entry.filename));
    }
    assert.equal(evidence.runs.length, 35);
    assert.equal(evidence.runs[0].stdout, evidence.runs[2].stdout, "whole original JSON repeat");
  } finally {
    persist();
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
