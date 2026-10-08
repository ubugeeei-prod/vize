import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { expectedBuildIdentity, validateBuildReceipt } from "../differential/build-receipt.ts";
import { sha256 } from "../differential/manifest.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const corpusPath =
  "tests/_fixtures/differential/formatter-regressions/root-comment-attachment-7877/cases.json";
const raw = fs.readFileSync(path.join(root, corpusPath));
const corpus = JSON.parse(raw);

void test("original root comments retain whole attachment bytes and CLI fixed points", (t) => {
  const identity = expectedBuildIdentity(root);
  const receipt = JSON.parse(
    fs.readFileSync(path.join(root, `${identity.binaryPath}.differential-build.json`)),
  );
  validateBuildReceipt(receipt, identity);
  assert.equal(corpus.issue, 7877);
  assert.equal(corpus.cases.length, 23);
  assert.equal(sha256(Buffer.from(corpus.cases[0].source)), corpus.source.originalSha256);
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-root-comment-"));
  const report = {
    schema: "vize.formatter-root-comment-observation",
    version: 1,
    corpusPath,
    corpusSha256: sha256(raw),
    buildReceipt: receipt,
    nativeHandled: 0,
    historicalDifferent: 0,
    currentMatches: 0,
    rows: [],
  };
  try {
    fs.mkdirSync(path.join(directory, ".git"));
    for (const fixture of corpus.cases) {
      const file = path.join(directory, fixture.file);
      fs.writeFileSync(file, fixture.source);
      const config = {
        formatter: {
          ...(fixture.eol ? { endOfLine: fixture.eol } : {}),
          ...(fixture.sortBlocks === undefined ? {} : { sortBlocks: fixture.sortBlocks }),
          ...(fixture.sortAttributes === undefined
            ? {}
            : { sortAttributes: fixture.sortAttributes }),
        },
      };
      const configured = Object.keys(config.formatter).length > 0;
      if (configured)
        fs.writeFileSync(path.join(directory, "vize.config.json"), `${JSON.stringify(config)}\n`);
      if (fixture.existingHistoryId) {
        for (const original of [fixture.existingInput, fixture.existingExpected]) {
          assert.equal(sha256(fs.readFileSync(path.join(root, original.path))), original.sha256);
        }
      }
      const row = {
        id: fixture.id,
        input: fixture.source,
        expected: fixture.expected,
        inputSha256: sha256(Buffer.from(fixture.source)),
        attempts: [],
      };
      report.rows.push(row);
      const invoke = (mode) => {
        const argv = [
          "fmt",
          mode,
          fixture.file,
          ...(configured ? ["--config", "vize.config.json"] : ["--no-config"]),
        ];
        const before = fs.readFileSync(file);
        const result = spawnSync(path.join(root, identity.binaryPath), argv, {
          cwd: directory,
          env: { ...process.env, NO_COLOR: "1" },
          timeout: 30000,
          maxBuffer: 1048576,
        });
        const attempt = {
          argv,
          status: result.status,
          signal: result.signal,
          error: result.error?.message ?? null,
          stdout: result.stdout?.toString() ?? "",
          stderr: result.stderr?.toString() ?? "",
          before: before.toString(),
          after: null,
          beforeSha256: sha256(before),
          afterSha256: null,
          afterReadError: null,
        };
        row.attempts.push(attempt);
        let after;
        try {
          after = fs.readFileSync(file);
          attempt.after = after.toString();
          attempt.afterSha256 = sha256(after);
        } catch (error) {
          attempt.afterReadError = error.message;
          throw error;
        }
        assert.equal(result.error, undefined, fixture.id);
        assert.equal(result.signal, null, fixture.id);
        return { result, before, after };
      };
      const initial = invoke("--check");
      assert.deepEqual(initial.after, initial.before, `${fixture.id}: check must be read-only`);
      assert.equal(initial.result.status, fixture.source === fixture.expected ? 0 : 1, fixture.id);
      for (let pass = 1; pass <= 3; pass++) {
        const { result, after } = invoke("--write");
        assert.equal(result.status, 0, `${fixture.id}: pass ${pass}`);
        assert.deepEqual(
          after,
          Buffer.from(fixture.expected),
          `${fixture.id}: complete pass ${pass}`,
        );
        if (fixture.historicalId) {
          assert.equal(
            sha256(Buffer.from(fixture.historicalExpected)),
            fixture.historicalExpectedSha256,
          );
          assert.notDeepEqual(after, Buffer.from(fixture.historicalExpected));
          row.historicalComparison = "different";
          row.currentComparison = "equal";
        }
      }
      const final = invoke("--check");
      assert.equal(final.result.status, 0, fixture.id);
      assert.deepEqual(final.after, final.before, `${fixture.id}: final check must be read-only`);
      if (fixture.historicalId) report.historicalDifferent++;
      report.currentMatches++;
    }
    assert.equal(report.currentMatches, 23);
    assert.equal(report.historicalDifferent, 2);
  } finally {
    const artifact = path.join(
      root,
      "target/differential/formatter-root-comment-7877/cli-report.json",
    );
    fs.mkdirSync(path.dirname(artifact), { recursive: true });
    fs.writeFileSync(artifact, `${JSON.stringify(report, null, 2)}\n`);
    t.diagnostic(`Whole original/current outputs and CLI processes: ${artifact}`);
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
