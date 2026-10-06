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
const fixture = path.join(root, "crates/vize_patina/tests/fixtures/issue-7963");
const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");
const rule = "nuxt/nuxt-config-keys-order";

function expectedJson(entry, fixed) {
  const messages = [];
  if (!fixed && entry.finding) {
    const start = entry.source.indexOf(entry.finding.target);
    assert.ok(start >= 0, entry.id);
    const location = (offset) => {
      const lines = entry.source.slice(0, offset).split("\n");
      return { line: lines.length, column: [...lines.at(-1)].length + 1 };
    };
    const from = location(start);
    const to = location(start + entry.finding.target.length);
    messages.push({
      ruleId: rule,
      ruleDocsPath: "docs/content/rules/index.md",
      severity: 2,
      message: `[vize:${rule}] Expected config key "${entry.finding.left}" to come after "${entry.finding.right}"`,
      line: from.line,
      column: from.column,
      endLine: to.line,
      endColumn: to.column,
    });
  }
  return [{ file: entry.filename, messages, errorCount: messages.length, warningCount: 0 }];
}

await test("source CLI preserves Nuxt comma style through full lint/fix/check/idempotence", () => {
  const artifact = path.join(root, "target/differential/nuxt-key-order-comma-cli.json");
  fs.mkdirSync(path.dirname(artifact), { recursive: true });
  const evidence = { schema: "vize.lint.nuxt-key-order-comma", version: 1, runs: [] };
  const persist = () => fs.writeFileSync(artifact, JSON.stringify(evidence, null, 2) + "\n");
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-nuxt-comma-"));
  try {
    const build = expectedBuildIdentity(root);
    const receipt = JSON.parse(
      fs.readFileSync(path.join(root, build.binaryPath + ".differential-build.json"), "utf8"),
    );
    evidence.build = { receipt, expected: build };
    evidence.fixture = JSON.parse(fs.readFileSync(path.join(fixture, "source.json"), "utf8"));
    persist();
    validateBuildReceipt(receipt, build);
    assert.equal(evidence.fixture.issue, 7963);
    assert.deepEqual(evidence.fixture.author, {
      login: "ubugeeei",
      id: 71201308,
      coAuthor: "ubugeeei <71201308+ubugeeei@users.noreply.github.com>",
    });
    for (const input of evidence.fixture.inputs) {
      const bytes = fs.readFileSync(path.join(fixture, input.file));
      assert.equal(bytes.length, input.bytes, input.file);
      assert.equal(sha256(bytes), input.sha256, input.file);
    }
    const cases = JSON.parse(fs.readFileSync(path.join(fixture, "cases.json"), "utf8"));
    assert.equal(cases.length, evidence.fixture.caseCount);
    assert.equal(
      cases[0].source,
      fs.readFileSync(path.join(fixture, "nuxt.config.ts.fixture"), "utf8"),
    );
    assert.equal(
      cases[0].fixed,
      fs.readFileSync(path.join(fixture, "expected.config.ts.fixture"), "utf8"),
    );
    for (const entry of cases) {
      fs.writeFileSync(path.join(directory, entry.filename), entry.source);
      for (const mode of ["lint", "fix", "check", "refix"]) {
        const shouldFix = mode === "fix" || mode === "refix";
        const args = [
          "lint",
          "--no-config",
          "--preset",
          "nuxt",
          "--format",
          "json",
          "--locale",
          "en",
          "--help-level",
          "none",
          ...(shouldFix ? ["--fix"] : []),
          entry.filename,
        ];
        const run = spawnSync(path.join(root, build.binaryPath), args, {
          cwd: directory,
          env: process.env,
          encoding: "utf8",
          timeout: 60_000,
          maxBuffer: 4 * 1024 * 1024,
        });
        const output = fs.readFileSync(path.join(directory, entry.filename));
        const observation = {
          id: entry.id,
          mode,
          argv: args,
          status: run.status,
          signal: run.signal,
          error: run.error ? { name: run.error.name, message: run.error.message } : null,
          stdout: run.stdout ?? "",
          stderr: run.stderr ?? "",
          output: { text: output.toString("utf8"), sha256: sha256(output) },
          expected: expectedJson(entry, mode !== "lint"),
        };
        evidence.runs.push(observation);
        persist();
        assert.equal(observation.error, null, entry.id);
        assert.equal(run.signal, null, entry.id);
        assert.equal(
          run.status,
          mode === "lint" && entry.finding ? 1 : 0,
          JSON.stringify(observation),
        );
        assert.equal(observation.stderr, "", JSON.stringify(observation));
        assert.equal(
          output.toString("utf8"),
          mode === "lint" ? entry.source : entry.fixed,
          entry.id,
        );
        observation.actual = JSON.parse(observation.stdout);
        persist();
        assert.deepEqual(observation.actual, observation.expected, JSON.stringify(observation));
      }
      fs.unlinkSync(path.join(directory, entry.filename));
    }
    assert.equal(evidence.runs.length, 60);
    console.log("VIZE_NUXT_COMMA_FIX", JSON.stringify({ ...build, runs: evidence.runs.length }));
  } finally {
    try {
      persist();
    } finally {
      fs.rmSync(directory, { recursive: true, force: true });
    }
  }
});
