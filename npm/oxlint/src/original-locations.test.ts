import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { loadBinding } from "./native.ts";
import { resetFixtureDir } from "./test-support/fixture-dir.ts";
import type { PatinaLintResult } from "./model.js";

const root = path.resolve(fileURLToPath(new URL("..", import.meta.url)), "../..");
const corpus = path.join(root, "tests/_fixtures/differential/lint/oxlint-original-locations-7904");
const fixture = path.join(root, "target/vize-tests/oxlint-original-locations-7904");
const capturePath =
  process.env.VIZE_OXLINT_ORIGINAL_CAPTURE ??
  path.join(root, "target/oxlint-original-locations-7904.json");
const cli = path.join(root, "npm/oxlint/dist/cli.mjs");
const plugin = path.join(root, "npm/oxlint/dist/index.mjs");
process.env.VIZE_PREFER_WORKSPACE_BINDING = "1";
const cliEnv = { ...process.env };
// Match the reported terminal invocation, as the existing integration suite does.
delete cliEnv.GITHUB_ACTIONS;
const pins = JSON.parse(fs.readFileSync(path.join(corpus, "source.json"), "utf8")) as {
  files: Array<{ file: string; bytes: number; sha256: string }>;
};
type Row = {
  filename: string;
  file: string;
  native: PatinaLintResult;
  jsonDiagnostics: Diagnostic[];
};
type Diagnostic = {
  filename: string;
  code: string;
  severity: "error" | "warning";
  labels: Array<{ span: { offset: number; length: number; line: number; column: number } }>;
};
const rows = JSON.parse(fs.readFileSync(path.join(corpus, "expected.json"), "utf8")) as Row[];
const capture: { schema: string; complete: boolean; observations: unknown[] } = {
  schema: "vize.oxlint-original-locations-7904.v1",
  complete: false,
  observations: [],
};
function save(observation: unknown): void {
  capture.observations.push(observation);
  fs.mkdirSync(path.dirname(capturePath), { recursive: true });
  fs.writeFileSync(capturePath, JSON.stringify(capture, null, 2) + "\n");
}
const digest = (bytes: string | Buffer) => createHash("sha256").update(bytes).digest("hex");
const ordered = <T extends { filename?: string; rule?: string; code?: string }>(items: T[]) =>
  [...items].sort((a, b) => {
    const left = `${a.filename ?? ""}:${a.rule ?? a.code ?? ""}`;
    const right = `${b.filename ?? ""}:${b.rule ?? b.code ?? ""}`;
    return left < right ? -1 : left > right ? 1 : 0;
  });

resetFixtureDir(fixture);
try {
  for (const pin of pins.files) {
    const bytes = fs.readFileSync(path.join(corpus, pin.file));
    assert.equal(bytes.length, pin.bytes, pin.file);
    assert.equal(digest(bytes), pin.sha256, pin.file);
  }
  const config = JSON.parse(fs.readFileSync(path.join(corpus, ".oxlintrc.json"), "utf8"));
  // Only resolve the exact original package specifier to this commit's packed plugin.
  config.jsPlugins = [plugin];
  fs.writeFileSync(path.join(fixture, ".oxlintrc.json"), JSON.stringify(config));
  fs.mkdirSync(path.join(fixture, "node_modules/oxlint/bin"), { recursive: true });
  fs.symlinkSync(
    path.join(root, "node_modules/oxlint/bin/oxlint"),
    path.join(fixture, "node_modules/oxlint/bin/oxlint"),
  );
  const nativeFiles = fs
    .readdirSync(path.join(root, "npm/native"))
    .filter((name) => name.endsWith(".node"));
  assert.ok(nativeFiles.length > 0, "this checkout's freshly built native binding is required");
  save({
    node: process.version,
    cliSha256: digest(fs.readFileSync(cli)),
    pluginSha256: digest(fs.readFileSync(plugin)),
    nativeFiles: nativeFiles.map((name) => ({
      name,
      sha256: digest(fs.readFileSync(path.join(root, "npm/native", name))),
    })),
    pins,
  });

  function run(args: string[], status = 1) {
    const actual = spawnSync(process.execPath, [cli, ...args], {
      cwd: fixture,
      env: cliEnv,
      encoding: "utf8",
      maxBuffer: 8 * 1024 * 1024,
    });
    save({
      args,
      status: actual.status,
      signal: actual.signal,
      error: actual.error?.message ?? null,
      stdout: actual.stdout,
      stderr: actual.stderr,
    });
    assert.equal(actual.error, undefined);
    assert.equal(actual.signal, null);
    assert.equal(actual.status, status);
    assert.doesNotMatch(
      actual.stdout + actual.stderr,
      /node_modules[/\\]\.vize[/\\]oxlint-plugin-vize|Error running JS plugin|RangeError/u,
    );
    assert.equal(fs.existsSync(path.join(fixture, "node_modules/.vize/oxlint-plugin-vize")), false);
    return actual.stdout;
  }
  function checkJson(args: string[], expected: Diagnostic[]) {
    const raw = run(
      ["-f", "json", ...args],
      expected.some((diagnostic) => diagnostic.severity === "error") ? 1 : 0,
    );
    const report = JSON.parse(raw) as {
      diagnostics: Diagnostic[];
      number_of_files: number;
      number_of_rules: number;
      threads_count: number;
      start_time: number;
    };
    assert.deepEqual(Object.keys(report).sort(), [
      "diagnostics",
      "number_of_files",
      "number_of_rules",
      "start_time",
      "threads_count",
    ]);
    assert.equal(report.number_of_files, args.length);
    assert.ok(Number.isSafeInteger(report.number_of_rules) && report.number_of_rules > 0);
    assert.ok(Number.isSafeInteger(report.threads_count) && report.threads_count >= 0);
    assert.ok(Number.isFinite(report.start_time) && report.start_time >= 0);
    assert.deepEqual(ordered(report.diagnostics), ordered(expected));
    for (const diagnostic of report.diagnostics) {
      const source = fs.readFileSync(path.join(fixture, diagnostic.filename));
      for (const { span } of diagnostic.labels) {
        assert.ok(span.offset >= 0 && span.offset + span.length <= source.length);
        if (diagnostic.code === "vize(vue/no-v-html)")
          assert.match(
            source.subarray(span.offset, span.offset + span.length).toString(),
            /^v-html="[^"\n]*"$/u,
          );
      }
    }
  }

  for (const row of rows) {
    const source = fs.readFileSync(path.join(corpus, row.file), "utf8");
    fs.writeFileSync(path.join(fixture, row.filename), source);
    const actual = loadBinding().lintPatinaSfc(source, {
      filename: row.filename,
      preset: "essential",
      helpLevel: "full",
      enabledRules: ["vue/no-v-html", "vue/multi-word-component-names"],
    });
    save({ filename: row.filename, source, sourceSha256: digest(source), native: actual });
    assert.deepEqual(
      { ...actual, diagnostics: ordered(actual.diagnostics) },
      { ...row.native, diagnostics: ordered(row.native.diagnostics) },
    );
  }
  // The original two full files and all four diagnostics, before derived cases.
  checkJson(
    ["Panel.vue", "Static.vue"],
    rows.slice(0, 2).flatMap((row) => row.jsonDiagnostics),
  );
  for (const row of rows) checkJson([row.filename], row.jsonDiagnostics);
  const graphical = run(["Static.vue"]);
  const ansi = String.raw`\u001B\[[0-9;]*m`;
  assert.match(
    graphical,
    new RegExp(String.raw`╭─\[(?:${ansi})*Static\.vue(?:${ansi})*:1:11\]`, "u"),
  );
  const plain = run(["-f", "unix", "Static.vue"]);
  assert.match(plain, /^Static\.vue:1:11: Component name "Static" should be multi-word/mu);
  const stylish = run(["-f", "stylish", "Unicode.vue"]);
  assert.match(stylish, /^\s+1:28\s+warning\s+v-html can lead to XSS attacks\./mu);
  assert.match(stylish, /^\s+1:11\s+error\s+Component name "Unicode" should be multi-word/mu);
  // The real scoped-config transport must bind the same four original diagnostics.
  config.overrides = [{ files: ["*.vue"], rules: {} }];
  fs.writeFileSync(path.join(fixture, ".oxlintrc.json"), JSON.stringify(config));
  checkJson(
    ["Panel.vue", "Static.vue"],
    rows.slice(0, 2).flatMap((row) => row.jsonDiagnostics),
  );
  capture.complete = true;
  save({ terminal: "all original/native/JSON/plain/stylish/scoped controls passed" });
} finally {
  fs.rmSync(fixture, { recursive: true, force: true });
}
