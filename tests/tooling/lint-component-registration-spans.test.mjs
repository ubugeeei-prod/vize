import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { stripVTControlCharacters } from "node:util";
import { expectedBuildIdentity, validateBuildReceipt } from "../differential/build-receipt.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const fixture = path.join(root, "crates/vize_patina/tests/fixtures/issue-7979");
const rule = "vue/require-component-registration";
const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");
const help = "Import the component in <script setup> or register it in components option";

function position(source, offset) {
  const lines = source.slice(0, offset).split("\n");
  // The published lint JSON uses one-based Unicode scalar columns.
  return { line: lines.length, column: [...lines.at(-1)].length + 1 };
}

function expected(source, filename, tags) {
  let cursor = 0;
  const messages = tags.map((tag) => {
    const start = source.indexOf(`<${tag}`, cursor) + 1;
    assert.ok(start > cursor);
    const end = start + tag.length;
    assert.equal(source.slice(start, end), tag);
    cursor = end;
    const from = position(source, start);
    const to = position(source, end);
    return {
      ruleId: rule,
      ruleDocsPath: "docs/content/rules/vue.md",
      severity: 1,
      message: `[vize:${rule}] Component is used but not explicitly imported`,
      line: from.line,
      column: from.column,
      endLine: to.line,
      endColumn: to.column,
      help,
    };
  });
  return [{ file: filename, messages, errorCount: 0, warningCount: tags.length }];
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

await test("original component-registration CLI findings point to whole physical tag names", () => {
  const artifact = path.join(root, "target/differential/lint-component-registration-spans.json");
  fs.mkdirSync(path.dirname(artifact), { recursive: true });
  const evidence = {
    schema: "vize.lint.component-registration-spans",
    version: 1,
    issue: 7979,
    nativeHandled: 0,
    cliQualified: 0,
    runs: [],
  };
  const persist = () => fs.writeFileSync(artifact, JSON.stringify(evidence, null, 2) + "\n");
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-registration-spans-"));
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
    const original = fs.readFileSync(path.join(fixture, "my-panel.vue.txt"), "utf8");
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
    for (const [input, filename, tags] of [
      ["my-panel.vue.txt", "my-panel.vue", ["MyButton"]],
      ["crlf-panel.vue.txt", "my-panel.vue", ["MyButton"]],
      ["UnicodePanel.vue.txt", "UnicodePanel.vue", ["MissingPanel", "my-button"]],
      ["RegisteredPanel.vue.txt", "RegisteredPanel.vue", []],
    ]) {
      const bytes = fs.readFileSync(path.join(fixture, input));
      const source = bytes.toString("utf8");
      fs.writeFileSync(path.join(directory, filename), bytes);
      let selectedConfig = config;
      if (input === "RegisteredPanel.vue.txt") {
        // Preserve the original bytes and record the new unconfigured finding
        // before selecting the plugin registration this positive case assumes.
        const unconfigured = capture(
          binary,
          ["lint", filename, "--format", "json", "--locale", "en", "--help-level", "full"],
          directory,
        );
        evidence.unconfiguredRouter = { source, config, run: unconfigured };
        persist();
        assert.equal(unconfigured.error, null);
        assert.equal(unconfigured.signal, null);
        assert.equal(unconfigured.status, 0);
        assert.equal(unconfigured.stderr, "");
        assert.deepEqual(
          JSON.parse(unconfigured.stdout),
          expected(source, filename, ["router-link"]),
        );
        const configured = JSON.parse(config);
        configured.linter.ruleOptions = {
          "vue/require-component-registration": { globals: ["RouterLink"] },
        };
        selectedConfig = JSON.stringify(configured) + "\n";
        fs.writeFileSync(path.join(directory, "vize.config.json"), selectedConfig);
      }
      if (input === "crlf-panel.vue.txt") assert.equal(source, original.replaceAll("\n", "\r\n"));
      const oracle = expected(source, filename, tags);
      if (input === "my-panel.vue.txt") {
        const { line, column, endLine, endColumn } = oracle[0].messages[0];
        assert.deepEqual({ line, column, endLine, endColumn }, manifest.expectedOriginalRange);
      }
      const run = capture(
        binary,
        ["lint", filename, "--format", "json", "--locale", "en", "--help-level", "full"],
        directory,
      );
      evidence.runs.push({
        input,
        filename,
        source,
        config: selectedConfig,
        bytes: bytes.length,
        sha256: sha256(bytes),
        oracle,
        run,
      });
      persist();
      assert.equal(run.error, null, JSON.stringify(run));
      assert.equal(run.signal, null, JSON.stringify(run));
      assert.equal(run.status, 0, JSON.stringify(run));
      assert.equal(run.stderr, "");
      assert.deepEqual(JSON.parse(run.stdout), oracle);
      assert.equal(sha256(fs.readFileSync(path.join(directory, filename))), sha256(bytes));
      evidence.cliQualified++;
      persist();
      if (input === "my-panel.vue.txt") {
        const ansi = capture(
          binary,
          ["lint", filename, "--format", "ansi", "--locale", "en", "--help-level", "none"],
          directory,
        );
        evidence.ansi = { source, run: ansi };
        persist();
        assert.equal(ansi.error, null, JSON.stringify(ansi));
        assert.equal(ansi.signal, null, JSON.stringify(ansi));
        assert.equal(ansi.status, 0, JSON.stringify(ansi));
        assert.equal(ansi.stderr, "");
        const visible = stripVTControlCharacters(ansi.stdout);
        assert.match(visible, /my-panel\.vue:3:6/);
        assert.match(visible, /MyButton>OK<\/MyButton>/);
        assert.match(visible, /────────/);
      }
    }
    assert.equal(evidence.cliQualified, 4);
  } finally {
    persist();
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
