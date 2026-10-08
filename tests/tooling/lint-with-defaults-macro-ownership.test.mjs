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
const fixture = path.join(root, "tests/_fixtures/differential/lint-with-defaults");
const rule = "script/no-with-defaults";
const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");
const help =
  "Use destructuring with defaults: const { count = 0, name = 'default' } = defineProps<Props>()";

function position(bytes, offset) {
  const lines = bytes.subarray(0, offset).toString("utf8").split("\n");
  return { line: lines.length, column: [...lines.at(-1)].length + 1 };
}

function expected(bytes, entry) {
  const messages = entry.spans.map(([start, end]) => {
    const from = position(bytes, start);
    const to = position(bytes, end);
    return {
      ruleId: rule,
      ruleDocsPath: "docs/content/rules/type-and-script.md",
      severity: 2,
      message: `[vize:${rule}] Prefer destructuring defaults over withDefaults (Vue 3.5+)`,
      line: from.line,
      column: from.column,
      endLine: to.line,
      endColumn: to.column,
      help,
    };
  });
  return [{ file: entry.filename, messages, errorCount: messages.length, warningCount: 0 }];
}

function capture(binary, args, directory) {
  const run = spawnSync(binary, args, {
    cwd: directory,
    encoding: "utf8",
    timeout: 60_000,
    maxBuffer: 4 * 1024 * 1024,
    env: { ...process.env, NO_COLOR: "1" },
  });
  return {
    args,
    status: run.status,
    signal: run.signal,
    error: run.error ? { name: run.error.name, message: run.error.message } : null,
    stdout: run.stdout ?? "",
    stderr: run.stderr ?? "",
  };
}

function assert_run(run, status) {
  assert.equal(run.error, null, JSON.stringify(run));
  assert.equal(run.signal, null, JSON.stringify(run));
  assert.equal(run.status, status, JSON.stringify(run));
  assert.equal(run.stderr, "");
}

await test("original withDefaults CLI corpus reports only actual script-setup macro owners", () => {
  const artifact = path.join(root, "target/differential/lint-with-defaults-macro-ownership.json");
  fs.mkdirSync(path.dirname(artifact), { recursive: true });
  const evidence = {
    schema: "vize.lint.with-defaults-macro-ownership",
    version: 1,
    issue: 7962,
    nativeHandled: 0,
    cliQualified: 0,
    runs: [],
  };
  const persist = () => fs.writeFileSync(artifact, JSON.stringify(evidence, null, 2) + "\n");
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-with-defaults-"));
  try {
    const manifest = JSON.parse(fs.readFileSync(path.join(fixture, "source.json"), "utf8"));
    assert.equal(manifest.author.login, "ubugeeei");
    assert.equal(manifest.author.id, 71201308);
    evidence.manifest = manifest;
    for (const [file, pin] of Object.entries(manifest.files)) {
      const bytes = fs.readFileSync(path.join(fixture, file));
      assert.equal(bytes.length, pin.bytes, file);
      assert.equal(sha256(bytes), pin.sha256, file);
    }
    const issue = fs.readFileSync(path.join(fixture, "original-issue.md"), "utf8");
    const config = fs.readFileSync(path.join(fixture, "vize.config.json"), "utf8");
    assert.equal(issue.match(/```json\n([\s\S]*?)```/)[1], config);
    assert.equal(
      issue.match(/```vue\n([\s\S]*?)```/)[1],
      fs.readFileSync(path.join(fixture, "SizeLabel.vue.txt"), "utf8"),
    );
    const ts = [...issue.matchAll(/```ts\n([\s\S]*?)```/g)].map((match) => match[1]);
    assert.deepEqual(
      ts,
      ["comment-only.ts.txt", "source-check.ts.txt"].map((file) =>
        fs.readFileSync(path.join(fixture, file), "utf8"),
      ),
    );
    fs.writeFileSync(path.join(directory, "vize.config.json"), config);
    const build = expectedBuildIdentity(root);
    const receipt = JSON.parse(
      fs.readFileSync(path.join(root, build.binaryPath + ".differential-build.json"), "utf8"),
    );
    evidence.build = { expected: build, receipt };
    persist();
    validateBuildReceipt(receipt, build);
    const binary = path.join(root, build.binaryPath);
    const cases = JSON.parse(fs.readFileSync(path.join(fixture, "cases.json"), "utf8"));
    assert.equal(cases.length, 17);
    assert.equal(new Set(cases.map((entry) => entry.id)).size, cases.length);
    for (const entry of cases) {
      const bytes = fs.readFileSync(path.join(fixture, entry.input));
      fs.writeFileSync(path.join(directory, entry.filename), bytes);
      const oracle = expected(bytes, entry);
      for (let repeat = 0; repeat < 2; repeat++) {
        const run = capture(
          binary,
          ["lint", entry.filename, "--format", "json", "--locale", "en", "--help-level", "full"],
          directory,
        );
        evidence.runs.push({
          entry,
          repeat,
          source: bytes.toString("utf8"),
          sha256: sha256(bytes),
          oracle,
          run,
        });
        persist();
        assert_run(run, entry.spans.length ? 1 : 0);
        assert.deepEqual(JSON.parse(run.stdout), oracle);
        assert.equal(sha256(fs.readFileSync(path.join(directory, entry.filename))), sha256(bytes));
        evidence.cliQualified++;
        persist();
      }
    }
    const originalFiles = ["SizeLabel.vue", "comment-only.ts", "source-check.ts"];
    const original = capture(
      binary,
      ["lint", "-f", "plain", "--help-level", "none", ...originalFiles],
      directory,
    );
    evidence.originalCommand = { originalFiles, run: original };
    persist();
    assert_run(original, 0);
    assert.equal(original.stdout, "Patina lint report: No problems found in 3 file(s)\n");
    assert.equal(evidence.cliQualified, 34);
  } finally {
    persist();
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
