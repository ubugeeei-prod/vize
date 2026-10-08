import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { expectedBuildIdentity, validateBuildReceipt } from "../differential/build-receipt.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const fixture = path.join(root, "crates/vize_patina/tests/fixtures/issue-8018");
const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");

await test("source-built lint matches complete original and configured content-directive reports", () => {
  const source = JSON.parse(fs.readFileSync(path.join(fixture, "source.json"), "utf8"));
  assert.deepEqual(source.author, {
    login: "ubugeeei",
    id: 71201308,
    coauthor: "ubugeeei <71201308+ubugeeei@users.noreply.github.com>",
  });
  for (const row of [...source.inputs, ...source.authored]) {
    const bytes = fs.readFileSync(path.join(fixture, row.file));
    assert.equal(bytes.length, row.bytes);
    assert.equal(sha256(bytes), row.sha256);
  }
  const references = JSON.parse(fs.readFileSync(path.join(fixture, "references.json"), "utf8"));
  assert.equal(references.scenarios.length, 11);
  const build = expectedBuildIdentity(root);
  const receipt = JSON.parse(
    fs.readFileSync(path.join(root, build.binaryPath + ".differential-build.json"), "utf8"),
  );
  validateBuildReceipt(receipt, build);
  const artifact = path.join(root, "target/differential/lint-content-directives.json");
  fs.mkdirSync(path.dirname(artifact), { recursive: true });
  const evidence = {
    schema: "vize.lint.content-directives.execution",
    source,
    build,
    receipt,
    runs: [],
  };
  const persist = () => fs.writeFileSync(artifact, JSON.stringify(evidence, null, 2) + "\n");
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-content-directives-"));
  const cli = path.join(root, build.binaryPath);
  const run = (args) => {
    const actual = spawnSync(cli, args, {
      cwd: directory,
      env: process.env,
      encoding: "utf8",
      timeout: 30_000,
      maxBuffer: 2 * 1024 * 1024,
    });
    return {
      args,
      status: actual.status,
      signal: actual.signal,
      error: actual.error?.message ?? null,
      stdout: actual.stdout ?? "",
      stderr: actual.stderr ?? "",
    };
  };
  try {
    for (const row of source.inputs)
      fs.copyFileSync(path.join(fixture, row.file), path.join(directory, row.path));
    fs.copyFileSync(
      path.join(fixture, "Controls.vue.fixture"),
      path.join(directory, "Controls.vue"),
    );
    for (const scenario of references.scenarios) {
      fs.writeFileSync(path.join(directory, "vize.config.json"), JSON.stringify(scenario.config));
      for (let pass = 0; pass < 3; pass++) {
        const raw = run([
          "lint",
          "-f",
          "json",
          "--locale",
          "en",
          "--help-level",
          "full",
          scenario.file,
        ]);
        const observation = {
          id: scenario.id,
          pass,
          config: scenario.config,
          raw,
          expected: scenario.expectedCli,
        };
        evidence.runs.push(observation);
        persist();
        assert.equal(raw.error, null, scenario.id);
        assert.equal(raw.signal, null, scenario.id);
        assert.equal(
          raw.status,
          scenario.expectedCli[0].errorCount > 0 ? 1 : 0,
          JSON.stringify(raw),
        );
        assert.deepEqual(JSON.parse(raw.stdout), scenario.expectedCli, scenario.id);
      }
    }
    // Run the literal reported command against its untouched original config.
    fs.copyFileSync(
      path.join(fixture, "vize.config.json.fixture"),
      path.join(directory, "vize.config.json"),
    );
    const plain = run(["lint", "-f", "plain", "--help-level", "none", "Notice.vue"]);
    evidence.reportedPlain = plain;
    persist();
    assert.equal(plain.error, null);
    assert.equal(plain.signal, null);
    assert.equal(plain.status, 1);
    assert.equal(
      plain.stdout,
      "Patina lint report: 1 error in 1 file\n\nNotice.vue\n  Notice.vue:10:5 error html/no-empty-palpable-content <p> element is empty but expects visible content\n    Reference: docs/content/rules/html.md\n",
    );
    for (const row of source.inputs) {
      if (row.path !== "vize.config.json")
        assert.equal(sha256(fs.readFileSync(path.join(directory, row.path))), row.sha256);
    }
    assert.equal(
      sha256(fs.readFileSync(path.join(directory, "Controls.vue"))),
      source.authored[0].sha256,
    );
  } finally {
    try {
      persist();
    } finally {
      fs.rmSync(directory, { recursive: true, force: true });
    }
  }
});
