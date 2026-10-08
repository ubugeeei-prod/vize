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
const fixture = path.join(root, "crates/vize_patina/tests/fixtures/issue-7977");
const rule = "a11y/no-aria-hidden-on-focusable";
const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");

function position(source, offset) {
  const lines = source.slice(0, offset).split("\n");
  return { line: lines.length, column: [...lines.at(-1)].length + 1 };
}

function expected(source, filename, targets) {
  const messages = targets.map((target) => {
    const start = source.indexOf(target);
    assert.ok(start >= 0, target);
    assert.equal(source.slice(start, start + target.length), target);
    const from = position(source, start);
    // This fixture calls the template API, whose established span is the opening tag.
    const openingEnd = target.indexOf(">");
    assert.ok(openingEnd >= 0, target);
    const to = position(source, start + openingEnd + 1);
    return {
      ruleId: rule,
      ruleDocsPath: "docs/content/rules/accessibility.md",
      severity: 2,
      message: `[vize:${rule}] aria-hidden="true" must not be used on focusable elements`,
      line: from.line,
      column: from.column,
      endLine: to.line,
      endColumn: to.column,
      help: target.includes('tabindex="-1"')
        ? 'Remove aria-hidden="true". tabindex="-1" is still programmatically focusable'
        : 'Remove aria-hidden="true".',
    };
  });
  return [{ file: filename, messages, errorCount: targets.length, warningCount: 0 }];
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

await test("source CLI preserves the original static non-focusable controls and complete findings", () => {
  const artifact = path.join(root, "target/differential/lint-aria-hidden-static-focus.json");
  fs.mkdirSync(path.dirname(artifact), { recursive: true });
  const evidence = {
    schema: "vize.lint.aria-hidden-static-focus",
    version: 1,
    issue: 7977,
    nativeHandled: 0,
    cliQualified: 0,
    runs: [],
  };
  const persist = () => fs.writeFileSync(artifact, JSON.stringify(evidence, null, 2) + "\n");
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-aria-hidden-"));
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
    const originalIssue = fs.readFileSync(path.join(fixture, "original-issue.md"), "utf8");
    const original = fs.readFileSync(path.join(fixture, "MySelect.vue.txt"), "utf8");
    const config = fs.readFileSync(path.join(fixture, "vize.config.json"), "utf8");
    assert.equal(originalIssue.match(/```vue\n([\s\S]*?)```/)[1], original);
    assert.equal(originalIssue.match(/```json\n([\s\S]*?)```/)[1], config);
    fs.writeFileSync(path.join(directory, "vize.config.json"), config);
    const build = expectedBuildIdentity(root);
    const receipt = JSON.parse(
      fs.readFileSync(path.join(root, build.binaryPath + ".differential-build.json"), "utf8"),
    );
    evidence.build = { expected: build, receipt };
    persist();
    validateBuildReceipt(receipt, build);
    const binary = path.join(root, build.binaryPath);
    for (const [input, filename, targets] of [
      ["MySelect.vue.txt", "MySelect.vue", manifest.originalTargets],
      ["MySelectCRLF.vue.txt", "MySelect.vue", manifest.originalTargets],
      ["Controls.vue.txt", "Controls.vue", manifest.controlTargets],
    ]) {
      const bytes = fs.readFileSync(path.join(fixture, input));
      const source = bytes.toString("utf8");
      fs.writeFileSync(path.join(directory, filename), bytes);
      if (input === "MySelectCRLF.vue.txt") assert.equal(source, original.replaceAll("\n", "\r\n"));
      const oracle = expected(source, filename, targets);
      const row = {
        input,
        filename,
        source,
        bytes: bytes.length,
        sha256: sha256(bytes),
        oracle,
        captures: [],
      };
      evidence.runs.push(row);
      for (let repeat = 0; repeat < 2; repeat++) {
        const run = capture(
          binary,
          ["lint", filename, "--format", "json", "--locale", "en", "--help-level", "full"],
          directory,
        );
        row.captures.push(run);
        persist();
        assert.equal(run.error, null, JSON.stringify(run));
        assert.equal(run.signal, null, JSON.stringify(run));
        assert.equal(run.status, 1, JSON.stringify(run));
        assert.equal(run.stderr, "");
        assert.deepEqual(JSON.parse(run.stdout), oracle);
        assert.equal(sha256(fs.readFileSync(path.join(directory, filename))), sha256(bytes));
      }
      assert.equal(row.captures[0].stdout, row.captures[1].stdout);
      evidence.cliQualified++;
      persist();
    }
    assert.equal(evidence.cliQualified, 3);
  } finally {
    persist();
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
